//! The Answer Log: every answer the Applicant gives in study mode.
//!
//! The store is keyed by its own sequence, so the log keeps the order the
//! answers were given in (ADR 0007). A record is JSON text — the same shape the
//! export bundle will carry — which keeps the conversions pure and testable on
//! the host; only `append` and `entries` reach the browser.

use domain::{AnswerLogEntry, Verdict};

use super::{ANSWER_LOG_STORE, read_all, write_next};
use crate::Result;

/// Appends one answer to the Answer Log, and hands back what was stored.
pub async fn append(question: u16, answer: &str, verdict: Verdict) -> Result<AnswerLogEntry> {
    let entry = AnswerLogEntry {
        at: now(),
        question,
        answer: answer.to_string(),
        verdict,
        overridden: false,
    };

    write_next(ANSWER_LOG_STORE, &to_record(&entry)?).await?;

    Ok(entry)
}

/// Every answer in the log, oldest first.
pub async fn entries() -> Result<Vec<AnswerLogEntry>> {
    read_all(ANSWER_LOG_STORE)
        .await?
        .iter()
        .map(|record| from_record(record))
        .collect()
}

/// The record one answer is stored as.
pub fn to_record(entry: &AnswerLogEntry) -> Result<String> {
    Ok(serde_json::to_string(entry)?)
}

/// The answer a stored record holds.
pub fn from_record(record: &str) -> Result<AnswerLogEntry> {
    Ok(serde_json::from_str(record)?)
}

// region:    --- Support

/// The device's clock, in milliseconds since the Unix epoch.
fn now() -> u64 {
    js_sys::Date::now() as u64
}

// endregion: --- Support

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
}

// endregion: --- Tests
