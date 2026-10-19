//! The Civics Test Questions, the Sections and Topics they sit in, and the
//! shape of the committed records.
//!
//! `crates/app/data/questions.json` is a [`QuestionRecords`]: the questions in
//! document order, each with the Accepted Answers the test accepts. The records
//! are generated from M-1778 and corrected by hand (ADR 0002); the pipeline
//! that produces them serializes these types, so the file and the model cannot
//! drift apart.

use core::fmt::{Display, Formatter};

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use crate::Profile;

/// One Civics Test Question with its Accepted Answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// The question's number, 1 to 128, in M-1778's order.
    pub number: u16,

    pub section: Section,
    pub topic: Topic,
    pub question: String,
    pub accepted_answers: Vec<AcceptedAnswer>,

    /// How many distinct Accepted Answers an answer must carry: 2 for
    /// "Name two", and 1 for the rest.
    pub answers_required: u16,

    /// Whether the question is one of the 20 starred questions, which are the
    /// 65/20 Special Consideration's whole scope.
    pub starred: bool,

    /// The Explanation, absent until the study guide's text is mapped onto the
    /// questions.
    pub explanation: Option<String>,
}

/// The committed records file.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionRecords {
    pub questions: Vec<Question>,

    /// Reviewed forms attached to the published answers (ADR 0011).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<QuestionAlternatives>,
}

/// The Alternative Answers one Civics Test Question carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionAlternatives {
    /// The Civics Test Question's number.
    pub number: u16,

    /// The reviewed forms, each attached to one published Accepted Answer.
    pub forms: Vec<AlternativeForm>,
}

/// One reviewed form that counts as a published Accepted Answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeForm {
    /// The published Accepted Answer this form stands for.
    pub answer: AcceptedAnswer,

    /// The reviewed phrasing that counts as that answer.
    pub form: AcceptedAnswer,
}

/// One official answer to a Civics Test Question, as USCIS publishes it.
///
/// Parenthetical words are read by shape and square brackets annotate; both are
/// the matcher's business, not the record's (ADR 0004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, From)]
#[display("{_0}")]
#[serde(transparent)]
pub struct AcceptedAnswer(String);

/// One of the three top-level divisions of the 2025 civics test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Section {
    #[serde(rename = "American Government")]
    AmericanGovernment,

    #[serde(rename = "American History")]
    AmericanHistory,

    #[serde(rename = "Symbols and Holidays")]
    SymbolsAndHolidays,
}

/// One of the eight Topics; each sits inside one Section.
///
/// The names are USCIS's own. The last American History Topic is long enough
/// that the UI may show a shorter label; that is a presentation concern, not a
/// second name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topic {
    // -- American Government
    #[serde(rename = "Principles of American Government")]
    PrinciplesOfAmericanGovernment,

    #[serde(rename = "System of Government")]
    SystemOfGovernment,

    #[serde(rename = "Rights and Responsibilities")]
    RightsAndResponsibilities,

    // -- American History
    #[serde(rename = "Colonial Period and Independence")]
    ColonialPeriodAndIndependence,

    #[serde(rename = "1800s")]
    EighteenHundreds,

    #[serde(rename = "Recent American History and Other Important Historical Information")]
    RecentAmericanHistoryAndOtherImportantHistoricalInformation,

    // -- Symbols and Holidays
    Symbols,
    Holidays,
}

impl QuestionRecords {
    /// The Civics Test Question with this number, where the records hold it.
    pub fn question(&self, number: u16) -> Option<&Question> {
        self.questions
            .iter()
            .find(|question| question.number == number)
    }

    /// The Civics Test Questions the Applicant is accountable for: all of them,
    /// or the 20 starred questions when the 65/20 Special Consideration
    /// applies.
    ///
    /// Study selection, Coverage, and the Exam all apply this one rule, so it
    /// sits with the records.
    pub fn accountable(&self, profile: &Profile) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|question| !profile.special_consideration_65_20 || question.starred)
            .collect()
    }

    /// The reviewed forms attached to question `number`, if any.
    ///
    /// An empty slice means the question has no Alternative Answers; the
    /// published answers alone decide it (ADR 0011).
    pub fn alternatives(&self, number: u16) -> &[AlternativeForm] {
        self.alternatives
            .iter()
            .find(|entry| entry.number == number)
            .map(|entry| entry.forms.as_slice())
            .unwrap_or(&[])
    }
}

impl AcceptedAnswer {
    /// The answer as USCIS publishes it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AcceptedAnswer {
    fn from(answer: &str) -> Self {
        Self(answer.to_string())
    }
}

impl Section {
    /// Every Section, in M-1778's order.
    pub const ALL: [Section; 3] = [
        Section::AmericanGovernment,
        Section::AmericanHistory,
        Section::SymbolsAndHolidays,
    ];

    /// The Section's Topics, in M-1778's order.
    pub fn topics(self) -> &'static [Topic] {
        match self {
            Section::AmericanGovernment => &[
                Topic::PrinciplesOfAmericanGovernment,
                Topic::SystemOfGovernment,
                Topic::RightsAndResponsibilities,
            ],
            Section::AmericanHistory => &[
                Topic::ColonialPeriodAndIndependence,
                Topic::EighteenHundreds,
                Topic::RecentAmericanHistoryAndOtherImportantHistoricalInformation,
            ],
            Section::SymbolsAndHolidays => &[Topic::Symbols, Topic::Holidays],
        }
    }
}

impl Topic {
    /// The Section this Topic sits in.
    pub fn section(self) -> Section {
        match self {
            Topic::PrinciplesOfAmericanGovernment
            | Topic::SystemOfGovernment
            | Topic::RightsAndResponsibilities => Section::AmericanGovernment,
            Topic::ColonialPeriodAndIndependence
            | Topic::EighteenHundreds
            | Topic::RecentAmericanHistoryAndOtherImportantHistoricalInformation => {
                Section::AmericanHistory
            }
            Topic::Symbols | Topic::Holidays => Section::SymbolsAndHolidays,
        }
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Section::AmericanGovernment => "American Government",
            Section::AmericanHistory => "American History",
            Section::SymbolsAndHolidays => "Symbols and Holidays",
        })
    }
}

impl Display for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Topic::PrinciplesOfAmericanGovernment => "Principles of American Government",
            Topic::SystemOfGovernment => "System of Government",
            Topic::RightsAndResponsibilities => "Rights and Responsibilities",
            Topic::ColonialPeriodAndIndependence => "Colonial Period and Independence",
            Topic::EighteenHundreds => "1800s",
            Topic::RecentAmericanHistoryAndOtherImportantHistoricalInformation => {
                "Recent American History and Other Important Historical Information"
            }
            Topic::Symbols => "Symbols",
            Topic::Holidays => "Holidays",
        })
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    #[test]
    fn test_question_section_names_match_the_records() -> Result<()> {
        // -- Exec & Check
        for section in Section::ALL {
            let serialized = serde_json::to_string(&section)?;

            assert_eq!(
                serialized,
                format!("\"{section}\""),
                "a Section's display name and its record name must agree"
            );
        }

        Ok(())
    }

    #[test]
    fn test_question_topic_names_match_the_records() -> Result<()> {
        // -- Exec & Check
        for topic in Section::ALL.iter().flat_map(|section| section.topics()) {
            let serialized = serde_json::to_string(topic)?;

            assert_eq!(
                serialized,
                format!("\"{topic}\""),
                "a Topic's display name and its record name must agree"
            );
        }

        Ok(())
    }

    #[test]
    fn test_question_records_accountable_narrows_to_the_starred_scope() {
        // -- Setup & Fixtures
        let records = QuestionRecords {
            questions: (1..=3)
                .map(|number| Question {
                    number,
                    section: Section::AmericanGovernment,
                    topic: Topic::PrinciplesOfAmericanGovernment,
                    question: "Fixture.".to_string(),
                    accepted_answers: vec![AcceptedAnswer::from("Republic")],
                    answers_required: 1,
                    starred: number == 2,
                    explanation: None,
                })
                .collect(),
            alternatives: Vec::new(),
        };
        let reduced = Profile {
            special_consideration_65_20: true,
            ..Profile::default()
        };

        // -- Exec
        let all = records.accountable(&Profile::default());
        let accountable = records.accountable(&reduced);

        // -- Check
        assert_eq!(all.len(), 3);
        assert_eq!(accountable.len(), 1, "65/20 studies the starred questions");
        assert!(
            accountable.iter().all(|question| question.starred),
            "nothing outside the starred scope is ever asked"
        );
        assert_eq!(records.question(2).map(|question| question.number), Some(2));
        assert_eq!(records.question(99).map(|question| question.number), None);
    }

    #[test]
    fn test_question_records_alternatives_returns_the_questions_forms() {
        // -- Setup & Fixtures
        let records = QuestionRecords {
            questions: Vec::new(),
            alternatives: vec![QuestionAlternatives {
                number: 4,
                forms: vec![AlternativeForm {
                    answer: AcceptedAnswer::from("Self-government"),
                    form: AcceptedAnswer::from("Self governance"),
                }],
            }],
        };

        // -- Exec & Check
        assert_eq!(records.alternatives(4).len(), 1);
        assert_eq!(records.alternatives(4)[0].form.as_str(), "Self governance");
        assert!(
            records.alternatives(5).is_empty(),
            "a question with no forms gets an empty slice"
        );
    }

    #[test]
    fn test_question_sections_hold_their_topics() {
        // -- Exec & Check
        let topics: Vec<Topic> = Section::ALL
            .iter()
            .flat_map(|section| section.topics().iter().copied())
            .collect();

        assert_eq!(
            topics.len(),
            8,
            "the 2025 test has eight Topics across its three Sections"
        );

        for section in Section::ALL {
            for topic in section.topics() {
                assert_eq!(
                    topic.section(),
                    section,
                    "topic {topic} must belong to the Section that lists it"
                );
            }
        }
    }

    #[test]
    fn test_question_accepted_answer_keeps_its_text() {
        // -- Setup & Fixtures
        let answer = AcceptedAnswer::from("(U.S.) Constitution".to_string());

        // -- Exec & Check
        assert_eq!(answer.as_str(), "(U.S.) Constitution");
        assert_eq!(answer.to_string(), "(U.S.) Constitution");
    }
}

// endregion: --- Tests
