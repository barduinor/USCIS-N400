//! Cleared state, Coverage, and the Answer Log entries they derive from.
//!
//! A Civics Test Question is Cleared by its first correct answer, unless it has
//! been answered wrong before, in which case it needs three correct answers in a
//! row. A later wrong answer makes it unclear again. Coverage is the share of the
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

use crate::{Profile, Question, QuestionRecords, Section, Topic, Verdict};

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

/// How many correct answers in a row Clear a question that was answered wrong
/// before.
const CLEARED_STREAK: u32 = 3;

/// What the Answer Log says about one Civics Test Question.
#[derive(Debug, Default)]
struct QuestionState {
    /// Whether any answer to the question was wrong without an Override.
    wrong_before: bool,

    /// How many correct answers in a row the question has since its last wrong
    /// answer. A wrong answer resets it.
    correct_streak: u32,
}

impl QuestionState {
    /// Records one answer to the question.
    fn record(&mut self, counts_correct: bool) {
        if counts_correct {
            self.correct_streak += 1;
        } else {
            self.wrong_before = true;
            self.correct_streak = 0;
        }
    }

    /// Whether the question is Cleared now.
    fn is_cleared(&self) -> bool {
        self.correct_streak >= CLEARED_STREAK || (self.correct_streak > 0 && !self.wrong_before)
    }
}

/// The Civics Test Questions the Answer Log has Cleared.
///
/// A question is Cleared by its first correct answer. Once it has been answered
/// wrong, it needs three correct answers in a row after that wrong answer; a
/// later wrong answer makes it unclear again. An Override counts as correct for
/// its attempt, so an overridden answer feeds the streak and never changes the
/// Accepted Answers.
pub fn cleared(log: &[AnswerLogEntry]) -> HashSet<u16> {
    let mut states: HashMap<u16, QuestionState> = HashMap::new();

    for entry in log {
        states
            .entry(entry.question)
            .or_default()
            .record(entry.counts_correct());
    }

    states
        .into_iter()
        .filter_map(|(question, state)| state.is_cleared().then_some(question))
        .collect()
}

/// The whole percent `count` is of `total`, rounded to the nearest. Nothing of
/// nothing reads as zero.
pub(crate) fn percentage(count: u16, total: u16) -> u8 {
    if total == 0 {
        return 0;
    }

    let total = u32::from(total);
    let percent = (u32::from(count) * 100 + total / 2) / total;

    u8::try_from(percent).unwrap_or(100)
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
        Self::among(records, profile, log, |_| true)
    }

    /// Coverage of the Applicant's accountable questions in one Section.
    pub fn of_section(
        records: &QuestionRecords,
        profile: &Profile,
        log: &[AnswerLogEntry],
        section: Section,
    ) -> Self {
        Self::among(records, profile, log, |question| {
            question.section == section
        })
    }

    /// Coverage of the Applicant's accountable questions in one Topic.
    pub fn of_topic(
        records: &QuestionRecords,
        profile: &Profile,
        log: &[AnswerLogEntry],
        topic: Topic,
    ) -> Self {
        Self::among(records, profile, log, |question| question.topic == topic)
    }

    /// Coverage of the accountable questions a filter picks out, so that the
    /// Sections and their Topics partition the Applicant's scope exactly.
    fn among(
        records: &QuestionRecords,
        profile: &Profile,
        log: &[AnswerLogEntry],
        keep: impl Fn(&Question) -> bool,
    ) -> Self {
        let accountable = records.accountable(profile);
        let cleared = cleared(log);
        let scope: Vec<&Question> = accountable
            .into_iter()
            .filter(|question| keep(question))
            .collect();
        let count = scope
            .iter()
            .filter(|question| cleared.contains(&question.number))
            .count();

        Self::new(
            u16::try_from(count).unwrap_or(u16::MAX),
            u16::try_from(scope.len()).unwrap_or(u16::MAX),
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
        percentage(self.cleared, self.total)
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

    /// Three correct answers in a row for one question.
    fn cleared_streak(question: u16) -> Vec<AnswerLogEntry> {
        (0..3)
            .map(|_| answered(question, Verdict::Correct, false))
            .collect()
    }

    /// One answer to question 1 per Verdict, in the order given.
    fn answers(verdicts: &[Verdict]) -> Vec<AnswerLogEntry> {
        verdicts
            .iter()
            .map(|verdict| answered(1, *verdict, false))
            .collect()
    }

    /// Whether the answers in order Clear question 1.
    fn is_cleared(verdicts: &[Verdict]) -> bool {
        cleared(&answers(verdicts)).contains(&1)
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
            alternatives: Vec::new(),
        }
    }

    #[test]
    fn test_progress_cleared_with_no_answers_is_not_cleared() {
        // -- Exec & Check
        assert!(
            !is_cleared(&[]),
            "a question with no answers is not Cleared"
        );
        assert!(cleared(&[]).is_empty(), "an empty log Clears nothing");
    }

    #[test]
    fn test_progress_cleared_on_the_first_correct_answer_when_never_wrong() {
        // -- Exec & Check
        assert!(
            is_cleared(&[Verdict::Correct]),
            "a question never answered wrong is Cleared by one correct answer"
        );
    }

    #[test]
    fn test_progress_cleared_stays_after_a_second_correct_answer() {
        // -- Exec & Check
        assert!(
            is_cleared(&[Verdict::Correct, Verdict::Correct]),
            "a Cleared question stays Cleared through another correct answer"
        );
    }

    #[test]
    fn test_progress_cleared_wrong_answer_alone_is_not_cleared() {
        // -- Exec & Check
        assert!(
            !is_cleared(&[Verdict::Wrong]),
            "a wrong answer does not Clear the question"
        );
    }

    #[test]
    fn test_progress_cleared_one_correct_answer_after_a_wrong_one_is_not_enough() {
        // -- Exec & Check
        assert!(
            !is_cleared(&[Verdict::Wrong, Verdict::Correct]),
            "the streak since the wrong answer is one"
        );
    }

    #[test]
    fn test_progress_cleared_three_in_a_row_after_a_wrong_answer_clears() {
        // -- Exec & Check
        assert!(
            is_cleared(&[
                Verdict::Wrong,
                Verdict::Correct,
                Verdict::Correct,
                Verdict::Correct,
            ]),
            "three in a row after the wrong answer Clear the question"
        );
    }

    #[test]
    fn test_progress_cleared_correct_then_wrong_makes_it_unclear() {
        // -- Exec & Check
        assert!(
            !is_cleared(&[Verdict::Correct, Verdict::Wrong]),
            "a wrong answer makes a Cleared question unclear again"
        );
    }

    #[test]
    fn test_progress_cleared_two_in_a_row_after_correct_then_wrong_is_not_enough() {
        // -- Exec & Check
        assert!(
            !is_cleared(&[
                Verdict::Correct,
                Verdict::Wrong,
                Verdict::Correct,
                Verdict::Correct,
            ]),
            "after the wrong answer the question owes three in a row, and has two"
        );
    }

    #[test]
    fn test_progress_cleared_three_in_a_row_after_correct_then_wrong_clears() {
        // -- Exec & Check
        assert!(
            is_cleared(&[
                Verdict::Correct,
                Verdict::Wrong,
                Verdict::Correct,
                Verdict::Correct,
                Verdict::Correct,
            ]),
            "three correct answers after the wrong answer Clear the question"
        );
    }

    #[test]
    fn test_progress_cleared_counts_an_override_as_a_first_correct_answer() {
        // -- Setup & Fixtures
        let log = vec![answered(1, Verdict::Wrong, true)];

        // -- Exec & Check
        assert!(
            cleared(&log).contains(&1),
            "an Override counts as correct for its attempt, so it is the first correct answer"
        );
    }

    #[test]
    fn test_progress_cleared_counts_an_override_in_the_streak_after_a_wrong_answer() {
        // -- Setup & Fixtures
        // The first attempt stands as wrong; the second, overridden, is correct.
        let log = vec![
            answered(1, Verdict::Wrong, false),
            answered(1, Verdict::Wrong, true),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];

        // -- Exec & Check
        assert!(
            cleared(&log).contains(&1),
            "the overridden answer is the first of the three in a row"
        );
    }

    #[test]
    fn test_progress_cleared_counts_an_override_only_for_its_own_attempt() {
        // -- Setup & Fixtures
        // A later wrong answer is not covered by the earlier Override.
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
            "a later wrong answer makes the question unclear again"
        );
    }

    #[test]
    fn test_progress_cleared_reads_each_questions_own_answers() {
        // -- Setup & Fixtures
        // Two questions interleaved: question 1 is Cleared by its first correct
        // answer and then made unclear, while question 2 has its own answer.
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(2, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Wrong, false),
        ];

        // -- Exec & Check
        assert_eq!(
            cleared(&log),
            HashSet::from([2]),
            "question 2 is Cleared and question 1 is not"
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
    fn test_progress_coverage_from_log_counts_questions_cleared_by_their_first_correct_answer() {
        // -- Setup & Fixtures
        let records = records(128, 0);
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(2, Verdict::Correct, false),
            answered(3, Verdict::Correct, false),
        ];

        // -- Exec
        let coverage = Coverage::from_log(&records, &Profile::default(), &log);

        // -- Check
        assert_eq!(
            coverage.to_string(),
            "2% (3 of 128)",
            "three questions answered correctly once each are three Cleared questions"
        );
    }

    #[test]
    fn test_progress_coverage_from_log_drops_a_question_answered_wrong() {
        // -- Setup & Fixtures
        let records = records(128, 0);
        let log = vec![
            answered(1, Verdict::Correct, false),
            answered(2, Verdict::Correct, false),
            answered(3, Verdict::Correct, false),
            answered(1, Verdict::Wrong, false),
        ];

        // -- Exec
        let coverage = Coverage::from_log(&records, &Profile::default(), &log);

        // -- Check
        assert_eq!(
            coverage.to_string(),
            "2% (2 of 128)",
            "a wrong answer stops the question counting until three in a row Clear it"
        );
    }

    #[test]
    fn test_progress_coverage_from_log_waits_for_three_in_a_row_after_a_wrong_answer() {
        // -- Setup & Fixtures
        let records = records(128, 0);
        let mut log = vec![
            answered(1, Verdict::Wrong, false),
            answered(1, Verdict::Correct, false),
            answered(1, Verdict::Correct, false),
        ];

        // -- Exec & Check
        assert_eq!(
            Coverage::from_log(&records, &Profile::default(), &log).to_string(),
            "0% (0 of 128)",
            "two correct answers after a wrong one are not three"
        );

        log.push(answered(1, Verdict::Correct, false));

        assert_eq!(
            Coverage::from_log(&records, &Profile::default(), &log).to_string(),
            "1% (1 of 128)",
            "the third correct answer in a row Clears the question"
        );
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
    fn test_progress_coverage_of_a_section_counts_only_its_own_questions() {
        // -- Setup & Fixtures
        let records = records(2, 0);
        let log = cleared_streak(1);

        // -- Exec & Check
        assert_eq!(
            Coverage::of_section(
                &records,
                &Profile::default(),
                &log,
                Section::AmericanGovernment
            )
            .to_string(),
            "50% (1 of 2)"
        );
        assert_eq!(
            Coverage::of_section(
                &records,
                &Profile::default(),
                &log,
                Section::AmericanHistory
            )
            .to_string(),
            "0% (0 of 0)",
            "a Section with no accountable questions has no Coverage to report"
        );
    }

    #[test]
    fn test_progress_coverage_of_a_topic_counts_only_its_own_questions() {
        // -- Setup & Fixtures
        let records = records(2, 0);
        let log = cleared_streak(1);

        // -- Exec & Check
        assert_eq!(
            Coverage::of_topic(
                &records,
                &Profile::default(),
                &log,
                Topic::PrinciplesOfAmericanGovernment
            )
            .to_string(),
            "50% (1 of 2)"
        );
        assert_eq!(
            Coverage::of_topic(&records, &Profile::default(), &log, Topic::Symbols).to_string(),
            "0% (0 of 0)"
        );
    }

    #[test]
    fn test_progress_coverage_of_a_section_narrows_with_the_65_20_scope() {
        // -- Setup & Fixtures
        let records = records(3, 1);
        let log = cleared_streak(1);
        let reduced = Profile {
            special_consideration_65_20: true,
            ..Profile::default()
        };

        // -- Exec & Check
        assert_eq!(
            Coverage::of_section(&records, &reduced, &log, Section::AmericanGovernment).to_string(),
            "100% (1 of 1)",
            "only the starred question counts under the 65/20 Special Consideration"
        );
        assert_eq!(
            Coverage::of_section(
                &records,
                &Profile::default(),
                &log,
                Section::AmericanGovernment
            )
            .to_string(),
            "33% (1 of 3)"
        );
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
