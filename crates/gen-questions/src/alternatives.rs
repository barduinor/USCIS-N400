//! The reviewed Alternative Answers attached to published answers.
//!
//! These are not corrections to USCIS's text: they are reviewed phrasings that
//! count as a published Accepted Answer for this Applicant (ADR 0011). The
//! file is committed so a regeneration never discards them, and the generator
//! emits them into the question records.

use domain::{AcceptedAnswer, AlternativeForm, QuestionAlternatives, QuestionRecords};
use serde::Deserialize;

use crate::{Error, Result};

/// The committed reviewed forms.
#[derive(Debug, Deserialize)]
pub struct Alternatives {
    pub questions: Vec<QuestionAlternativesInput>,
}

/// The reviewed forms one Civics Test Question carries.
#[derive(Debug, Deserialize)]
pub struct QuestionAlternativesInput {
    /// Why these forms attach to this answer, for whoever reads the file next.
    pub reason: String,

    pub number: u16,
    pub forms: Vec<AlternativeFormInput>,
}

/// One reviewed form and the published answer it stands for.
#[derive(Debug, Deserialize)]
pub struct AlternativeFormInput {
    /// The published Accepted Answer this form stands for.
    pub answer: String,

    /// The reviewed phrasing that counts as that answer.
    pub form: String,
}

/// Reads the committed reviewed forms.
pub fn parse_alternatives(json: &str) -> Result<Alternatives> {
    let alternatives: Alternatives = serde_json::from_str(json)?;

    for (index, entry) in alternatives.questions.iter().enumerate() {
        if entry.reason.trim().is_empty() {
            return Err(format!("question {} has no review reason", entry.number).into());
        }
        if entry.forms.is_empty() {
            return Err(format!("question {} lists no reviewed forms", entry.number).into());
        }
        if alternatives.questions[..index]
            .iter()
            .any(|previous| previous.number == entry.number)
        {
            return Err(format!("question {} is reviewed twice", entry.number).into());
        }
    }

    Ok(alternatives)
}

/// Attaches the reviewed forms to the questions they belong to.
pub fn apply(records: &mut QuestionRecords, alternatives: &Alternatives) -> Result<()> {
    let mut attached = Vec::new();

    for entry in &alternatives.questions {
        let question = records
            .questions
            .iter()
            .find(|question| question.number == entry.number)
            .ok_or_else(|| {
                Error::from(format!(
                    "reviewed forms for unknown question {}",
                    entry.number
                ))
            })?;

        if domain::answers::is_fact(entry.number) {
            return Err(format!(
                "question {} takes its answer from the Profile; alternatives do not attach to it",
                entry.number
            )
            .into());
        }

        let mut forms = Vec::new();

        for form in &entry.forms {
            if form.answer.trim().is_empty() || form.form.trim().is_empty() {
                return Err(format!(
                    "question {} has an empty reviewed answer or form",
                    entry.number
                )
                .into());
            }

            if !question
                .accepted_answers
                .iter()
                .any(|answer| answer.as_str() == form.answer)
            {
                return Err(format!(
                    "question {} does not publish answer {:?}",
                    entry.number, form.answer
                )
                .into());
            }

            if forms
                .iter()
                .any(|existing: &AlternativeForm| existing.form.as_str() == form.form)
            {
                return Err(format!(
                    "question {} repeats reviewed form {:?}",
                    entry.number, form.form
                )
                .into());
            }

            forms.push(AlternativeForm {
                answer: AcceptedAnswer::from(form.answer.as_str()),
                form: AcceptedAnswer::from(form.form.as_str()),
            });
        }

        attached.push(QuestionAlternatives {
            number: entry.number,
            forms,
        });
    }

    records.alternatives = attached;

    Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Question, Section, Topic};

    fn records() -> QuestionRecords {
        QuestionRecords {
            questions: vec![Question {
                number: 4,
                section: Section::AmericanGovernment,
                topic: Topic::PrinciplesOfAmericanGovernment,
                question: "Fixture.".to_string(),
                accepted_answers: vec![AcceptedAnswer::from("Self-government")],
                answers_required: 1,
                starred: false,
                explanation: None,
            }],
            alternatives: Vec::new(),
        }
    }

    fn alternatives(number: u16, answer: &str, form: &str) -> Alternatives {
        Alternatives {
            questions: vec![QuestionAlternativesInput {
                reason: "Fixture.".to_string(),
                number,
                forms: vec![AlternativeFormInput {
                    answer: answer.to_string(),
                    form: form.to_string(),
                }],
            }],
        }
    }

    #[test]
    fn test_alternatives_apply_attaches_a_reviewed_form() -> Result<()> {
        // -- Setup & Fixtures
        let mut records = records();

        // -- Exec
        apply(
            &mut records,
            &alternatives(4, "Self-government", "Self governance"),
        )?;

        // -- Check
        assert_eq!(records.alternatives.len(), 1);
        assert_eq!(records.alternatives(4)[0].form.as_str(), "Self governance");

        Ok(())
    }

    #[test]
    fn test_alternatives_apply_rejects_an_unpublished_answer() {
        // -- Setup & Fixtures
        let mut records = records();

        // -- Exec
        let result = apply(
            &mut records,
            &alternatives(4, "Popular sovereignty", "Self governance"),
        );

        // -- Check
        assert!(
            result.is_err(),
            "the form must attach to a published answer"
        );
    }

    #[test]
    fn test_alternatives_apply_rejects_a_profile_backed_question() {
        // -- Setup & Fixtures
        let mut records = records();

        // -- Exec
        let result = apply(
            &mut records,
            &alternatives(61, "Answers will vary.", "Georgia"),
        );

        // -- Check
        assert!(
            result.is_err(),
            "a Profile-backed question has no static answer to attach to"
        );
    }

    #[test]
    fn test_alternatives_apply_rejects_a_repeated_form() {
        // -- Setup & Fixtures
        let mut records = records();
        let alternatives = Alternatives {
            questions: vec![QuestionAlternativesInput {
                reason: "Fixture.".to_string(),
                number: 4,
                forms: vec![
                    AlternativeFormInput {
                        answer: "Self-government".to_string(),
                        form: "Self governance".to_string(),
                    },
                    AlternativeFormInput {
                        answer: "Self-government".to_string(),
                        form: "Self governance".to_string(),
                    },
                ],
            }],
        };

        // -- Exec
        let result = apply(&mut records, &alternatives);

        // -- Check
        assert!(
            result.is_err(),
            "a question must not repeat a reviewed form"
        );
    }

    #[test]
    fn test_alternatives_parse_rejects_a_reviewed_twice_question() {
        // -- Setup & Fixtures
        let json = r#"{
            "questions": [
                {
                    "reason": "first",
                    "number": 4,
                    "forms": [{"answer": "Self-government", "form": "Self governance"}]
                },
                {
                    "reason": "second",
                    "number": 4,
                    "forms": [{"answer": "Self-government", "form": "Self rule"}]
                }
            ]
        }"#;

        // -- Exec
        let result = parse_alternatives(json);

        // -- Check
        assert!(result.is_err(), "one question is reviewed once");
    }
}

// endregion: --- Tests
