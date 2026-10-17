//! The device's storage: one IndexedDB database, and the only module that
//! touches it (ADR 0007).
//!
//! Records are stored as JSON text, the same shape the export bundle carries,
//! so every conversion between a record and a `domain` type stays pure and
//! tests on the host; only [`read`] and [`write`] reach the browser. The
//! database holds three collections plus a meta record, and its structural
//! version starts at 1.

// region:    --- Modules

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use rexie::{ObjectStore, Rexie, Transaction, TransactionMode};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::Result;

pub mod answer_log;
pub mod bundle;
pub mod exam;
pub mod hint;
pub mod profile;

// endregion: --- Modules

/// The Applicant's Profile: one record.
pub const PROFILE_STORE: &str = "profile";

/// The append-only Answer Log.
pub const ANSWER_LOG_STORE: &str = "answer_log";

/// The Exam records.
pub const EXAMS_STORE: &str = "exams";

/// The database's own bookkeeping: its version, and the last export.
pub const META_STORE: &str = "meta";

/// Every store the database holds, in the order a reset empties them.
const STORES: [&str; 4] = [PROFILE_STORE, ANSWER_LOG_STORE, EXAMS_STORE, META_STORE];

/// The database on the device.
const DATABASE: &str = "uscis-n400";

/// The database's structural version (ADR 0007: it starts at 1).
const DATABASE_VERSION: u32 = 1;

thread_local! {
    /// The open database, kept for the life of the page.
    static OPEN_DATABASE: RefCell<Option<Rc<Rexie>>> = const { RefCell::new(None) };

    /// Whether an open is in progress, so two screens cannot race the open.
    static OPENING: Cell<bool> = const { Cell::new(false) };
}

/// Reads the JSON text a record holds, if the store holds that key.
pub async fn read(store_name: &str, key: &str) -> Result<Option<String>> {
    let database = database().await?;
    let transaction = database.transaction(&[store_name], TransactionMode::ReadOnly)?;
    let value = transaction
        .store(store_name)?
        .get(JsValue::from_str(key))
        .await?;

    transaction.done().await?;

    let Some(value) = value else {
        return Ok(None);
    };
    let record = value.as_string().ok_or("the stored record is not text")?;

    Ok(Some(record))
}

/// Writes a record's JSON text under a key, replacing what was there.
pub async fn write(store_name: &str, key: &str, record: &str) -> Result<()> {
    let database = database().await?;
    let transaction = database.transaction(&[store_name], TransactionMode::ReadWrite)?;

    transaction
        .store(store_name)?
        .put(&JsValue::from_str(record), Some(&JsValue::from_str(key)))
        .await?;
    transaction.commit().await?;

    Ok(())
}

/// Starts over: empties every store, so the device holds no Profile, no Answer
/// Log, no Exam records, and no bookkeeping.
///
/// One transaction covers all four stores, so the device changes from its whole
/// state to none of it and never in between. The worker and its cache are not
/// this database and are untouched.
pub async fn reset() -> Result<()> {
    let database = database().await?;
    let transaction = database.transaction(&STORES, TransactionMode::ReadWrite)?;

    for store_name in STORES {
        transaction.store(store_name)?.clear().await?;
    }
    transaction.commit().await?;

    Ok(())
}

/// Reads every record's JSON text, in key order.
pub async fn read_all(store_name: &str) -> Result<Vec<String>> {
    let database = database().await?;
    let transaction = database.transaction(&[store_name], TransactionMode::ReadOnly)?;
    let values = transaction.store(store_name)?.get_all(None, None).await?;

    transaction.done().await?;

    values
        .into_iter()
        .map(|value| {
            value
                .as_string()
                .ok_or_else(|| crate::Error::custom("a stored record is not text"))
        })
        .collect()
}

/// The count a meta key records, if it holds a number.
pub async fn meta_count(key: &str) -> Result<Option<u64>> {
    let Some(value) = read(META_STORE, key).await? else {
        return Ok(None);
    };

    Ok(value.parse().ok())
}

/// Notes a store's current count in the meta record, inside `transaction`.
///
/// The count and the record it counts commit together, so an empty read later
/// can be recognized as suspicious rather than mistaken for no progress.
pub(super) async fn write_store_count(
    transaction: &Transaction,
    store_name: &str,
    count_key: &str,
) -> Result<u32> {
    let count = transaction.store(store_name)?.count(None).await?;

    transaction
        .store(META_STORE)?
        .put(
            &JsValue::from_str(&count.to_string()),
            Some(&JsValue::from_str(count_key)),
        )
        .await?;

    Ok(count)
}

/// Writes a count into the meta record.
pub(super) async fn write_count(key: &str, count: u64) -> Result<()> {
    write(META_STORE, key, &count.to_string()).await
}

/// Asks the browser to keep this origin's storage, so it is not evicted under
/// storage pressure.
///
/// The answer is informational: a refusal changes nothing, and the app keeps
/// working. The request is safe to repeat; the browser answers from its own
/// record.
pub async fn request_persistent_storage() {
    let Some(window) = web_sys::window() else {
        return;
    };

    if let Ok(promise) = window.navigator().storage().persist() {
        let _ = JsFuture::from(promise).await;
    }
}

// region:    --- Support

/// The device's clock, in milliseconds since the Unix epoch.
pub fn now() -> u64 {
    js_sys::Date::now() as u64
}

/// The database, opened on first use.
///
/// Only one open runs at a time: screens mount together and each asks for the
/// database, so a second caller waits for the first rather than opening a
/// second connection and reading beside it.
async fn database() -> Result<Rc<Rexie>> {
    loop {
        if let Some(database) = OPEN_DATABASE.with(|open| open.borrow().clone()) {
            return Ok(database);
        }

        if !OPENING.with(Cell::get) {
            break;
        }

        TimeoutFuture::new(10).await;
    }

    OPENING.with(|opening| opening.set(true));
    let built = Rexie::builder(DATABASE)
        .version(DATABASE_VERSION)
        .add_object_store(ObjectStore::new(PROFILE_STORE))
        .add_object_store(ObjectStore::new(ANSWER_LOG_STORE).auto_increment(true))
        .add_object_store(ObjectStore::new(EXAMS_STORE).auto_increment(true))
        .add_object_store(ObjectStore::new(META_STORE))
        .build()
        .await;
    OPENING.with(|opening| opening.set(false));

    let database = Rc::new(built?);
    OPEN_DATABASE.with(|open| *open.borrow_mut() = Some(Rc::clone(&database)));

    Ok(database)
}

// endregion: --- Support
