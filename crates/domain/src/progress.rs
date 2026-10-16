//! Cleared state, Coverage, and the Answer Log entries they derive from.
//!
//! A Civics Test Question is Cleared after three correct answers in a row, and
//! a wrong answer makes it unclear again. Coverage is the share of the
//! Applicant's accountable questions that are currently Cleared: the whole list,
//! or the 20 starred questions when the 65/20 Special Consideration applies.
//!
//! This module holds the value the screens show, and the entry the Answer Log
//! is made of. Both Cleared and Coverage are derived from the log at read time,
//! so the log stays the single source of truth and an Override moves them
//! without rewriting anything.

use core::fmt::{Display, Formatter};
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{Profile, QuestionRecords, Verdict};

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

/// How many correct answers in a row Clear a Civics Test Question.
const CLEARED_STREAK: u32 = 3;

/// The Civics Test Questions the Answer Log has Cleared.
///
/// A question is Cleared by three correct answers in a row; a wrong answer makes
/// it unclear again. An Override counts as correct for its attempt, so an
/// overridden answer feeds the streak and never changes the Accepted Answers.
pub fn cleared(log: &[AnswerLogEntry]) -> HashSet<u16> {
    let mut streaks: HashMap<u16, u32> = HashMap::new();
    let mut cleared = HashSet::new();

    for entry in log {
        let streak = streaks.entry(entry.question).or_default();

        if entry.counts_correct() {
            *streak += 1;

            if *streak >= CLEARED_STREAK {
                cleared.insert(entry.question);
            }
        } else {
            *streak = 0;
            cleared.remove(&entry.question);
        }
    }

    cleared
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

    /// Coverage derived from the Answer Log: the share of the Applicant's
    /// accountable questions that are Cleared right now.
    ///
    /// An Exam never appends to the log, so a lucky Exam cannot clear a question
    /// the Applicant has never studied.
    pub fn from_log(records: &QuestionRecords, profile: &Profile, log: &[AnswerLogEntry]) -> Self {
        let accountable = records.accountable(profile);
        let cleared = cleared(log);
        let count = accountable
            .iter()
            .filter(|question| cleared.contains(&question.number))
            .count();

        Self::new(
            u16::try_from(count).unwrap_or(u16::MAX),
            u16::try_from(accountable.len()).unwrap_or(u16::MAX),
        )
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
    use crate::{AcceptedAnswer, Question, Section, Topic};

    /// One answer the Applicant gave to one question.
    fn answered(question: u16, verdict: Verdict, overridden: bool) -> AnswerLogEntry {
        AnswerLogEntry {
            at: 1_770_000_000_000 + u64::from(question),
            question,
            answer: "Mikie Sherrill".to_string(),
            verdict,
            overridden,
        }
    }

    /// Three correct answers in a row, which Clear the question.
    fn cleared_streak(question: u16) -> Vec<AnswerLogEntry> {
        (0..3)
            .map(|_| answered(question, Verdict::Correct, false))
            .collect()
    }

    /// Records of `count` Civics Test Questions, the first `starred` of them
    /// starred.
    fn records(count: u16, starred: u16) -> QuestionRecords {
        QuestionRecords {
            questions: (1..=count)
                .map(|number| Question {
                    number,
                    section: Section::AmericanGovernment,
                    topic: Topic::PrinciplesOfAmericanGovernment,
                    question: "Fixture.".to_string(),
                    accepted_answers: vec![AcceptedAnswer::from("Republic")],
                    answers_required: 1,
                    starred: number <= starred,
                    explanation: None,
                })
                .collect(),
        }
    }

    #[test]
    fn test_progress_cleared_needs_three_correct_answers_in_a_row() {
        // -- Setup & Fixtures
        let two = vec![
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];

        // -- Exec
        let mut three = two.clone();
        three.push(answered(1, Verdict::Correct, false));

        // -- Check
        assert!(
            !cleared(&two).contains(&1),
            "two correct answers in a row are not three"
        );
        assert!(
            cleared(&three).contains(&1),
            "three correct answers in a row Clear the question"
        );
    }

    #[test]
    fn test_progress_cleared_is_lost_on_a_later_wrong_answer() {
        // -- Setup & Fixtures
        let mut log = cleared_streak(1);

        // -- Exec
        log.push(answered(1, Verdict::Wrong, false));

        // -- Check
        assert!(
            !cleared(&log).contains(&1),
            "a wrong answer makes the question unclear again"
        );
        assert!(cleared(&log).is_empty(), "a wrong answer Clears nothing");
    }

    #[test]
    fn test_progress_cleared_counts_the_three_in_a_row_after_a_wrong_answer() {
        // -- Setup & Fixtures
        // The case the GLOSSARY works through: a wrong answer resets the count,
        // and the three in a row after it are the three that Clear.
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Wrong, false),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];

        // -- Exec & Check
        assert!(
            cleared(&log).contains(&1),
            "three in a row after the wrong answer Clear the question"
        );
    }

    #[test]
    fn test_progress_cleared_counts_an_override_only_for_its_own_attempt() {
        // -- Setup & Fixtures
        // The same phrasing, wrong twice: the first attempt overridden, the
        // second left as the matcher judged it.
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Wrong, true),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Wrong, false),
        ];

        // -- Exec & Check
        assert!(
            !cleared(&log).contains(&1),
            "the second wrong answer stands on its own and breaks the streak"
        );
    }

    #[test]
    fn test_progress_cleared_counts_an_override_as_correct() {
        // -- Setup & Fixtures
        let mut log = vec![
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];
        log.push(answered(1, Verdict::Wrong, true));

        // -- Exec & Check
        assert!(
            cleared(&log).contains(&1),
            "an Override counts as correct for its attempt"
        );
    }

    #[test]
    fn test_progress_cleared_reads_each_questions_own_streak() {
        // -- Setup & Fixtures
        // Two questions interleaved, so neither streak is a run of the log.
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(2, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
            answered(2, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];

        // -- Exec & Check
        assert_eq!(
            cleared(&log),
            HashSet::from([1]),
            "question 1 has its three, question 2 has only two"
        );
    }

    #[test]
    fn test_progress_coverage_from_log_counts_the_cleared_accountable_questions() {
        // -- Setup & Fixtures
        let records = records(3, 1);
        let mut log = cleared_streak(1);
        log.extend(cleared_streak(3));

        // -- Exec
        let coverage = Coverage::from_log(&records, &Profile::default(), &log);

        // -- Check
        assert_eq!(coverage.cleared(), 2);
        assert_eq!(coverage.total(), 3);
        assert_eq!(coverage.to_string(), "67% (2 of 3)");
    }

    #[test]
    fn test_progress_coverage_from_log_narrows_with_the_65_20_scope() {
        // -- Setup & Fixtures
        let records = records(3, 1);
        let log = cleared_streak(1);
        let reduced = Profile {
            special_consideration_65_20: true,
            ..Profile::default()
        };

        // -- Exec
        let coverage = Coverage::from_log(&records, &reduced, &log);

        // -- Check
        assert_eq!(
            coverage.to_string(),
            "100% (1 of 1)",
            "under the 65/20 Special Consideration the starred question is the whole scope"
        );
    }

    #[test]
    fn test_progress_coverage_from_log_is_zero_without_answers() {
        // -- Setup & Fixtures
        let full = records(3, 1);

        // -- Exec & Check
        assert_eq!(
            Coverage::from_log(&full, &Profile::default(), &[]).to_string(),
            "0% (0 of 3)"
        );
        assert_eq!(
            Coverage::from_log(&records(0, 0), &Profile::default(), &[]).to_string(),
            "0% (0 of 0)",
            "no questions at all reads as no Coverage, not as a division"
        );
    }

    #[test]
    fn test_progress_answer_log_entry_counts_an_override_as_correct() {
        // -- Setup & Fixtures
        let correct = answered(61, Verdict::Correct, false);
        let wrong = answered(61, Verdict::Wrong, false);
        let overridden = answered(61, Verdict::Wrong, true);

        // -- Exec & Check
        assert!(correct.counts_correct());
        assert!(!wrong.counts_correct());
        assert!(overridden.counts_correct(), "an Override counts as correct");
    }

    #[test]
    fn test_progress_answer_log_entry_round_trips_the_stored_record() -> Result<()> {
        // -- Setup & Fixtures
        let entry = answered(61, Verdict::Wrong, true);

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
