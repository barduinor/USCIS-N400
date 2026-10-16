//! The Civics Test model and its rules.
//!
//! `domain` holds the Civics Test Questions, the Accepted Answers, and every
//! rule the application obeys: answer matching, question selection, Cleared and
//! Coverage, the Exam, and the Profile. It carries no I/O and no wasm
//! dependencies, so the rules are tested natively through this crate's public
//! API, and the UI only renders what `domain` decides.

// region:    --- Modules

mod error;
pub mod profile;
pub mod progress;
pub mod question;

pub use error::{Error, Result};
pub use profile::{Profile, TestUpdates};
pub use progress::Coverage;
pub use question::{AcceptedAnswer, Question, QuestionRecords, Section, Topic};

// endregion: --- Modules
