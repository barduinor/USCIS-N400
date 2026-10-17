//! The committed Test Update record, checkable without the network.
//!
//! The data file is the release's copy of the USCIS page. These tests prove it
//! is complete, names its source, and carries a fetch date; the parser tests in
//! `gen_questions::test_updates` prove how the page becomes this shape.

mod test_support;

use domain::TestUpdatesRecord;
use gen_questions::test_updates::{SOURCE_URL, TEST_UPDATES_PATH};
use test_support::read_committed;

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

#[test]
fn test_test_updates_committed_record_holds_the_four_values() -> Result<()> {
    // -- Setup & Fixtures
    let record: TestUpdatesRecord = serde_json::from_str(&read_committed(TEST_UPDATES_PATH)?)?;

    // -- Exec & Check
    assert_eq!(
        record.source, SOURCE_URL,
        "the record names the page every value came from"
    );
    assert!(
        is_iso_date(&record.fetched),
        "the record says when the values were fetched: {}",
        record.fetched
    );

    for (field, value) in [
        ("president", &record.values.president),
        ("vice president", &record.values.vice_president),
        ("speaker of the House", &record.values.speaker_of_the_house),
        ("chief justice", &record.values.chief_justice),
    ] {
        assert!(!value.trim().is_empty(), "the {field} must ship filled");
        assert!(
            !value.contains(['<', '>']) && !value.contains("&nbsp;"),
            "the {field} still carries markup: {value}"
        );
        assert!(
            !value.to_ascii_lowercase().contains("answers will vary"),
            "the {field} is still a placeholder: {value}"
        );
        assert!(
            !value.to_ascii_lowercase().contains("testupdates"),
            "the {field} still points at the page instead of naming the officeholder: {value}"
        );
    }

    Ok(())
}

/// Whether the text is a `YYYY-MM-DD` date with a real month and day.
fn is_iso_date(text: &str) -> bool {
    let bytes = text.as_bytes();

    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }

    let month = text[5..7].parse::<u8>().unwrap_or(0);
    let day = text[8..10].parse::<u8>().unwrap_or(0);

    (1..=12).contains(&month) && (1..=31).contains(&day)
}
