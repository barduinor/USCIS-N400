//! The Answer Log: every answer the Applicant gives in study mode.
//!
//! The store is keyed by its own sequence, so the log keeps the order the
//! answers were given in (ADR 0007). A record is JSON text — the same shape the
//! export bundle will carry — which keeps the conversions pure and testable on
//! the host; only `append`, `override_last`, and `entries` reach the browser.

use domain::{AnswerLogEntry, Verdict};
use gloo_timers::future::TimeoutFuture;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use super::{
    ANSWER_LOG_STORE, META_STORE, database, meta_count, now, read_all, write_count,
    write_store_count,
};
use crate::{Error, Result};

/// The meta key that records how many answers the log held.
pub(super) const COUNT_KEY: &str = "answer_count";

/// How long the read waits before trying the log again, when the meta record
/// says it held answers but the log came back empty.
const RETRY_MILLIS: [u32; 3] = [50, 150, 400];

/// Appends one answer to the Answer Log, and hands back what was stored.
///
/// The record and the log's count commit in one transaction, so the count
/// never claims a record the log does not hold.
pub async fn append(question: u16, answer: &str, verdict: Verdict) -> Result<AnswerLogEntry> {
    let entry = AnswerLogEntry {
        at: now(),
        question,
        answer: answer.to_string(),
        verdict,
        overridden: false,
    };

    let database = database().await?;
    let transaction =
        database.transaction(&[ANSWER_LOG_STORE, META_STORE], TransactionMode::ReadWrite)?;

    transaction
        .store(ANSWER_LOG_STORE)?
        .add(&JsValue::from_str(&to_record(&entry)?), None)
        .await?;
    write_store_count(&transaction, ANSWER_LOG_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(entry)
}

/// Marks the last answer in the log as the Applicant's Override.
///
/// That entry then counts as correct for its attempt, and Cleared and Coverage
/// follow it. The question's Accepted Answers are untouched: the same phrasing
/// is judged wrong again on a later attempt.
pub async fn override_last() -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[ANSWER_LOG_STORE, META_STORE], TransactionMode::ReadWrite)?;
    let store = transaction.store(ANSWER_LOG_STORE)?;
    let keys = store.get_all_keys(None, None).await?;
    let Some(key) = keys.last().cloned() else {
        return Err(Error::custom("the Answer Log holds no answer to override"));
    };
    let Some(value) = store.get(key.clone()).await? else {
        return Err(Error::custom("the last answer could not be read back"));
    };
    let record = value
        .as_string()
        .ok_or_else(|| Error::custom("the stored answer is not text"))?;
    let mut entry = from_record(&record)?;
    entry.overridden = true;

    store
        .put(&JsValue::from_str(&to_record(&entry)?), Some(&key))
        .await?;
    write_store_count(&transaction, ANSWER_LOG_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(())
}

/// Every answer in the log, oldest first.
pub async fn entries() -> Result<Vec<AnswerLogEntry>> {
    read_all(ANSWER_LOG_STORE)
        .await?
        .iter()
        .map(|record| from_record(record))
        .collect()
}

/// Every answer in the log, with an empty read checked against the log's own
/// count.
///
/// An empty store is only believed when the meta record also holds zero. When
/// the meta record says the log held answers, the read is retried before the
/// screen can show `0%`; if it stays empty, the call fails and the screen shows
/// a recovery message instead of a false figure. Nothing is written on this
/// path.
pub async fn entries_verified() -> Result<Vec<AnswerLogEntry>> {
    let read = entries().await?;

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

        let read = entries().await?;

        if !read.is_empty() {
            return Ok(read);
        }
    }

    Err(Error::custom(
        format!(
            "the Answer Log could not be read, although this device records {count} answers. \
             Nothing has been cleared. Reload the app, and export your progress before changing \
             anything."
        )
        .as_str(),
    ))
}

/// Empties the Answer Log, after the Applicant has confirmed the Clear.
///
/// The records and the count go in one transaction, so a screen that reads
/// afterwards sees a log that is empty on purpose, not a failed read.
pub async fn clear() -> Result<()> {
    let database = database().await?;
    let transaction =
        database.transaction(&[ANSWER_LOG_STORE, META_STORE], TransactionMode::ReadWrite)?;

    transaction.store(ANSWER_LOG_STORE)?.clear().await?;
    write_store_count(&transaction, ANSWER_LOG_STORE, COUNT_KEY).await?;
    transaction.commit().await?;

    Ok(())
}

/// Backfills the count for answers stored before the marker existed, and keeps
/// the marker at least as high as the log.
async fn remember_count(count: usize) -> Result<()> {
    let stored = meta_count(COUNT_KEY).await?.unwrap_or(0);
    let count = count as u64;

    if stored < count {
        write_count(COUNT_KEY, count).await?;
    }

    Ok(())
}

/// The record one answer is stored as.
pub fn to_record(entry: &AnswerLogEntry) -> Result<String> {
    Ok(serde_json::to_string(entry)?)
}

/// The answer a stored record holds.
pub fn from_record(record: &str) -> Result<AnswerLogEntry> {
    Ok(serde_json::from_str(record)?)
}
// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    /// One answer the Applicant gave.
    fn entry() -> AnswerLogEntry {
        AnswerLogEntry {
            at: 1_770_000_000_000,
            question: 23,
            answer: "Cory Booker".to_string(),
            verdict: Verdict::Correct,
            overridden: false,
        }
    }

    #[test]
    fn test_answer_log_record_round_trips_an_answer() -> Result<()> {
        // -- Setup & Fixtures
        let entry = entry();

        // -- Exec
        let read_back = from_record(&to_record(&entry)?)?;

        // -- Check
        assert_eq!(read_back, entry);

        Ok(())
    }

    #[test]
    fn test_answer_log_from_record_rejects_another_shape() -> Result<()> {
        // -- Exec
        let result = from_record("{\"question\": 23}");

        // -- Check
        assert!(
            result.is_err(),
            "a record that is not an answer must not pass"
        );

        Ok(())
    }

    /// One record as an earlier build wrote it, with the `overridden` field
    /// absent: what the browser holds when a record shape moves on. The Study
    /// screen carries this text to the Applicant, so it reads as the app's
    /// sentence and never as the parser's complaint.
    #[test]
    fn test_answer_log_unreadable_record_reads_as_the_apps_own_sentence() -> Result<()> {
        // -- Setup & Fixtures
        let record = "{\"at\": 1770000000000, \"question\": 23, \"answer\": \"Cory Booker\", \
                      \"verdict\": \"correct\"}";

        // -- Exec
        let Err(error) = from_record(record) else {
            return Err("a record with no `overridden` must not read as an answer".into());
        };

        // -- Check
        assert_eq!(
            error.to_string(),
            "the storage on this device holds something the app cannot read, and importing a \
             bundle from the Profile screen is the way back."
        );

        Ok(())
    }
}

// endregion: --- Tests
