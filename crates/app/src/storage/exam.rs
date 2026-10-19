//! The Exam records: what each Exam came to.
//!
//! A record is JSON text, the same shape the export bundle will carry — the
//! track it ran under, the questions asked, the score, and the outcome — which
//! keeps the conversions pure and testable on the host. Only `append`,
//! `latest`, `replace_latest`, and `remove_latest` reach the browser.

use domain::ExamRecord;
use gloo_timers::future::TimeoutFuture;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use super::{
    EXAMS_STORE, META_STORE, database, meta_count, read_all, write_count, write_store_count,
};
use crate::{Error, Result};

/// The meta key that records how many Exam records the store held.
pub(super) const COUNT_KEY: &str = "exam_count";

/// How long the read waits before trying again, when the meta record says
/// Exams were taken but the store came back empty.
const RETRY_MILLIS: [u32; 3] = [50, 150, 400];

/// Appends one Exam's record.
///
/// The record and the store's count commit in one transaction.
pub async fn append(record: &ExamRecord) -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[EXAMS_STORE, META_STORE], TransactionMode::ReadWrite)?;

    transaction
        .store(EXAMS_STORE)?
        .add(&JsValue::from_str(&to_record(record)?), None)
        .await?;
    write_store_count(&transaction, EXAMS_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(())
}

/// Replaces the most recent Exam's record, keeping the store's count.
///
/// The record and the count commit in one transaction. The caller keeps the
/// original `at`: an overridden record still says when its Exam ended. An
/// empty store has no latest record to replace, so the call aborts its
/// transaction and fails without writing anything.
pub async fn replace_latest(record: &ExamRecord) -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[EXAMS_STORE, META_STORE], TransactionMode::ReadWrite)?;
    let store = transaction.store(EXAMS_STORE)?;
    let keys = store.get_all_keys(None, None).await?;
    let Some(key) = keys.last().cloned() else {
        transaction.abort().await?;

        return Err(Error::custom("the Exam history holds no record to replace"));
    };
    store
        .put(&JsValue::from_str(&to_record(record)?), Some(&key))
        .await?;
    write_store_count(&transaction, EXAMS_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(())
}

/// Removes the most recent Exam's record, decrementing the store's count.
///
/// The removal and the count commit in one transaction. An empty store has
/// nothing to remove, so the call changes nothing.
pub async fn remove_latest() -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[EXAMS_STORE, META_STORE], TransactionMode::ReadWrite)?;
    let store = transaction.store(EXAMS_STORE)?;
    let keys = store.get_all_keys(None, None).await?;

    if let Some(key) = keys.last().cloned() {
        store.delete(key).await?;
        write_store_count(&transaction, EXAMS_STORE, COUNT_KEY).await?;
    }
    transaction.commit().await?;

    Ok(())
}

/// Every Exam's record, oldest first.
pub async fn exams() -> Result<Vec<ExamRecord>> {
    read_all(EXAMS_STORE)
        .await?
        .iter()
        .map(|record| from_record(record))
        .collect()
}

/// Every Exam's record, with an empty read checked against the store's own
/// count, for the same reason as the Answer Log's verified read.
pub async fn exams_verified() -> Result<Vec<ExamRecord>> {
    let read = exams().await?;

    if !read.is_empty() {
        remember_count(read.len()).await?;

        return Ok(read);
    }

    let Some(count) = meta_count(COUNT_KEY).await? else {
        return Ok(read);
    };

    if count == 0 {
        return Ok(read);
    }

    for delay in RETRY_MILLIS {
        TimeoutFuture::new(delay).await;

        let read = exams().await?;

        if !read.is_empty() {
            return Ok(read);
        }
    }

    Err(Error::custom(
        format!(
            "the Exam history could not be read, although this device records {count} Exams. \
             Nothing has been cleared. Reload the app, and export your progress before changing \
             anything."
        )
        .as_str(),
    ))
}

/// The most recent Exam's record, if the Applicant has taken one.
pub async fn latest() -> Result<Option<ExamRecord>> {
    Ok(exams_verified().await?.pop())
}

/// Empties the Exam records, after the Applicant has confirmed the Clear.
///
/// The records and the count go in one transaction.
pub async fn clear() -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[EXAMS_STORE, META_STORE], TransactionMode::ReadWrite)?;

    transaction.store(EXAMS_STORE)?.clear().await?;
    write_store_count(&transaction, EXAMS_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(())
}

/// Backfills the count for Exams stored before the marker existed, and keeps
/// the marker at least as high as the store.
async fn remember_count(count: usize) -> Result<()> {
    let stored = meta_count(COUNT_KEY).await?.unwrap_or(0);
    let count = count as u64;

    if stored < count {
        write_count(COUNT_KEY, count).await?;
    }

    Ok(())
}

/// The record one Exam is stored as.
pub fn to_record(record: &ExamRecord) -> Result<String> {
    Ok(serde_json::to_string(record)?)
}

/// The Exam a stored record holds.
pub fn from_record(record: &str) -> Result<ExamRecord> {
    Ok(serde_json::from_str(record)?)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;
    use domain::{Outcome, Track};

    /// One finished Exam.
    fn record() -> ExamRecord {
        ExamRecord {
            at: 1_770_000_000_000,
            track: Track::SpecialConsideration,
            questions: vec![2, 7, 12],
            correct_count: 2,
            outcome: Outcome::Passed,
        }
    }

    #[test]
    fn test_exam_record_round_trips_an_exam() -> Result<()> {
        // -- Setup & Fixtures
        let record = record();

        // -- Exec
        let read_back = from_record(&to_record(&record)?)?;

        // -- Check
        assert_eq!(read_back, record);

        Ok(())
    }

    #[test]
    fn test_exam_from_record_rejects_another_shape() -> Result<()> {
        // -- Exec
        let result = from_record("{\"track\": \"Standard\"}");

        // -- Check
        assert!(
            result.is_err(),
            "a record that is not an Exam must not pass"
        );

        Ok(())
    }
}

// endregion: --- Tests
