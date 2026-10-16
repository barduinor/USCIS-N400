//! The Exam records: what each Exam came to.
//!
//! A record is JSON text, the same shape the export bundle will carry — the
//! track it ran under, the questions asked, the score, and the outcome — which
//! keeps the conversions pure and testable on the host. Only `append` and
//! `latest` reach the browser.

use domain::ExamRecord;

use super::{EXAMS_STORE, read_all, write_next};
use crate::Result;

/// Appends one Exam's record.
pub async fn append(record: &ExamRecord) -> Result<()> {
    write_next(EXAMS_STORE, &to_record(record)?).await
}

/// The most recent Exam's record, if the Applicant has sat one.
pub async fn latest() -> Result<Option<ExamRecord>> {
    read_all(EXAMS_STORE)
        .await?
        .last()
        .map(|record| from_record(record))
        .transpose()
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
