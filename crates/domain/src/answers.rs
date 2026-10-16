//! The Accepted Answers a Civics Test Question accepts for one Applicant.
//!
//! Most questions accept the answers M-1778 publishes. Eight take their answer
//! from the Profile instead: the Local Facts (state senators, U.S.
//! representative, governor, state capital) and the four Test Updates. USCIS
//! publishes a placeholder for those — `Answers will vary.` for the Local Facts
//! and a pointer to uscis.gov/citizenship/testupdates for the Test Updates —
//! and the records keep that text verbatim, so the resolution happens here, at
//! judging time, rather than in the question list.
//!
//! A question whose value the Profile does not hold accepts nothing: it stays
//! in play and any answer is judged wrong until the Applicant fills the value
//! in (GLOSSARY).

use crate::{AcceptedAnswer, Profile, Question};

/// The Profile value a question's answer comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fact {
    StateSenators,
    Representative,
    Governor,
    StateCapital,
    President,
    VicePresident,
    SpeakerOfTheHouse,
    ChiefJustice,
}

/// The fact-bearing questions, keyed by M-1778's numbers; the numbering is
/// fixed by the 2025 instrument (ADR 0001).
///
/// The records contract test guards the pairing from both sides: every question
/// whose published answer is a placeholder must be listed here, and every
/// question listed here must publish a placeholder.
const FACTS: [(u16, Fact); 8] = [
    (23, Fact::StateSenators),
    (29, Fact::Representative),
    (30, Fact::SpeakerOfTheHouse),
    (38, Fact::President),
    (39, Fact::VicePresident),
    (57, Fact::ChiefJustice),
    (61, Fact::Governor),
    (62, Fact::StateCapital),
];

/// The Accepted Answers `question` accepts for this Applicant.
pub fn accepted_answers(question: &Question, profile: &Profile) -> Vec<AcceptedAnswer> {
    let Some((_, fact)) = FACTS.iter().find(|(number, _)| *number == question.number) else {
        return question.accepted_answers.clone();
    };

    fact.values(profile)
        .into_iter()
        .map(AcceptedAnswer::from)
        .collect()
}

impl Fact {
    /// The Applicant's values for this fact, taken from the Profile.
    ///
    /// Both of the state's senators answer for the senator question; the rest
    /// take one value. A value the Applicant has not filled in is absent, so a
    /// question needing it accepts nothing.
    fn values(self, profile: &Profile) -> Vec<&str> {
        match self {
            Fact::StateSenators => profile.senators.iter().filter_map(filled_in).collect(),
            Fact::Representative => filled_in(&profile.representative).into_iter().collect(),
            Fact::Governor => filled_in(&profile.governor).into_iter().collect(),
            Fact::StateCapital => filled_in(&profile.state_capital).into_iter().collect(),
            Fact::President => filled_in(&profile.test_updates.president)
                .into_iter()
                .collect(),
            Fact::VicePresident => filled_in(&profile.test_updates.vice_president)
                .into_iter()
                .collect(),
            Fact::SpeakerOfTheHouse => filled_in(&profile.test_updates.speaker_of_the_house)
                .into_iter()
                .collect(),
            Fact::ChiefJustice => filled_in(&profile.test_updates.chief_justice)
                .into_iter()
                .collect(),
        }
    }
}

/// The value behind a Profile field, where the Applicant filled it in.
fn filled_in(field: &Option<String>) -> Option<&str> {
    let value = field.as_deref()?.trim();

    (!value.is_empty()).then_some(value)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Section, Topic, matching};

    /// The question the fixture stands for: one that takes a Local Fact.
    fn question() -> Question {
        Question {
            number: 61,
            section: Section::AmericanGovernment,
            topic: Topic::SystemOfGovernment,
            question: "Who is the governor of your state now?".to_string(),
            accepted_answers: vec![AcceptedAnswer::from("Answers will vary.")],
            answers_required: 1,
            starred: true,
            explanation: None,
        }
    }

    /// A Profile with only the given facts filled in.
    fn profile() -> Profile {
        Profile {
            state: "New Jersey".to_string(),
            governor: Some("Mikie Sherrill".to_string()),
            senators: [
                Some("Cory Booker".to_string()),
                Some("Andy Kim".to_string()),
            ],
            ..Profile::default()
        }
    }

    #[test]
    fn test_answers_resolves_a_local_fact_from_the_profile() {
        // -- Setup & Fixtures
        let question = question();

        // -- Exec
        let accepted = accepted_answers(&question, &profile());

        // -- Check
        assert_eq!(
            accepted,
            vec![AcceptedAnswer::from("Mikie Sherrill")],
            "the published placeholder is not an answer"
        );
        assert_eq!(
            matching::judge(&question, &accepted, "Mikie Sherrill"),
            matching::Verdict::Correct
        );
        assert_eq!(
            matching::judge(&question, &accepted, "the governor"),
            matching::Verdict::Wrong
        );
    }

    #[test]
    fn test_answers_accepts_either_senator() {
        // -- Setup & Fixtures
        let question = Question {
            number: 23,
            accepted_answers: vec![AcceptedAnswer::from("Answers will vary.")],
            ..question()
        };

        // -- Exec
        let accepted = accepted_answers(&question, &profile());

        // -- Check
        assert_eq!(accepted.len(), 2);
        for senator in ["Cory Booker", "Andy Kim"] {
            assert_eq!(
                matching::judge(&question, &accepted, senator),
                matching::Verdict::Correct,
                "{senator} sits in the Senate"
            );
        }
    }

    #[test]
    fn test_answers_accepts_nothing_where_the_profile_is_blank() {
        // -- Setup & Fixtures
        let question = question();

        // -- Exec
        let accepted = accepted_answers(&question, &Profile::default());

        // -- Check
        assert!(accepted.is_empty(), "a missing value is not an answer");
        assert_eq!(
            matching::judge(&question, &accepted, "Mikie Sherrill"),
            matching::Verdict::Wrong,
            "the question stays in play and any answer is judged wrong"
        );
    }

    #[test]
    fn test_answers_leaves_published_answers_alone() {
        // -- Setup & Fixtures
        let question = Question {
            number: 1,
            question: "What is the form of government of the United States?".to_string(),
            accepted_answers: vec![AcceptedAnswer::from("Republic")],
            ..question()
        };

        // -- Exec
        let accepted = accepted_answers(&question, &profile());

        // -- Check
        assert_eq!(accepted, vec![AcceptedAnswer::from("Republic")]);
    }
}

// endregion: --- Tests
