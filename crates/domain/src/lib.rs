//! The Civics Test model and its rules.
//!
//! `domain` holds the Civics Test Questions, the Accepted Answers, and every
//! rule the application obeys: answer matching, question selection, Cleared and
//! Coverage, the Exam, and the Profile. It carries no I/O and no wasm
//! dependencies, so the rules are tested natively through this crate's public
//! API, and the UI only renders what `domain` decides.

// region:    --- Modules

pub mod answers;
mod error;
pub mod exam;
pub mod matching;
pub mod profile;
pub mod progress;
pub mod question;
pub mod selection;

pub use error::{Error, Result};
pub use exam::{Exam, ExamRecord, Outcome, Track};
pub use matching::Verdict;
pub use profile::{Profile, TestUpdates};
pub use progress::{AnswerLogEntry, Coverage};
pub use question::{AcceptedAnswer, Question, QuestionRecords, Section, Topic};

// endregion: --- Modules
