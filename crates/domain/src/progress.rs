//! Cleared state, Coverage, and the Answer Log entries they derive from.
//!
//! A Civics Test Question is Cleared after three correct answers in a row, and
//! a wrong answer makes it unclear again. Coverage is the share of the
//! Applicant's accountable questions that are currently Cleared: the whole list,
//! or the 20 starred questions when the 65/20 Special Consideration applies.
//!
//! This module holds the value the screens show, and the entry the Answer Log
//! is made of. Deriving Cleared and Coverage from the log belongs with the rest
//! of Cleared.

use core::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{QuestionRecords, Verdict};

/// One answer the Applicant gave in study mode (ADR 0007).
///
/// The log is append-only: an Override marks the entry it corrects, and Cleared
/// and Coverage are derived from the entries at read time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerLogEntry {
    /// When the answer was given, in milliseconds since the Unix epoch, as the
    /// Applicant's device reported them.
    pub at: u64,

    /// The number of the Civics Test Question that was asked.
    pub question: u16,

    /// What the Applicant typed.
    pub answer: String,

    /// What the tool decided about it.
    pub verdict: Verdict,

    /// Whether the Applicant overrode a wrong verdict.
    pub overridden: bool,
}

impl AnswerLogEntry {
    /// Whether this answer counts as correct towards Cleared and Coverage.
    ///
    /// An Override counts as correct for its attempt and never changes the
    /// question's Accepted Answers (GLOSSARY).
    pub fn counts_correct(&self) -> bool {
        self.verdict == Verdict::Correct || self.overridden
    }
}

/// The share of the Applicant's accountable questions that are Cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    cleared: u16,
    total: u16,
}

impl Coverage {
    /// Coverage of `cleared` questions out of `total` accountable ones.
    ///
    /// `cleared` beyond `total` reads as full Coverage.
    pub fn new(cleared: u16, total: u16) -> Self {
        Self { cleared, total }
    }

    /// Nothing Cleared yet, across the committed Civics Test Questions.
    pub fn none(records: &QuestionRecords) -> Self {
        let total = u16::try_from(records.questions.len()).unwrap_or(u16::MAX);

        Self::new(0, total)
    }

    /// How many questions are Cleared.
    pub fn cleared(self) -> u16 {
        self.cleared
    }

    /// How many questions the Applicant is accountable for.
    pub fn total(self) -> u16 {
        self.total
    }

    /// Coverage in whole percent, rounded to the nearest.
    pub fn percentage(self) -> u8 {
        if self.total == 0 {
            return 0;
        }

        let total = u32::from(self.total);
        let percent = (u32::from(self.cleared) * 100 + total / 2) / total;

        u8::try_from(percent).unwrap_or(100)
    }
}

impl Display for Coverage {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}% ({} of {})",
            self.percentage(),
            self.cleared,
            self.total
        )
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    /// One answer the Applicant gave.
    fn answer_entry(verdict: Verdict, overridden: bool) -> AnswerLogEntry {
        AnswerLogEntry {
            at: 1_770_000_000_000,
            question: 61,
            answer: "Mikie Sherrill".to_string(),
            verdict,
            overridden,
        }
    }

    #[test]
    fn test_progress_coverage_none_is_zero_percent() {
        // -- Setup & Fixtures
        let records = QuestionRecords {
            questions: Vec::new(),
        };

        // -- Exec
        let coverage = Coverage::none(&records);

        // -- Check
        assert_eq!(coverage.cleared(), 0);
        assert_eq!(coverage.total(), 0);
        assert_eq!(coverage.percentage(), 0);
        assert_eq!(coverage.to_string(), "0% (0 of 0)");
    }

    #[test]
    fn test_progress_answer_log_entry_counts_an_override_as_correct() {
        // -- Setup & Fixtures
        let correct = answer_entry(Verdict::Correct, false);
        let wrong = answer_entry(Verdict::Wrong, false);
        let overridden = answer_entry(Verdict::Wrong, true);

        // -- Exec & Check
        assert!(correct.counts_correct());
        assert!(!wrong.counts_correct());
        assert!(overridden.counts_correct(), "an Override counts as correct");
    }

    #[test]
    fn test_progress_answer_log_entry_round_trips_the_stored_record() -> Result<()> {
        // -- Setup & Fixtures
        let entry = answer_entry(Verdict::Wrong, true);

        // -- Exec
        let record = serde_json::to_string(&entry)?;
        let stored: serde_json::Value = serde_json::from_str(&record)?;

        // -- Check
        for field in ["at", "question", "answer", "verdict", "overridden"] {
            assert!(stored.get(field).is_some(), "a stored answer holds {field}");
        }
        assert_eq!(stored["verdict"], "wrong");
        assert_eq!(serde_json::from_str::<AnswerLogEntry>(&record)?, entry);

        Ok(())
    }

    #[test]
    fn test_progress_coverage_percentage_rounds_to_the_nearest() {
        // -- Setup & Fixtures
        let cases = [
            (0, 128, 0),
            (1, 128, 1),
            (1, 8, 13),
            (1, 3, 33),
            (2, 3, 67),
            (3, 4, 75),
            (127, 128, 99),
            (128, 128, 100),
        ];

        // -- Exec & Check
        for (cleared, total, expected) in cases {
            let coverage = Coverage::new(cleared, total);

            assert_eq!(
                coverage.percentage(),
                expected,
                "{cleared} of {total} should read as {expected}%"
            );
        }
    }

    #[test]
    fn test_progress_coverage_shows_a_percentage_and_a_count() {
        // -- Setup & Fixtures
        let coverage = Coverage::new(54, 128);

        // -- Exec & Check
        assert_eq!(coverage.to_string(), "42% (54 of 128)");
    }
}

// endregion: --- Tests
