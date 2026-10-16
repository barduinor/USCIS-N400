//! Choosing the Civics Test Question to ask next.
//!
//! The choice is a fair rotation: the candidate whose last ask is furthest back
//! wins, and the question just asked never comes straight back (there is one
//! absolute exclusion in the rules: the same question is never asked twice in a
//! row).
//!
//! The bias weights the rules call for — questions asked least often, questions
//! answered wrong, Sections that have been under-served — and the recent-ask
//! penalty that carries a wrong answer past roughly eight to ten other
//! questions are tuned work of their own. This rotation is what the loop
//! chooses from, and the seam those weights belong in.

use crate::{AnswerLogEntry, Question};

/// The Civics Test Question to ask next, or `None` when nothing is in scope.
///
/// `asked` is the question the Applicant has just seen answered, when the
/// caller knows it: the screen knows the moment an answer lands, while the log
/// it read may still be catching up.
pub fn next<'a>(
    candidates: &[&'a Question],
    log: &[AnswerLogEntry],
    asked: Option<u16>,
) -> Option<&'a Question> {
    let asked_last = asked.or_else(|| log.last().map(|entry| entry.question));

    candidates
        .iter()
        .filter(|question| Some(question.number) != asked_last)
        .min_by_key(|question| (last_asked_at(question.number, log), question.number))
        .or_else(|| candidates.first())
        .copied()
}

/// Where in the log the question was last asked.
///
/// `None` sorts before every ask, so a question the Applicant has never been
/// asked comes first.
fn last_asked_at(number: u16, log: &[AnswerLogEntry]) -> Option<usize> {
    log.iter().rposition(|entry| entry.question == number)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AcceptedAnswer, Section, Topic, Verdict};

    /// The Civics Test Question with this number.
    fn candidate(number: u16) -> Question {
        Question {
            number,
            section: Section::AmericanGovernment,
            topic: Topic::PrinciplesOfAmericanGovernment,
            question: "Fixture.".to_string(),
            accepted_answers: vec![AcceptedAnswer::from("Republic")],
            answers_required: 1,
            starred: false,
            explanation: None,
        }
    }

    /// A question's number, answerable as one list of candidates.
    fn candidates(numbers: &[u16]) -> Vec<Question> {
        numbers.iter().copied().map(candidate).collect()
    }

    /// One answer the Applicant gave, in order.
    fn asked(question: u16) -> AnswerLogEntry {
        AnswerLogEntry {
            at: 1_770_000_000_000,
            question,
            answer: "Republic".to_string(),
            verdict: Verdict::Wrong,
            overridden: false,
        }
    }

    /// The number the draw picks from the candidates.
    fn picked(candidates: &[Question], log: &[AnswerLogEntry]) -> Option<u16> {
        picked_after(candidates, log, None)
    }

    /// The number the draw picks, with the question just asked passed in.
    fn picked_after(
        candidates: &[Question],
        log: &[AnswerLogEntry],
        asked: Option<u16>,
    ) -> Option<u16> {
        let candidates: Vec<&Question> = candidates.iter().collect();

        next(&candidates, log, asked).map(|question| question.number)
    }

    #[test]
    fn test_selection_takes_the_candidate_asked_longest_ago() {
        // -- Setup & Fixtures
        let all = candidates(&[1, 2, 3]);
        let log = [asked(1), asked(2), asked(3)];

        // -- Exec & Check
        assert_eq!(picked(&all, &log), Some(1));
        assert_eq!(
            picked(&all, &[asked(2), asked(3), asked(1)]),
            Some(2),
            "the draw follows the log, not the numbering"
        );
    }

    #[test]
    fn test_selection_never_asks_the_same_question_twice_in_a_row() {
        // -- Setup & Fixtures
        let all = candidates(&[1, 2, 3]);

        // -- Exec & Check
        assert_eq!(picked(&all, &[]), Some(1), "nothing asked yet");
        assert_eq!(picked(&all, &[asked(1)]), Some(2));
        assert_eq!(picked(&all, &[asked(1), asked(2)]), Some(3));
        assert_eq!(picked(&all, &[asked(1), asked(2), asked(3)]), Some(1));
    }

    #[test]
    fn test_selection_excludes_the_question_just_asked() {
        // -- Setup & Fixtures
        let all = candidates(&[1, 2, 3]);

        // -- Exec & Check
        assert_eq!(
            picked_after(&all, &[], Some(1)),
            Some(2),
            "the answer may not have reached the log the caller read"
        );
        assert_eq!(picked_after(&all, &[asked(1)], Some(1)), Some(2));
        assert_eq!(picked_after(&all, &[asked(1)], None), Some(2));
    }

    #[test]
    fn test_selection_keeps_asking_when_one_question_is_all_there_is() {
        // -- Setup & Fixtures
        let all = candidates(&[7]);
        let log = [asked(7)];

        // -- Exec & Check
        assert_eq!(
            picked(&all, &log),
            Some(7),
            "a scope of one cannot avoid repeating itself"
        );
        assert_eq!(picked(&all, &[]), Some(7));
    }

    #[test]
    fn test_selection_returns_nothing_without_candidates() {
        // -- Exec & Check
        assert_eq!(picked(&[], &[asked(1)]), None);
        assert_eq!(picked(&[], &[]), None);
    }
}

// endregion: --- Tests
