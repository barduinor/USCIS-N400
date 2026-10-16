//! The device's storage: one IndexedDB database, and the only module that
//! touches it (ADR 0007).
//!
//! Records are stored as JSON text, the same shape the export bundle carries,
//! so every conversion between a record and a `domain` type stays pure and
//! tests on the host; only [`read`] and [`write`] reach the browser. The
//! database holds three collections plus a meta record, and its structural
//! version starts at 1.

// region:    --- Modules

use std::cell::RefCell;
use std::rc::Rc;

use rexie::{ObjectStore, Rexie, TransactionMode};
use wasm_bindgen::JsValue;

use crate::Result;

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

/// The database on the device.
const DATABASE: &str = "uscis-n400";

/// The database's structural version (ADR 0007: it starts at 1).
const DATABASE_VERSION: u32 = 1;

thread_local! {
    /// The open database, kept for the life of the page.
    static OPEN_DATABASE: RefCell<Option<Rc<Rexie>>> = const { RefCell::new(None) };
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

// region:    --- Support

/// The database, opened on first use.
async fn database() -> Result<Rc<Rexie>> {
    if let Some(database) = OPEN_DATABASE.with(|open| open.borrow().clone()) {
        return Ok(database);
    }

    let database = Rc::new(
        Rexie::builder(DATABASE)
            .version(DATABASE_VERSION)
            .add_object_store(ObjectStore::new(PROFILE_STORE))
            .add_object_store(ObjectStore::new(ANSWER_LOG_STORE).auto_increment(true))
            .add_object_store(ObjectStore::new(EXAMS_STORE).auto_increment(true))
            .add_object_store(ObjectStore::new(META_STORE))
            .build()
            .await?,
    );

    OPEN_DATABASE.with(|open| *open.borrow_mut() = Some(Rc::clone(&database)));

    Ok(database)
}

// endregion: --- Support
