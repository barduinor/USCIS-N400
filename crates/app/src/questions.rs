//! The Civics Test Questions, embedded at compile time.
//!
//! `crates/app/data/questions.json` is generated from M-1778 by `gen-questions`
//! and guarded by that generator's regeneration test and by `domain`'s records
//! contract. Embedding it with `include_str!` makes a missing records file a
//! build failure rather than a runtime surprise, and the parse happens once,
//! before the first screen renders.

use std::sync::LazyLock;

use domain::QuestionRecords;

use crate::{Error, Result};

/// The records as committed by `gen-questions`.
const RECORDS: &str = include_str!("../data/questions.json");

/// The records, parsed once. A failure here is a records file that no longer
/// matches the `domain` types, which the records contract test would have
/// caught first.
static PARSED: LazyLock<core::result::Result<QuestionRecords, String>> =
    LazyLock::new(|| serde_json::from_str(RECORDS).map_err(|error| error.to_string()));

/// The Civics Test Questions.
pub fn records() -> Result<&'static QuestionRecords> {
    match &*PARSED {
        Ok(records) => Ok(records),
        Err(message) => Err(Error::custom(message.as_str())),
    }
}
