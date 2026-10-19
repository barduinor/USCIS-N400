//! The records pipeline: the USCIS question list in, the committed records out.

use domain::QuestionRecords;

use crate::{Result, alternatives, overrides, parse};

/// The committed question list the pipeline reads.
pub const QUESTION_LIST_PATH: &str =
    "docs/resources/2025-Civics-Test-128-Questions-and-Answers-M-1778.pdf";

/// The committed corrections the pipeline applies.
pub const OVERRIDES_PATH: &str = "crates/gen-questions/overrides.json";

/// The committed reviewed forms the pipeline attaches.
pub const ALTERNATIVES_PATH: &str = "crates/gen-questions/alternatives.json";

/// The committed records the pipeline writes.
pub const RECORDS_PATH: &str = "crates/app/data/questions.json";

/// Runs the pipeline over the M-1778 bytes, the committed corrections, and the
/// reviewed forms.
pub fn records_from_pdf(
    pdf_bytes: &[u8],
    overrides_json: &str,
    alternatives_json: &str,
) -> Result<QuestionRecords> {
    let text = pdf_extract::extract_text_from_mem(pdf_bytes)?;
    let mut questions = parse::parse_text(&text)?;
    let corrections = overrides::parse_overrides(overrides_json)?;
    let reviewed = alternatives::parse_alternatives(alternatives_json)?;

    overrides::apply(&mut questions, &corrections)?;

    let mut records = QuestionRecords {
        questions,
        alternatives: Vec::new(),
    };
    alternatives::apply(&mut records, &reviewed)?;

    Ok(records)
}

/// Serializes records exactly as `crates/app/data/questions.json` is committed.
pub fn records_json(records: &QuestionRecords) -> Result<String> {
    let mut json = serde_json::to_string_pretty(records)?;

    json.push('\n');

    Ok(json)
}
