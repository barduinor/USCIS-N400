//! Judging a typed answer against the Accepted Answers a question accepts.
//!
//! Matching is normalized exact match (ADR 0004): lowercased, with periods
//! dropped, other punctuation and hyphens read as spaces, whitespace collapsed,
//! and an ordinal's stray space repaired (`19 th` is `19th`). Parenthetical
//! words in an Accepted Answer are read by shape (ADR 0004, GLOSSARY): a
//! fragment is accepted present or dropped, an alternative spelling (`or` or
//! `/` inside the parentheses) stands in the answer's place, and a numeral form
//! buys the digit form of the spelled-out number before it. Square brackets and
//! the other extraction artifacts are resolved in the generation pipeline, so
//! the committed records hold none of them, and the records contract test keeps
//! it that way.
//!
//! It is deliberately not fuzzy and not semantic: a wrong accept is invisible
//! to the Applicant and silently corrupts Coverage, so a valid phrasing that no
//! key lists is judged wrong and left to the Override (ADR 0004).
//!
//! A single-answer question is correct when the typed answer equals an Accepted
//! Answer, or joins two or more of their forms with no other words: the typed
//! words read as consecutive runs, each run a form of one Accepted Answer. A
//! multi-item question ("Name two", "Name three") is correct when the typed
//! answer carries as many distinct Accepted Answers as the question demands; a
//! bullet counts once, however many of its variants appear.

use core::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AcceptedAnswer, AlternativeForm, Question};

/// What the tool decided about one typed answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Correct,
    Wrong,
}

/// The Verdict on `typed`, judged against the published answers alone.
pub fn judge(question: &Question, accepted: &[AcceptedAnswer], typed: &str) -> Verdict {
    judge_with(question, accepted, &[], typed, false)
}

/// The Verdict on `typed`, judged against the published answers and the
/// reviewed Alternative Answers attached to them (ADR 0011).
///
/// The eight Profile-backed questions have no published answer to attach a
/// form to, so they take a narrow short-form and typo rule instead.
pub fn judge_reviewed(
    question: &Question,
    accepted: &[AcceptedAnswer],
    alternatives: &[AlternativeForm],
    typed: &str,
) -> Verdict {
    judge_with(question, accepted, alternatives, typed, true)
}

/// The Verdict on `typed`, with the reviewed path optional.
///
/// `reviewed` turns on the Profile-backed short-form rule; the exact `judge`
/// keeps it off, so the published-answer contract is unchanged.
fn judge_with(
    question: &Question,
    accepted: &[AcceptedAnswer],
    alternatives: &[AlternativeForm],
    typed: &str,
    reviewed: bool,
) -> Verdict {
    let typed = normalized(typed);

    if typed.is_empty() {
        return Verdict::Wrong;
    }

    if reviewed && crate::answers::is_fact(question.number) {
        if accepted.iter().any(|answer| forms(answer).contains(&typed)) {
            return Verdict::Correct;
        }

        return if fact_short_form(&typed, accepted) {
            Verdict::Correct
        } else {
            Verdict::Wrong
        };
    }

    let carried = if question.answers_required <= 1 {
        accepted
            .iter()
            .any(|answer| forms_with(answer, alternatives).contains(&typed))
            || joins(&typed, &all_forms(accepted, alternatives))
    } else {
        accepted
            .iter()
            .filter(|answer| {
                forms_with(answer, alternatives)
                    .iter()
                    .any(|form| carries(&typed, form))
            })
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

/// Whether the typed answer joins the question's forms with no other words.
///
/// A single-answer question may hear more than one of its Accepted Answers in
/// one line (`liberty island new york`). The typed words read as consecutive
/// runs, each run a form of one Accepted Answer, and every word must sit inside
/// a run: `not liberty island` is no joining, because `not` is no form's word.
fn joins(typed: &str, forms: &[String]) -> bool {
    let words = typed.split(' ').collect::<Vec<_>>();

    let mut joined = vec![false; words.len() + 1];
    joined[0] = true;

    for start in 0..words.len() {
        if !joined[start] {
            continue;
        }

        for form in forms {
            let end = start + form.split(' ').count();

            if end <= words.len() && form.split(' ').eq(words[start..end].iter().copied()) {
                joined[end] = true;
            }
        }
    }

    joined[words.len()]
}

/// Whether a Profile-backed answer is a shortened or lightly misspelled form
/// of one of the Applicant's values.
///
/// The rule requires at least two typed words, each matching a later word of
/// the value in order; a word match is exact, or the shorter word is the longer
/// with one character missing at either end. It is deliberately narrow: the
/// value is a name, a city, or a state, and `A`, a single first name, or a
/// different surname must not pass (ADR 0011).
fn fact_short_form(typed: &str, accepted: &[AcceptedAnswer]) -> bool {
    let typed_words = typed
        .split(' ')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    if typed_words.len() < 2 {
        return false;
    }

    accepted.iter().any(|answer| {
        let normalized_answer = normalized(answer.as_str());
        let answer_words = normalized_answer
            .split(' ')
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();

        carries_short_form(&typed_words, &answer_words)
    })
}

/// Whether every typed word matches a later answer word, in order.
fn carries_short_form(typed: &[&str], answer: &[&str]) -> bool {
    let mut next = 0;

    for word in answer {
        if next < typed.len() && word_matches(typed[next], word) {
            next += 1;
        }
    }

    next == typed.len()
}

/// Whether one word matches another: exactly, or the shorter is the longer
/// with one character missing at either end, when both are at least four
/// characters long.
fn word_matches(typed: &str, answer: &str) -> bool {
    typed == answer
        || (typed.chars().count() >= 4
            && answer.chars().count() >= 4
            && one_char_at_an_end(typed, answer))
}

/// Whether the shorter of two words is the longer with one end character
/// removed.
fn one_char_at_an_end(left: &str, right: &str) -> bool {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();

    if left.len().abs_diff(right.len()) != 1 {
        return false;
    }

    let (shorter, longer) = if left.len() < right.len() {
        (left, right)
    } else {
        (right, left)
    };

    longer.starts_with(&shorter) || longer.ends_with(&shorter)
}

/// The comparison forms of one Accepted Answer.
fn forms(answer: &AcceptedAnswer) -> Vec<String> {
    spellings(answer.as_str())
        .into_iter()
        .map(|spelling| normalized(&spelling))
        .collect()
}

/// The comparison forms of one Accepted Answer, plus the reviewed forms
/// attached to it (ADR 0011).
fn forms_with(answer: &AcceptedAnswer, alternatives: &[AlternativeForm]) -> Vec<String> {
    let mut comparison_forms = forms(answer);

    for alternative in alternatives {
        if alternative.answer == *answer {
            comparison_forms.extend(forms(&alternative.form));
        }
    }

    comparison_forms
}

/// Every comparison form the question accepts, published and reviewed.
fn all_forms(accepted: &[AcceptedAnswer], alternatives: &[AlternativeForm]) -> Vec<String> {
    accepted
        .iter()
        .flat_map(|answer| forms_with(answer, alternatives))
        .collect()
}

/// The spellings one Accepted Answer accepts: itself, plus for each
/// parenthetical the spelling without it, one for each `or` or `/` alternative
/// inside it, and the numeral form when the parenthetical holds digits
/// (ADR 0004).
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

            if is_numeral(content)
                && let Some(numeral) = numeral_form(&head, content)
            {
                grown.push(numeral);
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

/// The spelling with the spelled-out number before a numeral parenthetical
/// replaced by the parenthetical's own digits, when the words before it spell
/// that number out (ADR 0004).
///
/// `Twenty-seven (27)` buys `27`, and `Citizens eighteen (18) and older` buys
/// `Citizens 18 and older`; a numeral parenthetical that follows no spelled-out
/// number buys nothing, so `The Great Crash (1929)` has no numeral form.
fn numeral_form(head: &str, digits: &str) -> Option<String> {
    let trimmed = head.trim_end();
    let start = trailing_number(trimmed)?;

    Some(format!(
        "{}{digits}{}",
        &trimmed[..start],
        &head[trimmed.len()..]
    ))
}

/// Where the spelled-out number that ends `text` begins, when its last words
/// spell one: `Eighteen`, `Twenty-seven`, `Four hundred thirty-five`.
fn trailing_number(text: &str) -> Option<usize> {
    let mut start = text.len();
    let mut words: Vec<&str> = Vec::new();

    for (index, character) in text.char_indices().rev() {
        if !character.is_whitespace() {
            continue;
        }

        let word = text[index + character.len_utf8()..start].trim();

        if word.is_empty() {
            continue;
        }
        if !is_number_word(word) {
            break;
        }

        words.push(word);
        start = index + character.len_utf8();
    }

    let first = text[..start].trim_end();

    if is_number_word(first) {
        words.push(first);
        start = 0;
    }

    words.reverse();

    spells_a_number(&words).then_some(start)
}

/// Whether the words spell out a number: `Eighteen`, `twenty-seven`,
/// `Four hundred thirty-five`.
fn spells_a_number(words: &[&str]) -> bool {
    let mut value = 0u32;

    for word in words {
        for part in word.split('-') {
            match number_word(part) {
                Some(NumberWord::Plain(number)) => value += number,
                Some(NumberWord::Hundred) => {
                    if value == 0 {
                        return false;
                    }

                    value *= 100;
                }
                None => return false,
            }
        }
    }

    value > 0
}

/// Whether the parenthetical holds nothing but digits.
fn is_numeral(content: &str) -> bool {
    !content.is_empty() && content.bytes().all(|byte| byte.is_ascii_digit())
}

/// Whether the word is a number word, alone or as a hyphenated compound.
fn is_number_word(word: &str) -> bool {
    !word.is_empty() && word.split('-').all(|part| number_word(part).is_some())
}

/// What one number word is worth.
#[derive(Debug, Clone, Copy)]
enum NumberWord {
    /// A value that adds to the number being spelled: `thirty-five` is 30 + 5.
    Plain(u32),

    /// The word `hundred`, which multiplies what it follows.
    Hundred,
}

/// The number words, and what each is worth.
const NUMBER_WORDS: [(&str, NumberWord); 28] = [
    ("one", NumberWord::Plain(1)),
    ("two", NumberWord::Plain(2)),
    ("three", NumberWord::Plain(3)),
    ("four", NumberWord::Plain(4)),
    ("five", NumberWord::Plain(5)),
    ("six", NumberWord::Plain(6)),
    ("seven", NumberWord::Plain(7)),
    ("eight", NumberWord::Plain(8)),
    ("nine", NumberWord::Plain(9)),
    ("ten", NumberWord::Plain(10)),
    ("eleven", NumberWord::Plain(11)),
    ("twelve", NumberWord::Plain(12)),
    ("thirteen", NumberWord::Plain(13)),
    ("fourteen", NumberWord::Plain(14)),
    ("fifteen", NumberWord::Plain(15)),
    ("sixteen", NumberWord::Plain(16)),
    ("seventeen", NumberWord::Plain(17)),
    ("eighteen", NumberWord::Plain(18)),
    ("nineteen", NumberWord::Plain(19)),
    ("twenty", NumberWord::Plain(20)),
    ("thirty", NumberWord::Plain(30)),
    ("forty", NumberWord::Plain(40)),
    ("fifty", NumberWord::Plain(50)),
    ("sixty", NumberWord::Plain(60)),
    ("seventy", NumberWord::Plain(70)),
    ("eighty", NumberWord::Plain(80)),
    ("ninety", NumberWord::Plain(90)),
    ("hundred", NumberWord::Hundred),
];

/// The worth of one number word, if it is one.
fn number_word(word: &str) -> Option<NumberWord> {
    NUMBER_WORDS
        .iter()
        .find(|(number, _)| word.eq_ignore_ascii_case(number))
        .map(|(_, value)| *value)
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
    fn test_matching_reviewed_accepts_an_alternative_but_exact_does_not() {
        // -- Setup & Fixtures
        let fixture = question(&["Executive branch"]);
        let alternatives = [AlternativeForm {
            answer: AcceptedAnswer::from("Executive branch"),
            form: AcceptedAnswer::from("Executive"),
        }];

        // -- Exec & Check
        assert_eq!(
            judge_reviewed(
                &fixture,
                &fixture.accepted_answers,
                &alternatives,
                "Executive"
            ),
            Verdict::Correct,
            "the reviewed form counts as its published answer"
        );
        assert_eq!(
            judged(&fixture, "Executive"),
            Verdict::Wrong,
            "the exact judge does not read alternatives"
        );
    }

    #[test]
    fn test_matching_reviewed_does_not_count_two_forms_of_one_answer() {
        // -- Setup & Fixtures
        let fixture = several(2, &["Freedom of speech", "Freedom of religion"]);
        let alternatives = [AlternativeForm {
            answer: AcceptedAnswer::from("Freedom of religion"),
            form: AcceptedAnswer::from("Religious freedom"),
        }];

        // -- Exec & Check
        assert_eq!(
            judge_reviewed(
                &fixture,
                &fixture.accepted_answers,
                &alternatives,
                "Freedom of speech, religious freedom"
            ),
            Verdict::Correct,
            "two published answers, one through its reviewed form"
        );
        assert_eq!(
            judge_reviewed(
                &fixture,
                &fixture.accepted_answers,
                &alternatives,
                "Freedom of religion, religious freedom"
            ),
            Verdict::Wrong,
            "two forms of the same answer are still one answer"
        );
    }

    #[test]
    fn test_matching_reviewed_accepts_a_fact_short_form_but_not_a_placeholder() {
        // -- Setup & Fixtures
        let mut president = question(&["Donald Trump"]);
        president.number = 38;
        let mut chief = question(&["John G. Roberts Jr"]);
        chief.number = 57;
        let mut representative = question(&["Hank Johnson"]);
        representative.number = 29;

        // -- Exec & Check
        assert_eq!(
            judge_reviewed(&president, &president.accepted_answers, &[], "Donal trump"),
            Verdict::Correct,
            "a one-letter typo in the Profile value is accepted"
        );
        assert_eq!(
            judge_reviewed(&chief, &chief.accepted_answers, &[], "John roberts"),
            Verdict::Correct,
            "the middle initial and suffix are optional"
        );
        assert_eq!(
            judge_reviewed(&president, &president.accepted_answers, &[], "A"),
            Verdict::Wrong,
            "a placeholder never passes"
        );
        assert_eq!(
            judge_reviewed(
                &representative,
                &representative.accepted_answers,
                &[],
                "Hank Johnston"
            ),
            Verdict::Wrong,
            "a different surname is not a typo"
        );
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
    fn test_matching_takes_a_fragment_present_or_dropped() {
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
    fn test_matching_buys_the_digit_form_a_numeral_parenthetical_follows() {
        // -- Setup & Fixtures
        // The ten Accepted Answers whose parenthetical numeral buys a digit
        // form, as the committed records hold them (ADR 0004).
        let numerals = [
            ("Twenty-seven (27)", "27"),
            ("One hundred (100)", "100"),
            ("Six (6) years", "6 years"),
            ("Four hundred thirty-five (435)", "435"),
            ("Two (2) years", "2 years"),
            ("Two (2)", "2"),
            ("Four (4) years", "4 years"),
            ("Nine (9)", "9"),
            ("Five (5)", "5"),
            (
                "Citizens eighteen (18) and older (can vote).",
                "citizens 18 and older",
            ),
        ];

        // -- Exec & Check
        for (answer, digit_form) in numerals {
            let fixture = question(&[answer]);

            assert_eq!(
                judged(&fixture, answer),
                Verdict::Correct,
                "`{answer}` must still accept itself"
            );
            assert_eq!(
                judged(&fixture, digit_form),
                Verdict::Correct,
                "`{answer}` must accept its digit form `{digit_form}`"
            );
        }
    }

    #[test]
    fn test_matching_does_not_buy_a_numeral_that_follows_no_spelled_out_number() {
        // -- Setup & Fixtures
        let crash = question(&["The Great Crash (1929)"]);
        let other = question(&["Crash of (1929)"]);

        // -- Exec & Check
        assert_eq!(judged(&crash, "The Great Crash"), Verdict::Correct);
        assert_eq!(judged(&crash, "The Great Crash 1929"), Verdict::Correct);
        assert_eq!(
            judged(&crash, "1929"),
            Verdict::Wrong,
            "the numerals buy a digit form only where a spelled-out number precedes them"
        );
        assert_eq!(judged(&other, "1929"), Verdict::Wrong);
    }

    #[test]
    fn test_matching_takes_a_fragments_words_as_no_answer_of_their_own() {
        // -- Setup & Fixtures
        // A fragment before the answer, inside it, after it, and one that
        // holds a whole sentence rather than a phrase (ADR 0004).
        let fragments = [
            ("(U.S.) Constitution", "us"),
            ("(Example of) social contract", "example of"),
            (
                "Citizens in their (congressional) district",
                "congressional",
            ),
            ("House (of Representatives)", "of representatives"),
            (
                "Any citizen can vote. (Women and men can vote.)",
                "women and men can vote",
            ),
            ("The Great Crash (1929)", "1929"),
        ];

        // -- Exec & Check
        for (answer, words) in fragments {
            let fixture = question(&[answer]);

            assert_eq!(
                judged(&fixture, answer),
                Verdict::Correct,
                "`{answer}` must accept itself"
            );
            assert_eq!(
                judged(&fixture, words),
                Verdict::Wrong,
                "`{words}` are a fragment of `{answer}`, not an answer"
            );
        }
    }

    #[test]
    fn test_matching_joins_the_questions_forms_with_no_other_words() {
        // -- Setup & Fixtures
        // Where is the Statue of Liberty? — five answers, one required.
        let liberty = question(&[
            "New York (Harbor)",
            "Liberty Island",
            "New Jersey",
            "near New York City",
            "on the Hudson (River)",
        ]);
        let amendment = question(&["Twenty-seven (27)"]);

        // -- Exec & Check
        assert_eq!(
            judged(&liberty, "liberty island new york"),
            Verdict::Correct,
            "two of the question's answers may be said in one line"
        );
        assert_eq!(
            judged(&liberty, "new jersey liberty island"),
            Verdict::Correct,
            "the order of the answers does not matter"
        );
        assert_eq!(
            judged(&liberty, "new jersey near new york city"),
            Verdict::Correct,
            "either order may hold longer answers"
        );
        assert_eq!(
            judged(&amendment, "27 twenty seven"),
            Verdict::Correct,
            "both runs may be forms of the same answer"
        );
    }

    #[test]
    fn test_matching_does_not_join_words_beside_the_answers() {
        // -- Setup & Fixtures
        let liberty = question(&[
            "New York (Harbor)",
            "Liberty Island",
            "New Jersey",
            "near New York City",
            "on the Hudson (River)",
        ]);

        // -- Exec & Check
        assert_eq!(
            judged(&liberty, "not liberty island"),
            Verdict::Wrong,
            "`not` belongs to none of the question's forms"
        );
        assert_eq!(
            judged(&liberty, "statue island"),
            Verdict::Wrong,
            "`statue` belongs to none of the question's forms"
        );
        assert_eq!(
            judged(&liberty, "the statue"),
            Verdict::Wrong,
            "`statue` belongs to none of the question's forms"
        );
        assert_eq!(
            judged(&liberty, "i do not know"),
            Verdict::Wrong,
            "no word here belongs to any of the question's forms"
        );
        assert_eq!(
            judged(&liberty, "island liberty"),
            Verdict::Wrong,
            "an answer appears as a run of whole words in its own order"
        );
        assert_eq!(
            judged(&liberty, "the liberty island"),
            Verdict::Wrong,
            "a word that belongs to an answer but makes no form of its own stays out"
        );
    }

    #[test]
    fn test_matching_counts_a_numeral_form_as_its_bullet() {
        // -- Setup & Fixtures
        let fixture = several(2, &["Senate", "Two (2) years"]);

        // -- Exec & Check
        assert_eq!(judged(&fixture, "Senate and 2 years"), Verdict::Correct);
        assert_eq!(judged(&fixture, "2 years and the Senate"), Verdict::Correct);
        assert_eq!(judged(&fixture, "2 years"), Verdict::Wrong);
        assert_eq!(
            judged(&fixture, "two years and 2 years"),
            Verdict::Wrong,
            "a digit form is another form of its bullet, not a second answer"
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
