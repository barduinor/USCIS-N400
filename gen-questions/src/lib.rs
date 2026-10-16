//! The Civics Test records generator.
//!
//! `gen-questions` is a host tool: it reads the committed USCIS question list,
//! applies the hand-authored overrides, and writes the committed question
//! records the application embeds. Nothing depends on this crate, so its
//! document tooling can never reach the shipped web bundle.

// region:    --- Modules

mod cmd;
mod error;

pub use cmd::run;
pub use error::{Error, Result};

// endregion: --- Modules
