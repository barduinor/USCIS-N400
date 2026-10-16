//! The records contract: the committed records parse into the domain types.
//!
//! This sits outside the rule seams on purpose. It guards the artifact the
//! application embeds — 128 Civics Test Questions, 20 of them starred — so a
//! bad regeneration fails here rather than in the browser.

use std::path::{Path, PathBuf};

use domain::{QuestionRecords, Section};

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

/// The committed records, relative to the workspace root.
const RECORDS: &str = "crates/app/data/questions.json";

/// The 2025 test's size, and the 65/20 Special Consideration's scope.
const QUESTIONS: usize = 128;
const STARRED: usize = 20;

/// The 2025 test's Topics, across its three Sections.
const TOPICS: usize = 8;

#[test]
fn test_questions_data_parses_the_committed_records() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let parsed = records.questions.len();

    // -- Check
    assert_eq!(
        parsed, QUESTIONS,
        "the committed records must hold the 128 Civics Test Questions"
    );

    Ok(())
}

#[test]
fn test_questions_data_starred_questions_are_the_65_20_scope() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let starred = records.questions.iter().filter(|q| q.starred).count();

    // -- Check
    assert_eq!(
        starred, STARRED,
        "exactly 20 questions carry the asterisk the 65/20 Special Consideration studies"
    );

    Ok(())
}

#[test]
fn test_questions_data_topics_sit_in_their_sections() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec & Check
    let mut sections: Vec<Section> = Vec::new();
    let mut topics = Vec::new();

    for question in &records.questions {
        if !sections.contains(&question.section) {
            sections.push(question.section);
        }
        if !topics.contains(&question.topic) {
            topics.push(question.topic);
        }

        assert_eq!(
            question.topic.section(),
            question.section,
            "question {} is filed under a Topic from another Section",
            question.number
        );
    }

    assert_eq!(
        sections,
        Section::ALL,
        "the records must cover the three Sections, in the test's order"
    );
    assert_eq!(
        topics.len(),
        TOPICS,
        "the records must cover the eight Topics"
    );

    Ok(())
}

// region:    --- Support

fn committed_records() -> Result<QuestionRecords> {
    let path = workspace_root().join(RECORDS);
    let json = std::fs::read_to_string(path)?;

    Ok(serde_json::from_str(&json)?)
}

/// The workspace root: this crate's manifest directory, two levels up.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// endregion: --- Support
