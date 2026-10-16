//! The Civics Test Questions and the shape of the committed records.
//!
//! `data/questions.json` is a [`QuestionRecords`]: the questions in document
//! order, each with the Accepted Answers the test accepts. The records are
//! generated from M-1778 and corrected by hand (ADR 0002); the pipeline that
//! produces them serializes these types, so the file and the model cannot drift
//! apart.

use serde::{Deserialize, Serialize};

/// One Civics Test Question with its Accepted Answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// The question's number, 1 to 128, in M-1778's order.
    pub number: u16,

    pub section: String,
    pub topic: String,
    pub question: String,
    pub accepted_answers: Vec<String>,

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
}
