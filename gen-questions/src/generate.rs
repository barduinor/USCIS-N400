//! The records pipeline: the USCIS question list in, the committed records out.

use domain::QuestionRecords;

use crate::{Result, overrides, parse};

/// The committed question list the pipeline reads.
pub const QUESTION_LIST_PATH: &str =
    "docs/resources/2025-Civics-Test-128-Questions-and-Answers-M-1778.pdf";

/// The committed corrections the pipeline applies.
pub const OVERRIDES_PATH: &str = "gen-questions/overrides.json";

/// The committed records the pipeline writes.
pub const RECORDS_PATH: &str = "data/questions.json";

/// Runs the pipeline over the M-1778 bytes and the committed corrections.
pub fn records_from_pdf(pdf_bytes: &[u8], overrides_json: &str) -> Result<QuestionRecords> {
    let text = pdf_extract::extract_text_from_mem(pdf_bytes)?;
    let mut questions = parse::parse_text(&text)?;
    let corrections = overrides::parse_overrides(overrides_json)?;

    overrides::apply(&mut questions, &corrections)?;

    Ok(QuestionRecords { questions })
}

/// Serializes records exactly as `data/questions.json` is committed.
pub fn records_json(records: &QuestionRecords) -> Result<String> {
    let mut json = serde_json::to_string_pretty(records)?;

    json.push('\n');

    Ok(json)
}
