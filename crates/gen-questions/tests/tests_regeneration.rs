//! The regeneration guard: the pipeline against the committed records.
//!
//! Extractor upgrades, parser changes, and corrections all surface here as a
//! reviewable diff, rather than as a silently different application.

mod test_support;

use gen_questions::generate::{
    ALTERNATIVES_PATH, OVERRIDES_PATH, QUESTION_LIST_PATH, RECORDS_PATH, records_from_pdf,
    records_json,
};
use test_support::{read_committed, read_committed_bytes};

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

#[test]
fn test_regeneration_matches_committed_records() -> Result<()> {
    // -- Setup & Fixtures
    let pdf = read_committed_bytes(QUESTION_LIST_PATH)?;
    let overrides = read_committed(OVERRIDES_PATH)?;
    let alternatives = read_committed(ALTERNATIVES_PATH)?;
    let committed = read_committed(RECORDS_PATH)?;

    // -- Exec
    let regenerated = records_json(&records_from_pdf(&pdf, &overrides, &alternatives)?)?;

    // -- Check
    assert_records_match(&regenerated, &committed);

    Ok(())
}

// region:    --- Support

/// Reports the first line that differs, so a failed guard names the change.
fn assert_records_match(regenerated: &str, committed: &str) {
    if regenerated == committed {
        return;
    }

    let first_difference = regenerated
        .lines()
        .zip(committed.lines())
        .position(|(regenerated, committed)| regenerated != committed);

    match first_difference {
        Some(index) => panic!(
            "regeneration differs from {RECORDS_PATH} at line {}:\n  pipeline:  {}\n  committed: {}\n\
             The pipeline changed, or {RECORDS_PATH} was hand-edited; corrections belong in {OVERRIDES_PATH} and reviewed forms in {ALTERNATIVES_PATH}.",
            index + 1,
            regenerated.lines().nth(index).unwrap_or(""),
            committed.lines().nth(index).unwrap_or(""),
        ),
        None => panic!(
            "regeneration differs from {RECORDS_PATH} in length: {} bytes from the pipeline, {} committed",
            regenerated.len(),
            committed.len()
        ),
    }
}

// endregion: --- Support
