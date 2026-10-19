//! The Exam: a scoring simulation of the Civics Test.
//!
//! Without the 65/20 Special Consideration, twenty questions are drawn at random
//! from the Applicant's accountable questions, and the Exam ends the moment the
//! Applicant has answered twelve correctly or nine incorrectly. Under the 65/20
//! Special Consideration, ten questions are drawn from the twenty starred ones,
//! and it ends at six correctly or five incorrectly. Either way it stops exactly
//! as the officer stops the real test: no more questions than the outcome needs.
//!
//! An Exam keeps its own record — outcome, score, track, and the questions asked
//! — and never touches the Answer Log, so a lucky Exam cannot Clear a question
//! the Applicant has never studied.

use core::fmt::{Display, Formatter};
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::progress::percentage;
use crate::{Profile, Question, Verdict};

/// The administration an Exam runs under.
///
/// The track travels in the Exam's record (ADR 0007), so a later Profile change
/// never makes an old Exam unreadable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Track {
    #[serde(rename = "Standard")]
    Standard,

    #[serde(rename = "65/20")]
    SpecialConsideration,
}

/// What an Exam came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Passed,
    Failed,
}

impl Track {
    /// The track the Profile puts the Applicant in.
    pub fn of(profile: &Profile) -> Self {
        if profile.special_consideration_65_20 {
            Track::SpecialConsideration
        } else {
            Track::Standard
        }
    }

    /// How many questions are drawn.
    pub fn questions(self) -> usize {
        match self {
            Track::Standard => 20,
            Track::SpecialConsideration => 10,
        }
    }

    /// How many correct answers pass.
    pub fn pass_mark(self) -> usize {
        match self {
            Track::Standard => 12,
            Track::SpecialConsideration => 6,
        }
    }

    /// How many incorrect answers fail.
    pub fn fail_mark(self) -> usize {
        match self {
            Track::Standard => 9,
            Track::SpecialConsideration => 5,
        }
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Track::Standard => "Standard",
            Track::SpecialConsideration => "65/20 Special Consideration",
        })
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Outcome::Passed => "Passed",
            Outcome::Failed => "Failed",
        })
    }
}

/// One Exam, in progress or finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exam {
    track: Track,
    draw: VecDeque<u16>,
    asked: Vec<(u16, Verdict)>,
}

/// One Exam's record (ADR 0007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExamRecord {
    /// When the Exam ended, in milliseconds since the Unix epoch.
    pub at: u64,

    pub track: Track,

    /// The questions asked, in the order they were asked.
    pub questions: Vec<u16>,

    /// How many of them were answered correctly.
    pub correct_count: u16,

    pub outcome: Outcome,
}

impl ExamRecord {
    /// The share of the questions asked that were answered correctly, in whole
    /// percent.
    pub fn correct_percentage(&self) -> u8 {
        percentage(
            self.correct_count,
            u16::try_from(self.questions.len()).unwrap_or(u16::MAX),
        )
    }
}

impl Exam {
    /// Starts an Exam: the track's questions, drawn at random from the
    /// candidates with the seed the caller brings.
    pub fn start(candidates: &[&Question], track: Track, seed: u64) -> Self {
        Self {
            track,
            draw: drawn(candidates, track.questions(), seed),
            asked: Vec::new(),
        }
    }

    /// The question the officer is asking, or `None` once the Exam is over.
    pub fn current(&self) -> Option<u16> {
        if self.outcome().is_some() {
            return None;
        }

        self.draw.front().copied()
    }

    /// Records the verdict on the question being asked.
    ///
    /// Once the outcome is decided nothing more is recorded: the officer has
    /// stopped, and the Exam's score is the one it ended on.
    pub fn answer(&mut self, verdict: Verdict) {
        if self.outcome().is_some() {
            return;
        }

        let Some(number) = self.draw.pop_front() else {
            return;
        };

        self.asked.push((number, verdict));
    }

    /// Marks the last answer as correct, for the Applicant's Override.
    ///
    /// Reports whether a verdict changed. An Exam with no answers, or with a
    /// correct last answer, is left as it is. The Override never re-judges the
    /// answer and never changes the draw, so an Exam it un-ends resumes at the
    /// question already drawn.
    pub fn override_last(&mut self) -> bool {
        let Some((_, verdict)) = self.asked.last_mut() else {
            return false;
        };

        if *verdict == Verdict::Correct {
            return false;
        }

        *verdict = Verdict::Correct;

        true
    }

    /// The outcome, once it is decided.
    pub fn outcome(&self) -> Option<Outcome> {
        let correct = self.correct();

        if correct >= self.track.pass_mark() {
            Some(Outcome::Passed)
        } else if self.asked.len() - correct >= self.track.fail_mark() {
            Some(Outcome::Failed)
        } else {
            None
        }
    }

    /// How many questions have been answered correctly.
    pub fn correct(&self) -> usize {
        self.asked
            .iter()
            .filter(|(_, verdict)| *verdict == Verdict::Correct)
            .count()
    }

    /// How many questions have been answered wrong.
    pub fn wrong(&self) -> usize {
        self.asked.len() - self.correct()
    }

    /// The administration this Exam runs under.
    pub fn track(&self) -> Track {
        self.track
    }

    /// The questions asked so far, in the order the officer asked them.
    pub fn asked(&self) -> Vec<u16> {
        self.asked.iter().map(|(number, _)| *number).collect()
    }

    /// The record this Exam leaves behind, once its outcome is decided.
    pub fn record(&self, at: u64) -> Option<ExamRecord> {
        Some(ExamRecord {
            at,
            track: self.track,
            questions: self.asked(),
            correct_count: u16::try_from(self.correct()).unwrap_or(u16::MAX),
            outcome: self.outcome()?,
        })
    }
}

/// The questions drawn for one Exam, in the order the officer will ask them.
///
/// The draw is random and the seed is the caller's, so an Exam can be
/// reproduced exactly. Nothing here faces an adversary: it has to be fair, not
/// unpredictable.
fn drawn(candidates: &[&Question], count: usize, seed: u64) -> VecDeque<u16> {
    let mut numbers: Vec<u16> = candidates.iter().map(|question| question.number).collect();
    let mut state = seed | 1;

    for index in (1..numbers.len()).rev() {
        state = shuffled(state);
        numbers.swap(index, (state % (index as u64 + 1)) as usize);
    }

    numbers.truncate(count.min(numbers.len()));

    numbers.into()
}

/// The next value of a xorshift64 sequence.
fn shuffled(state: u64) -> u64 {
    let mut state = state;

    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;

    state
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;
    use crate::{AcceptedAnswer, Section, Topic};

    /// A pool of `count` Civics Test Questions, numbered from one.
    fn candidates(count: u16) -> Vec<Question> {
        (1..=count)
            .map(|number| Question {
                number,
                section: Section::AmericanGovernment,
                topic: Topic::PrinciplesOfAmericanGovernment,
                question: "Fixture.".to_string(),
                accepted_answers: vec![AcceptedAnswer::from("Republic")],
                answers_required: 1,
                starred: number % 3 == 0,
                explanation: None,
            })
            .collect()
    }

    /// Answers the Exam with the verdicts given, and reports how many questions
    /// it took.
    fn answer_with(exam: &mut Exam, verdicts: &[Verdict]) -> usize {
        for verdict in verdicts {
            if exam.current().is_none() {
                break;
            }

            exam.answer(*verdict);
        }

        exam.record(0)
            .map(|record| record.questions.len())
            .unwrap_or(0)
    }

    /// An Exam over `count` candidates on the standard track.
    fn exam(count: u16) -> Exam {
        let pool = candidates(count);

        Exam::start(&pool.iter().collect::<Vec<&Question>>(), Track::Standard, 7)
    }

    #[test]
    fn test_exam_standard_passes_at_twelve_correct() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        let corrects = vec![Verdict::Correct; 12];

        // -- Exec
        let asked = answer_with(&mut exam, &corrects);

        // -- Check
        assert_eq!(exam.outcome(), Some(Outcome::Passed));
        assert_eq!(asked, 12, "the Exam stops the moment twelve are correct");
        assert_eq!(exam.current(), None, "and asks nothing further");
    }

    #[test]
    fn test_exam_counts_the_current_exams_right_and_wrong_answers() {
        // -- Setup & Fixtures
        let mut exam = exam(128);

        // -- Exec
        answer_with(
            &mut exam,
            &[Verdict::Correct, Verdict::Wrong, Verdict::Correct],
        );

        // -- Check
        assert_eq!(exam.asked().len(), 3);
        assert_eq!(exam.correct(), 2);
        assert_eq!(exam.wrong(), 1);
        assert_eq!(exam.track(), Track::Standard);
    }

    #[test]
    fn test_exam_standard_fails_at_nine_incorrect() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        let wrongs = vec![Verdict::Wrong; 9];

        // -- Exec
        let asked = answer_with(&mut exam, &wrongs);

        // -- Check
        assert_eq!(exam.outcome(), Some(Outcome::Failed));
        assert_eq!(asked, 9, "the Exam stops the moment nine are wrong");
    }

    #[test]
    fn test_exam_standard_never_exceeds_twenty_questions() {
        // -- Setup & Fixtures
        // Eight wrong first: the twentieth question is the one that can pass.
        let mut eight_wrong = exam(128);
        let mut verdicts = vec![Verdict::Wrong; 8];
        verdicts.extend(vec![Verdict::Correct; 12]);

        // -- Exec
        let asked = answer_with(&mut eight_wrong, &verdicts);

        // -- Check
        assert_eq!(asked, 20);
        assert_eq!(eight_wrong.outcome(), Some(Outcome::Passed));

        // -- Setup & Fixtures
        // Eleven correct then nine wrong: the twentieth question decides it.
        let mut eleven_correct = exam(128);
        let mut verdicts = vec![Verdict::Correct; 11];
        verdicts.extend(vec![Verdict::Wrong; 9]);

        // -- Exec
        let asked = answer_with(&mut eleven_correct, &verdicts);

        // -- Check
        assert_eq!(asked, 20);
        assert_eq!(eleven_correct.outcome(), Some(Outcome::Failed));
    }

    #[test]
    fn test_exam_special_consideration_ends_at_six_correct_or_five_incorrect() {
        // -- Setup & Fixtures
        let pool = candidates(128);
        let starred: Vec<&Question> = pool.iter().filter(|question| question.starred).collect();

        // -- Exec
        let mut passed = Exam::start(&starred, Track::SpecialConsideration, 7);
        let asked = answer_with(&mut passed, &[Verdict::Correct; 6]);

        // -- Check
        assert_eq!(passed.outcome(), Some(Outcome::Passed));
        assert_eq!(asked, 6);

        // -- Exec
        let mut failed = Exam::start(&starred, Track::SpecialConsideration, 7);
        let asked = answer_with(&mut failed, &[Verdict::Wrong; 5]);

        // -- Check
        assert_eq!(failed.outcome(), Some(Outcome::Failed));
        assert_eq!(asked, 5);
    }

    #[test]
    fn test_exam_draws_the_tracks_questions_from_the_candidates() {
        // -- Setup & Fixtures
        let pool = candidates(128);
        let refs: Vec<&Question> = pool.iter().collect();

        // -- Exec
        let exam = Exam::start(&refs, Track::Standard, 7);
        let record = exam.record(0);

        // -- Check
        assert_eq!(record, None, "an undecided Exam leaves no record yet");
        // The drawn questions are the pool's, and no question comes twice.
        let mut asked = Vec::new();
        let mut exam = exam;
        while let Some(number) = exam.current() {
            asked.push(number);
            exam.answer(Verdict::Wrong);
        }

        assert!(asked.len() >= 9, "the draw carries enough questions");
        assert!(
            asked
                .iter()
                .all(|number| pool.iter().any(|question| question.number == *number)),
            "every drawn question is one of the candidates"
        );
        let distinct: std::collections::HashSet<u16> = asked.iter().copied().collect();
        assert_eq!(distinct.len(), asked.len(), "no question is asked twice");
        assert!(
            asked.iter().all(|number| *number <= 128),
            "the draw stays in the pool"
        );
    }

    #[test]
    fn test_exam_draw_follows_the_caller_seed() {
        // -- Setup & Fixtures
        let pool = candidates(128);
        let refs: Vec<&Question> = pool.iter().collect();

        // -- Exec
        let first = Exam::start(&refs, Track::Standard, 7);
        let again = Exam::start(&refs, Track::Standard, 7);

        // -- Check
        assert_eq!(first, again, "the same seed draws the same Exam");
        assert_ne!(
            first,
            Exam::start(&refs, Track::Standard, 8),
            "a different seed draws another"
        );
    }

    #[test]
    fn test_exam_record_carries_the_track_the_questions_and_the_score() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(&mut exam, &[Verdict::Correct, Verdict::Wrong]);

        // -- Exec
        let record = exam.record(1_770_000_000_000);

        // -- Check
        assert!(record.is_none(), "an Exam still running leaves no record");

        // -- Exec
        answer_with(&mut exam, &[Verdict::Correct; 11]);
        let record = exam.record(1_770_000_000_000).expect("a decided Exam");

        // -- Check
        assert_eq!(record.track, Track::Standard);
        assert_eq!(record.outcome, Outcome::Passed);
        assert_eq!(record.correct_count, 12);
        assert_eq!(
            record.questions.len(),
            13,
            "twelve correct out of thirteen asked"
        );
        assert_eq!(record.at, 1_770_000_000_000);
    }

    #[test]
    fn test_exam_record_round_trips_the_stored_record() -> Result<()> {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(&mut exam, &[Verdict::Correct; 12]);
        let record = exam.record(1_770_000_000_000).expect("a decided Exam");

        // -- Exec
        let stored = serde_json::to_string(&record)?;
        let fields: serde_json::Value = serde_json::from_str(&stored)?;

        // -- Check
        for field in ["at", "track", "questions", "correct_count", "outcome"] {
            assert!(fields.get(field).is_some(), "a stored Exam holds {field}");
        }
        assert_eq!(fields["track"], "Standard");
        assert_eq!(fields["outcome"], "passed");
        assert_eq!(serde_json::from_str::<ExamRecord>(&stored)?, record);

        Ok(())
    }

    #[test]
    fn test_exam_override_last_with_no_answers_changes_nothing() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        let current = exam.current();

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(
            !overridden,
            "an Exam with no answers has nothing to override"
        );
        assert_eq!(exam.current(), current, "the draw is untouched");
        assert_eq!(exam.correct(), 0);
        assert_eq!(exam.wrong(), 0);
        assert_eq!(exam.outcome(), None);
        assert_eq!(exam.record(0), None);
    }

    #[test]
    fn test_exam_override_last_turns_the_last_wrong_answer_correct() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(
            &mut exam,
            &[Verdict::Correct, Verdict::Wrong, Verdict::Wrong],
        );
        let asked = exam.asked();
        let current = exam.current();

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(overridden);
        assert_eq!(exam.correct(), 2);
        assert_eq!(exam.wrong(), 1, "only the last verdict changes");
        assert_eq!(exam.asked(), asked, "the Override changes no question");
        assert_eq!(exam.current(), current, "nor the next question");
        assert_eq!(exam.outcome(), None);
    }

    #[test]
    fn test_exam_override_last_is_a_no_op_when_the_last_answer_is_correct() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(&mut exam, &[Verdict::Wrong, Verdict::Correct]);

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(!overridden);
        assert_eq!(exam.correct(), 1);
        assert_eq!(exam.wrong(), 1);
    }

    #[test]
    fn test_exam_override_last_leaves_a_passed_exam_passed() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(&mut exam, &[Verdict::Correct; 12]);
        let record = exam.record(1_770_000_000_000).expect("a decided Exam");

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(!overridden, "the last verdict is already correct");
        assert_eq!(exam.outcome(), Some(Outcome::Passed));
        assert_eq!(exam.record(1_770_000_000_000), Some(record));
    }

    #[test]
    fn test_exam_override_last_un_ends_a_standard_failed_exam() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        let asked = answer_with(&mut exam, &[Verdict::Wrong; 9]);
        assert_eq!(asked, 9);
        assert_eq!(exam.outcome(), Some(Outcome::Failed));
        assert_eq!(exam.current(), None);

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(overridden);
        assert_eq!(exam.correct(), 1);
        assert_eq!(exam.wrong(), 8, "the fail condition is gone");
        assert_eq!(exam.outcome(), None);
        assert_eq!(exam.record(0), None, "a resumed Exam leaves no record");
        assert_eq!(exam.asked().len(), 9, "the score keeps its questions");
        let next = exam
            .current()
            .expect("the resumed Exam asks the next drawn question");
        assert!(!exam.asked().contains(&next), "the next question is new");
    }

    #[test]
    fn test_exam_override_last_turns_a_standard_failed_exam_into_a_passed_one() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        let mut verdicts = vec![Verdict::Correct; 11];
        verdicts.extend(vec![Verdict::Wrong; 9]);
        let asked = answer_with(&mut exam, &verdicts);
        assert_eq!(asked, 20);
        assert_eq!(exam.outcome(), Some(Outcome::Failed));

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(overridden);
        assert_eq!(exam.correct(), 12);
        assert_eq!(exam.wrong(), 8);
        assert_eq!(exam.outcome(), Some(Outcome::Passed));
        assert_eq!(exam.current(), None, "the Exam stays ended");

        let record = exam.record(1_770_000_000_000).expect("a decided Exam");
        assert_eq!(record.outcome, Outcome::Passed);
        assert_eq!(record.correct_count, 12);
        assert_eq!(record.questions.len(), 20);
        assert_eq!(record.at, 1_770_000_000_000);
    }

    #[test]
    fn test_exam_override_last_un_ends_a_special_consideration_failed_exam() {
        // -- Setup & Fixtures
        let pool = candidates(128);
        let starred: Vec<&Question> = pool.iter().filter(|question| question.starred).collect();
        let mut exam = Exam::start(&starred, Track::SpecialConsideration, 7);
        let asked = answer_with(&mut exam, &[Verdict::Wrong; 5]);
        assert_eq!(asked, 5);
        assert_eq!(exam.outcome(), Some(Outcome::Failed));

        // -- Exec
        let overridden = exam.override_last();

        // -- Check
        assert!(overridden);
        assert_eq!(exam.correct(), 1);
        assert_eq!(exam.wrong(), 4);
        assert_eq!(exam.outcome(), None);
        assert_eq!(exam.record(0), None);
        assert!(
            exam.current().is_some(),
            "the next starred question is asked"
        );

        // The resumed Exam can still fail on the next wrong answer.
        let asked = answer_with(&mut exam, &[Verdict::Wrong]);
        assert_eq!(asked, 6);
        assert_eq!(exam.outcome(), Some(Outcome::Failed));
    }

    #[test]
    fn test_exam_record_after_a_resumed_exam_counts_the_amended_answer() {
        // -- Setup & Fixtures
        let mut exam = exam(128);
        answer_with(&mut exam, &[Verdict::Wrong; 9]);
        exam.override_last();

        // -- Exec
        let asked = answer_with(&mut exam, &[Verdict::Wrong]);
        let record = exam
            .record(1_770_000_000_000)
            .expect("the resumed Exam ended");

        // -- Check
        assert_eq!(asked, 10);
        assert_eq!(record.questions.len(), 10);
        assert_eq!(record.correct_count, 1);
        assert_eq!(record.outcome, Outcome::Failed);
    }

    #[test]
    fn test_exam_record_shows_its_score_as_a_percentage() {
        // -- Setup & Fixtures
        let record = ExamRecord {
            at: 1,
            track: Track::Standard,
            questions: vec![1, 2, 3],
            correct_count: 2,
            outcome: Outcome::Passed,
        };

        // -- Exec & Check
        assert_eq!(record.correct_percentage(), 67, "two of three, rounded");
    }

    #[test]
    fn test_exam_track_follows_the_profile() {
        // -- Exec & Check
        assert_eq!(Track::of(&Profile::default()), Track::Standard);
        assert_eq!(
            Track::of(&Profile {
                special_consideration_65_20: true,
                ..Profile::default()
            }),
            Track::SpecialConsideration
        );
    }
}

// endregion: --- Tests
