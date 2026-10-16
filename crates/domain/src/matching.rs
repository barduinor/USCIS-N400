//! Judging a typed answer against the Accepted Answers a question accepts.
//!
//! Matching is normalized exact match (ADR 0004): lowercased, with periods
//! dropped, other punctuation and hyphens read as spaces, whitespace collapsed,
//! and an ordinal's stray space repaired (`19 th` is `19th`). Parenthetical
//! words in an Accepted Answer are optional, and `or` or `/` inside parentheses
//! spell alternatives. Square brackets and the other extraction artifacts are
//! resolved in the generation pipeline, so the committed records hold none of
//! them, and the records contract test keeps it that way.
//!
//! It is deliberately not fuzzy and not semantic: a wrong accept is invisible
//! to the Applicant and silently corrupts Coverage, so a valid phrasing that no
//! key lists is judged wrong and left to the Override (ADR 0004).
//!
//! A single-answer question is correct when the typed answer equals an Accepted
//! Answer. A multi-item question ("Name two", "Name three") is correct when the
//! typed answer carries as many distinct Accepted Answers as the question
//! demands; a bullet counts once, however many of its variants appear.

use core::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AcceptedAnswer, Question};

/// What the tool decided about one typed answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Correct,
    Wrong,
}

/// The Verdict on `typed`, judged against the answers this question accepts.
pub fn judge(question: &Question, accepted: &[AcceptedAnswer], typed: &str) -> Verdict {
    let typed = normalized(typed);

    if typed.is_empty() {
        return Verdict::Wrong;
    }

    let carried = if question.answers_required <= 1 {
        accepted.iter().any(|answer| forms(answer).contains(&typed))
    } else {
        accepted
            .iter()
            .filter(|answer| forms(answer).iter().any(|form| carries(&typed, form)))
            .count()
            >= usize::from(question.answers_required)
    };

    if carried {
        Verdict::Correct
    } else {
        Verdict::Wrong
    }
}

/// Whether the typed answer carries these words as a run of whole words.
///
/// Multi-item questions are the only place a typed answer may hold more than
/// the answer itself (`Senate and the House of Representatives`).
fn carries(typed: &str, form: &str) -> bool {
    !form.is_empty() && format!(" {typed} ").contains(&format!(" {form} "))
}

/// The comparison forms of one Accepted Answer.
fn forms(answer: &AcceptedAnswer) -> Vec<String> {
    spellings(answer.as_str())
        .into_iter()
        .map(|spelling| normalized(&spelling))
        .collect()
}

/// The spellings one Accepted Answer accepts: itself, plus for each
/// parenthetical the spelling without it and one for each `or` or `/`
/// alternative inside it (ADR 0004).
///
/// A parenthetical that opens with `or` corrects the word before it rather than
/// adding one: `Saved (or preserved) the Union` accepts `Saved the Union` and
/// `Preserved the Union`, not `Saved preserved the Union`.
fn spellings(answer: &str) -> Vec<String> {
    let mut spellings = vec![String::new()];
    let mut rest = answer;

    while let Some(open) = rest.find('(') {
        let Some(close) = rest[open..].find(')').map(|close| open + close) else {
            break;
        };
        let (before, parenthetical) = rest.split_at(open);
        let content = parenthetical[1..close - open].trim();
        let (correction, content) = match content.strip_prefix("or ") {
            Some(content) => (true, content),
            None => match content.strip_prefix('/') {
                Some(content) => (true, content),
                None => (false, content),
            },
        };

        let mut grown = Vec::new();
        for spelling in &spellings {
            let mut head = spelling.clone();
            head.push_str(before);

            grown.push(head.clone());
            for alternative in alternatives(content) {
                let alternative = if correction {
                    format!("{} {alternative}", without_last_word(&head))
                } else {
                    format!("{head} {alternative}")
                };

                grown.push(alternative);
            }
        }
        spellings = grown;
        rest = &rest[close + 1..];
    }

    for spelling in &mut spellings {
        spelling.push_str(rest);
    }

    spellings
}

/// The text without its last word.
fn without_last_word(text: &str) -> &str {
    let trimmed = text.trim_end();

    match trimmed.rfind(' ') {
        Some(space) => &trimmed[..space],
        None => "",
    }
}

/// The alternatives one parenthetical holds, split on `or` and `/`.
///
/// Only parentheses are split: splitting everywhere would break "work for
/// local, state, or federal government" (ADR 0004).
fn alternatives(parenthetical: &str) -> Vec<&str> {
    parenthetical
        .split(" or ")
        .flat_map(|alternative| alternative.split('/'))
        .map(str::trim)
        .filter(|alternative| !alternative.is_empty())
        .collect()
}

/// The comparison form of an answer (ADR 0004): lowercased alphanumeric words,
/// with periods dropped as abbreviation marks, other punctuation read as
/// spaces, whitespace collapsed, and an ordinal's stray space repaired.
fn normalized(answer: &str) -> String {
    let spaced: String = answer
        .chars()
        .filter_map(|character| match character {
            '.' => None,
            _ if character.is_alphanumeric() => Some(character.to_ascii_lowercase()),
            _ => Some(' '),
        })
        .collect();
    let collapsed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");

    repaired_ordinals(&collapsed)
}

/// Repairs the space an extraction leaves inside an ordinal: `19 th` is `19th`.
fn repaired_ordinals(answer: &str) -> String {
    let words = answer.split(' ').collect::<Vec<_>>();
    let mut repaired = String::with_capacity(answer.len());
    let mut index = 0;

    while index < words.len() {
        let word = words[index];
        let suffix = words.get(index + 1).copied().filter(|suffix| {
            !word.is_empty()
                && word.chars().all(|character| character.is_ascii_digit())
                && matches!(*suffix, "th" | "nd" | "rd" | "st")
        });

        if !repaired.is_empty() {
            repaired.push(' ');
        }

        match suffix {
            Some(suffix) => {
                repaired.push_str(word);
                repaired.push_str(suffix);
                index += 2;
            }
            None => {
                repaired.push_str(word);
                index += 1;
            }
        }
    }

    repaired
}

impl Display for Verdict {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Verdict::Correct => "Correct",
            Verdict::Wrong => "Wrong",
        })
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Section, Topic};

    /// A one-answer question holding the given published answers.
    fn question(answers: &[&str]) -> Question {
        several(1, answers)
    }

    /// A question demanding `answers_required` distinct answers.
    fn several(answers_required: u16, answers: &[&str]) -> Question {
        Question {
            number: 1,
            section: Section::AmericanGovernment,
            topic: Topic::PrinciplesOfAmericanGovernment,
            question: "Fixture.".to_string(),
            accepted_answers: answers.iter().copied().map(AcceptedAnswer::from).collect(),
            answers_required,
            starred: false,
            explanation: None,
        }
    }

    /// What the matcher decides about `typed`.
    fn judged(fixture: &Question, typed: &str) -> Verdict {
        judge(fixture, &fixture.accepted_answers, typed)
    }

    #[test]
    fn test_matching_judges_an_official_answer_correct() {
        // -- Setup & Fixtures
        let fixture = question(&[
            "Republic",
            "Constitution-based federal republic",
            "Representative democracy",
        ]);

        // -- Exec & Check
        assert_eq!(
            judged(&fixture, "Representative democracy"),
            Verdict::Correct
        );
        assert_eq!(
            judged(&fixture, "representative DEMOCRACY"),
            Verdict::Correct
        );
        assert_eq!(
            judged(&fixture, "Constitution-based federal republic"),
            Verdict::Correct
        );
    }

    #[test]
    fn test_matching_judges_an_unlisted_phrasing_wrong() {
        // -- Setup & Fixtures
        let fixture = question(&["Republic"]);

        // -- Exec & Check
        assert_eq!(judged(&fixture, "A republic"), Verdict::Wrong);
        assert_eq!(judged(&fixture, "democracy"), Verdict::Wrong);
        assert_eq!(judged(&fixture, ""), Verdict::Wrong);
        assert_eq!(judged(&fixture, "   "), Verdict::Wrong);
    }

    #[test]
    fn test_matching_takes_parenthetical_words_as_optional() {
        // -- Setup & Fixtures
        let fixture = question(&["House (of Representatives)"]);

        // -- Exec & Check
        assert_eq!(
            judged(&fixture, "House of Representatives"),
            Verdict::Correct
        );
        assert_eq!(
            judged(&fixture, "the House of Representatives"),
            Verdict::Wrong
        );
        assert_eq!(judged(&fixture, "House"), Verdict::Correct);
    }

    #[test]
    fn test_matching_accepts_each_parenthetical_alternative() {
        // -- Setup & Fixtures
        let fixture = question(&["(U.S. or national) legislature"]);

        // -- Exec & Check
        assert_eq!(judged(&fixture, "U.S. legislature"), Verdict::Correct);
        assert_eq!(judged(&fixture, "national legislature"), Verdict::Correct);
        assert_eq!(judged(&fixture, "legislature"), Verdict::Correct);
        assert_eq!(judged(&fixture, "state legislature"), Verdict::Wrong);
    }

    #[test]
    fn test_matching_does_not_split_on_or_outside_parentheses() {
        // -- Setup & Fixtures
        let fixture = question(&["Work for local, state, or federal government"]);

        // -- Exec & Check
        assert_eq!(
            judged(&fixture, "Work for local, state, or federal government"),
            Verdict::Correct
        );
        assert_eq!(judged(&fixture, "federal government"), Verdict::Wrong);
    }

    #[test]
    fn test_matching_reads_periods_hyphens_and_ordinals_from_both_sides() {
        // -- Setup & Fixtures
        let constitution = question(&["(U.S.) Constitution"]);
        let vice_president = question(&["Vice-President"]);
        let amendment = question(&["19th Amendment"]);

        // -- Exec & Check
        assert_eq!(judged(&constitution, "U.S. Constitution"), Verdict::Correct);
        assert_eq!(judged(&constitution, "US Constitution"), Verdict::Correct);
        assert_eq!(judged(&constitution, "Constitution."), Verdict::Correct);
        assert_eq!(judged(&vice_president, "Vice President"), Verdict::Correct);
        assert_eq!(judged(&amendment, "19 th amendment"), Verdict::Correct);
        assert_eq!(judged(&amendment, "the 19th Amendment"), Verdict::Wrong);
    }

    #[test]
    fn test_matching_lets_a_parenthetical_correct_the_word_before_it() {
        // -- Setup & Fixtures
        let fixture = question(&["Saved (or preserved) the Union"]);

        // -- Exec & Check
        assert_eq!(judged(&fixture, "Saved the Union"), Verdict::Correct);
        assert_eq!(judged(&fixture, "Preserved the Union"), Verdict::Correct);
        assert_eq!(
            judged(&fixture, "Saved preserved the Union"),
            Verdict::Wrong,
            "the parenthetical corrects the word before it, it does not add one"
        );
    }

    #[test]
    fn test_matching_counts_distinct_answers_for_multi_item_questions() {
        // -- Setup & Fixtures
        let fixture = several(2, &["Senate", "House (of Representatives)"]);
        let branches = several(3, &["Legislative", "executive", "judicial"]);

        // -- Exec & Check
        assert_eq!(
            judged(&fixture, "Senate and the House of Representatives"),
            Verdict::Correct
        );
        assert_eq!(judged(&fixture, "House and Senate"), Verdict::Correct);
        assert_eq!(judged(&fixture, "House"), Verdict::Wrong);
        assert_eq!(judged(&fixture, "senate"), Verdict::Wrong);
        assert_eq!(judged(&fixture, "Senate and senate"), Verdict::Wrong);
        assert_eq!(
            judged(&branches, "legislative and executive"),
            Verdict::Wrong
        );
        assert_eq!(
            judged(&branches, "Legislative, executive, and judicial"),
            Verdict::Correct
        );
    }

    #[test]
    fn test_matching_counts_bullets_not_variants() {
        // -- Setup & Fixtures
        let fixture = several(2, &["Senate", "House (of Representatives)"]);

        // -- Exec & Check
        assert_eq!(
            judged(&fixture, "House and House of Representatives"),
            Verdict::Wrong,
            "two spellings of one bullet are one answer"
        );
    }
}

// endregion: --- Tests
