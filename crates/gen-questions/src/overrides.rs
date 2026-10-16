//! The hand-authored corrections to the extracted questions.
//!
//! Everything here is a judgment call about USCIS's own text: the square-bracket
//! annotations that are not optional words, and the two bullets that hold a
//! comma list of several Accepted Answers. Extraction mechanics live in `parse`;
//! this module is where the source text is deliberately rewritten, and the
//! rewritten text is committed in `crates/gen-questions/overrides.json`, so a
//! regeneration never discards a correction.

use domain::{AcceptedAnswer, Question};
use serde::Deserialize;

use crate::{Error, Result};

/// The committed corrections.
#[derive(Debug, Deserialize)]
pub struct Overrides {
    pub questions: Vec<QuestionOverride>,
}

/// The Accepted Answers one Civics Test Question really has.
#[derive(Debug, Deserialize)]
pub struct QuestionOverride {
    /// Why the extracted answers are wrong, for whoever reads the file next.
    pub reason: String,

    pub number: u16,
    pub accepted_answers: Vec<AcceptedAnswer>,
}

/// Reads the committed corrections.
pub fn parse_overrides(json: &str) -> Result<Overrides> {
    let overrides: Overrides = serde_json::from_str(json)?;

    for (index, correction) in overrides.questions.iter().enumerate() {
        let corrected_twice = overrides.questions[..index]
            .iter()
            .any(|earlier| earlier.number == correction.number);

        if corrected_twice {
            return Err(format!("question {} is corrected twice", correction.number).into());
        }
    }

    Ok(overrides)
}

/// Applies the corrections to the extracted questions.
pub fn apply(questions: &mut [Question], overrides: &Overrides) -> Result<()> {
    for correction in &overrides.questions {
        let question = questions
            .iter_mut()
            .find(|question| question.number == correction.number)
            .ok_or_else(|| {
                Error::from(format!(
                    "correction for unknown question {}",
                    correction.number
                ))
            })?;

        question.accepted_answers = correction.accepted_answers.clone();
    }

    Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use domain::{Section, Topic};

    use super::*;

    #[test]
    fn test_overrides_apply_replaces_answers() -> Result<()> {
        // -- Setup & Fixtures
        let mut questions = [question(16, ["Legislative, executive, and judicial"])];
        let overrides = parse_overrides(
            r#"{
                "questions": [
                    {
                        "reason": "one bullet holds a comma list",
                        "number": 16,
                        "accepted_answers": ["Legislative", "executive", "judicial"]
                    }
                ]
            }"#,
        )?;

        // -- Exec
        apply(&mut questions, &overrides)?;

        // -- Check
        assert_eq!(
            questions[0].accepted_answers,
            [
                AcceptedAnswer::from("Legislative"),
                AcceptedAnswer::from("executive"),
                AcceptedAnswer::from("judicial"),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_overrides_apply_unknown_number_errors() -> Result<()> {
        // -- Setup & Fixtures
        let mut questions = [question(16, ["Legislative, executive, and judicial"])];
        let overrides = parse_overrides(
            r#"{
                "questions": [
                    { "reason": "gone", "number": 200, "accepted_answers": ["Nothing"] }
                ]
            }"#,
        )?;

        // -- Exec
        let result = apply(&mut questions, &overrides);

        // -- Check
        assert!(
            result.is_err(),
            "a correction for a question that does not exist must not pass"
        );

        Ok(())
    }

    #[test]
    fn test_overrides_parse_overrides_repeated_number_errors() -> Result<()> {
        // -- Setup & Fixtures
        let json = r#"{
            "questions": [
                { "reason": "first", "number": 16, "accepted_answers": ["One"] },
                { "reason": "second", "number": 16, "accepted_answers": ["Two"] }
            ]
        }"#;

        // -- Exec
        let result = parse_overrides(json);

        // -- Check
        assert!(
            result.is_err(),
            "the same question corrected twice must not pass"
        );

        Ok(())
    }

    #[test]
    fn test_overrides_parse_overrides_missing_reason_errors() -> Result<()> {
        // -- Setup & Fixtures
        let json = r#"{
            "questions": [
                { "number": 16, "accepted_answers": ["One"] }
            ]
        }"#;

        // -- Exec
        let result = parse_overrides(json);

        // -- Check
        assert!(
            result.is_err(),
            "a correction without a reason must not pass"
        );

        Ok(())
    }

    // region:    --- Support

    fn question<const N: usize>(number: u16, answers: [&str; N]) -> Question {
        Question {
            number,
            section: Section::AmericanGovernment,
            topic: Topic::SystemOfGovernment,
            question: "Name the three branches of government.".to_string(),
            accepted_answers: answers
                .iter()
                .map(|answer| AcceptedAnswer::from(*answer))
                .collect(),
            answers_required: 1,
            starred: false,
            explanation: None,
        }
    }

    // endregion: --- Support
}

// endregion: --- Tests
