//! Choosing the Civics Test Question to ask next in study mode.
//!
//! The Exam draws its questions at random instead (GLOSSARY, §Exam): the bias
//! below is what keeps study relevant, not what a test simulation may do.
//!
//! The next question is the highest-weighted candidate, and the weights are the
//! rules' bias: questions asked least often, questions answered wrong, and
//! Sections that have been under-served. The recent-ask penalty carries a
//! question past roughly eight to ten others before it can win again; it decays
//! towards zero without reaching it, so a question is never blocked, only held
//! back. Cleared questions are never excluded — a question the Applicant keeps
//! getting right reappears whenever its weight wins.
//!
//! The one absolute exclusion is the question just asked, which the caller
//! passes in: the screen knows the moment an answer lands, while the log it read
//! may still be catching up.
//!
//! The constants below are the tuning knobs. They are worth adjusting against
//! real sessions, not defending.

use core::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use crate::{AnswerLogEntry, Question, Section};

/// What one question is worth for having been asked less often than the busiest
/// candidate.
const LEAST_ASKED_WEIGHT: i64 = 2;

/// What a question is worth whose last answer was wrong.
///
/// This is a state, not an accumulation: a question the Applicant keeps failing
/// cannot outgrow the recent-ask penalty and crowd the field, so the gap the
/// penalty promises survives however many times it has been missed. An Override
/// counts as correct, so an overridden answer clears this.
const WRONG_ANSWER_WEIGHT: i64 = 6;

/// What one question of an under-served Section is worth, per question of
/// deficit.
///
/// Sections are compared per question, not in total: a Section of two questions
/// asked four times is served, a Section of twelve asked twelve times is not.
/// The deficit is capped (below) because the bias steers the choice, it does not
/// decide it — anything larger starts spending the gap a wrong answer is owed.
const SECTION_BALANCE_WEIGHT: i64 = 2;

/// How far behind a Section may count as, in questions.
const SECTION_DEFICIT_CAP: u32 = 2;

/// The recent-ask penalty, divided by the number of questions asked since, so it
/// shrinks with distance and never reaches zero: a question asked recently is
/// held back, never blocked.
///
/// Sized against the wrong-answer weight: with these two, a question just
/// answered wrong comes back after about ten other questions, which is the gap
/// the rules ask for.
const RECENT_ASK_PENALTY: i64 = 40;

/// The Civics Test Question to ask next, or `None` when nothing is in scope.
pub fn next<'a>(
    candidates: &[&'a Question],
    log: &[AnswerLogEntry],
    asked: Option<u16>,
) -> Option<&'a Question> {
    let service = Service::of(candidates, log);
    let asked_last = asked.or_else(|| log.last().map(|entry| entry.question));

    candidates
        .iter()
        .filter(|question| Some(question.number) != asked_last)
        .max_by_key(|question| {
            // The weight leads; a tie goes to the question asked longest ago —
            // one never asked counts as longest ago — and then to the lowest
            // number.
            (
                service.weight(question, log),
                Reverse(service.asked_at(question.number, log)),
                Reverse(question.number),
            )
        })
        .or_else(|| candidates.first())
        .copied()
}

/// How the Answer Log has served the candidates.
struct Service {
    /// How often each question was asked, by question number.
    asks: HashMap<u16, u32>,

    /// The questions whose last answer was wrong.
    failed: HashSet<u16>,

    /// How often the candidates of each Section were asked, in `Section::ALL`'s
    /// order.
    section_asks: [u32; Section::ALL.len()],

    /// How many candidates each Section has, in `Section::ALL`'s order.
    section_sizes: [u32; Section::ALL.len()],

    /// The count the busiest candidate carries.
    most_asked: u32,
}

impl Service {
    /// Reads the log, and how it has served these candidates.
    fn of(candidates: &[&Question], log: &[AnswerLogEntry]) -> Self {
        let mut asks = HashMap::new();
        let mut failed = HashSet::new();

        for entry in log {
            *asks.entry(entry.question).or_default() += 1;

            if entry.counts_correct() {
                failed.remove(&entry.question);
            } else {
                failed.insert(entry.question);
            }
        }

        let mut section_asks = [0; Section::ALL.len()];
        let mut section_sizes = [0; Section::ALL.len()];
        let mut most_asked = 0;

        for question in candidates {
            let asked = asks.get(&question.number).copied().unwrap_or_default();
            let section = section_index(question.section);

            section_asks[section] += asked;
            section_sizes[section] += 1;
            most_asked = most_asked.max(asked);
        }

        Self {
            asks,
            failed,
            section_asks,
            section_sizes,
            most_asked,
        }
    }

    /// What the question is worth: the bias it has earned, less the penalty for
    /// having been asked recently.
    fn weight(&self, question: &Question, log: &[AnswerLogEntry]) -> i64 {
        let asked = self.asks.get(&question.number).copied().unwrap_or_default();
        let failed = if self.failed.contains(&question.number) {
            WRONG_ANSWER_WEIGHT
        } else {
            0
        };

        let bias = i64::from(self.most_asked - asked) * LEAST_ASKED_WEIGHT
            + failed
            + i64::from(self.deficit(question.section)) * SECTION_BALANCE_WEIGHT;
        let penalty = match self.asked_at(question.number, log) {
            None => 0,
            Some(at) => {
                let gaps = (log.len() - at) as i64;

                // Never zero: a question asked long ago is still asked, and an
                // unasked one is still the better candidate.
                (RECENT_ASK_PENALTY / gaps).max(1)
            }
        };

        bias - penalty
    }

    /// How far the Section is behind the best-served Section, in the Section's
    /// own questions.
    ///
    /// The Sections are compared per question — each Section's asks scaled to
    /// the largest Section's size — so a Section of two questions asked four
    /// times is served and a Section of twelve asked twelve times is not.
    fn deficit(&self, section: Section) -> u32 {
        let largest = self.section_sizes.iter().copied().max().unwrap_or_default();

        if largest == 0 {
            return 0;
        }

        let scaled = |at: usize| -> u32 {
            let size = self.section_sizes[at];

            if size == 0 {
                return 0;
            }

            self.section_asks[at] * largest / size
        };
        let best = (0..Section::ALL.len())
            .map(scaled)
            .max()
            .unwrap_or_default();
        let section = section_index(section);
        let behind = (best - scaled(section)) / self.section_sizes[section].max(1);

        behind.min(SECTION_DEFICIT_CAP)
    }

    /// Where in the log the question was last asked.
    ///
    /// `None` means the Applicant has never been asked it.
    fn asked_at(&self, number: u16, log: &[AnswerLogEntry]) -> Option<usize> {
        log.iter().rposition(|entry| entry.question == number)
    }
}

/// The slot a Section's asks are counted in.
fn section_index(section: Section) -> usize {
    match section {
        Section::AmericanGovernment => 0,
        Section::AmericanHistory => 1,
        Section::SymbolsAndHolidays => 2,
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AcceptedAnswer, Section, Topic, Verdict};

    /// The Civics Test Question with this number, in this Section.
    fn candidate_in(number: u16, section: Section) -> Question {
        Question {
            number,
            section,
            topic: Topic::PrinciplesOfAmericanGovernment,
            question: "Fixture.".to_string(),
            accepted_answers: vec![AcceptedAnswer::from("Republic")],
            answers_required: 1,
            starred: false,
            explanation: None,
        }
    }

    /// The Civil Test Question with this number, in a Section of its own.
    fn candidate(number: u16) -> Question {
        candidate_in(number, Section::AmericanGovernment)
    }

    /// The questions numbered 1 to `count`, all in one Section, so that only the
    /// ask count and the wrong answers are in play.
    fn cohort(count: u16) -> Vec<Question> {
        (1..=count).map(candidate).collect()
    }

    /// One answer the Applicant gave.
    fn answered(question: u16, verdict: Verdict) -> AnswerLogEntry {
        AnswerLogEntry {
            at: 1_770_000_000_000 + u64::from(question),
            question,
            answer: "Republic".to_string(),
            verdict,
            overridden: false,
        }
    }

    /// The question the weights choose, or `None` when nothing is in scope.
    fn picked(candidates: &[Question], log: &[AnswerLogEntry], asked: Option<u16>) -> Option<u16> {
        let candidates: Vec<&Question> = candidates.iter().collect();

        next(&candidates, log, asked).map(|question| question.number)
    }

    /// Asks `picks` questions in a row, answering each one with `verdict`, and
    /// reports the sequence that came up.
    fn asked_in_a_row(
        candidates: &[Question],
        log: &mut Vec<AnswerLogEntry>,
        picks: usize,
        verdict: Verdict,
    ) -> Vec<u16> {
        let mut asked = Vec::new();

        for _ in 0..picks {
            let question = picked(candidates, log, None).expect("a candidate");

            asked.push(question);
            log.push(answered(question, verdict));
        }

        asked
    }

    #[test]
    fn test_selection_asks_the_least_asked_question_first() {
        // -- Setup & Fixtures
        let candidates = cohort(8);

        // -- Exec
        let asked = asked_in_a_row(
            &candidates,
            &mut Vec::new(),
            candidates.len(),
            Verdict::Wrong,
        );

        // -- Check
        assert_eq!(
            asked,
            (1..=8).collect::<Vec<_>>(),
            "every question comes up once before any of them comes up twice"
        );
    }

    #[test]
    fn test_selection_never_asks_the_same_question_twice_in_a_row() {
        // -- Setup & Fixtures
        let candidates = cohort(3);

        // -- Exec
        let asked = asked_in_a_row(&candidates, &mut Vec::new(), 30, Verdict::Wrong);

        // -- Check
        for pair in asked.windows(2) {
            assert_ne!(pair[0], pair[1], "the same question came straight back");
        }
    }

    #[test]
    fn test_selection_carries_a_wrong_answer_past_several_other_questions() {
        // -- Setup & Fixtures
        let candidates = cohort(128);

        // -- Exec
        let asked = asked_in_a_row(&candidates, &mut Vec::new(), 40, Verdict::Wrong);

        // -- Check
        let (first, repeat) = asked
            .iter()
            .enumerate()
            .find_map(|(position, question)| {
                asked[..position]
                    .iter()
                    .position(|earlier| earlier == question)
                    .map(|earlier| (earlier, position))
            })
            .expect("a question comes back");
        let gap = repeat - first;

        assert!(
            (8..=16).contains(&gap),
            "a wrong answer came back after {gap} other questions: {asked:?}"
        );
    }

    #[test]
    fn test_selection_returns_an_old_wrong_answer_before_an_older_right_one() {
        // -- Setup & Fixtures
        let candidates = cohort(10);
        let mut log = vec![answered(2, Verdict::Correct)];
        log.push(answered(1, Verdict::Wrong));
        for number in 3..=10 {
            log.push(answered(number, Verdict::Correct));
        }

        // -- Exec
        let picked = picked(&candidates, &log, None);

        // -- Check
        assert_eq!(
            picked,
            Some(1),
            "the wrong answer wins although it was asked later than 2"
        );
    }

    #[test]
    fn test_selection_turns_to_an_under_served_section() {
        // -- Setup & Fixtures
        let mut candidates: Vec<Question> = (1..=3).map(candidate).collect();
        candidates.push(candidate_in(73, Section::AmericanHistory));

        // Three Government questions asked twice each, the History question once
        // and most recently: per question the History Section is the one behind,
        // and that has to carry its question over the recency penalty.
        let log = vec![
            answered(73, Verdict::Correct),
            answered(1, Verdict::Correct),
            answered(2, Verdict::Correct),
            answered(3, Verdict::Correct),
            answered(1, Verdict::Correct),
            answered(2, Verdict::Correct),
            answered(3, Verdict::Correct),
        ];

        // -- Exec
        let picked = picked(&candidates, &log, None);

        // -- Check
        assert_eq!(
            picked,
            Some(73),
            "one History question against three Government ones asked twice each"
        );
    }

    #[test]
    fn test_selection_serves_the_sections_in_proportion() {
        // -- Setup & Fixtures
        let mut candidates: Vec<Question> = (1..=4).map(candidate).collect();
        candidates.push(candidate_in(73, Section::AmericanHistory));
        candidates.push(candidate_in(74, Section::AmericanHistory));

        // -- Exec
        let mut log = Vec::new();
        let asked = asked_in_a_row(&candidates, &mut log, 120, Verdict::Correct);

        // -- Check
        // Counted per question, so the Section with fewer questions is not
        // starved and the Section with more is not flooded.
        let counts: Vec<usize> = candidates
            .iter()
            .map(|candidate| {
                asked
                    .iter()
                    .filter(|number| **number == candidate.number)
                    .count()
            })
            .collect();
        let least = counts.iter().copied().min().unwrap_or_default();
        let most = counts.iter().copied().max().unwrap_or_default();

        assert!(
            most - least <= 4,
            "every question is served about as often, whatever its Section: {counts:?}"
        );
    }

    #[test]
    fn test_selection_does_not_let_a_repeatedly_wrong_question_crowd_the_field() {
        // -- Setup & Fixtures
        let candidates = cohort(20);
        let mut log = Vec::new();
        let mut asked = Vec::new();

        // -- Exec
        // Question 1 is missed every time it comes up, the rest are answered.
        for _ in 0..60 {
            let question = picked(&candidates, &log, None).expect("a candidate");
            let verdict = if question == 1 {
                Verdict::Wrong
            } else {
                Verdict::Correct
            };

            asked.push(question);
            log.push(answered(question, verdict));
        }

        // -- Check
        let appearances: Vec<usize> = asked
            .iter()
            .enumerate()
            .filter(|(_, number)| **number == 1)
            .map(|(position, _)| position)
            .collect();
        let gap = appearances
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .min()
            .expect("the question comes back");

        assert!(
            gap >= 8,
            "a question the Applicant keeps missing came back after {gap} other questions: {asked:?}"
        );
    }

    #[test]
    fn test_selection_excludes_the_question_just_asked() {
        // -- Setup & Fixtures
        let candidates = cohort(3);

        // -- Exec & Check
        assert_eq!(
            picked(&candidates, &[], Some(1)),
            Some(2),
            "the answer may not have reached the log the caller read"
        );
        assert_eq!(
            picked(&candidates, &[answered(1, Verdict::Wrong)], Some(1)),
            Some(2)
        );
        assert_eq!(
            picked(&candidates, &[answered(1, Verdict::Wrong)], None),
            Some(2)
        );
    }

    #[test]
    fn test_selection_prefers_the_question_asked_longest_ago_when_all_else_is_equal() {
        // -- Setup & Fixtures
        let candidates = cohort(3);

        // -- Exec & Check
        assert_eq!(
            picked(
                &candidates,
                &[
                    answered(1, Verdict::Wrong),
                    answered(2, Verdict::Wrong),
                    answered(3, Verdict::Wrong)
                ],
                None
            ),
            Some(1)
        );
        assert_eq!(
            picked(
                &candidates,
                &[
                    answered(2, Verdict::Wrong),
                    answered(3, Verdict::Wrong),
                    answered(1, Verdict::Wrong)
                ],
                None
            ),
            Some(2),
            "the weights follow the log, not the numbering"
        );
    }

    #[test]
    fn test_selection_keeps_asking_when_one_question_is_all_there_is() {
        // -- Setup & Fixtures
        let candidates = cohort(1);
        let log = vec![answered(1, Verdict::Wrong)];

        // -- Exec & Check
        assert_eq!(
            picked(&candidates, &log, None),
            Some(1),
            "a scope of one cannot avoid repeating itself"
        );
        assert_eq!(picked(&candidates, &[], None), Some(1));
    }

    #[test]
    fn test_selection_returns_nothing_without_candidates() {
        // -- Exec & Check
        assert_eq!(picked(&[], &[answered(1, Verdict::Wrong)], None), None);
        assert_eq!(picked(&[], &[], None), None);
    }
}

// endregion: --- Tests
