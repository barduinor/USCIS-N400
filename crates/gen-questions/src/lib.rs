//! The Civics Test records generator.
//!
//! `gen-questions` is a host tool: it extracts the 2025 question list from the
//! committed USCIS document, applies the committed corrections, and writes the
//! question records the application embeds. Nothing depends on this crate, so
//! its document tooling can never reach the shipped web bundle.

// region:    --- Modules

mod cmd;
mod error;
pub mod generate;
pub mod overrides;
pub mod parse;
pub mod test_updates;

pub use cmd::run;
pub use error::{Error, Result};

// endregion: --- Modules
