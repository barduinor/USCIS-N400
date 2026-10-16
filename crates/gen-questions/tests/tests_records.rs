//! The committed records, against M-1778 and the documented structure.
//!
//! The regeneration guard proves `crates/app/data/questions.json` is what the
//! pipeline produces; these tests prove what that file says.

mod test_support;

use domain::{Question, QuestionRecords};
use gen_questions::generate::RECORDS_PATH;
use test_support::read_committed;

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

#[test]
fn test_records_questions_are_the_128_in_order() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let numbers: Vec<u16> = records.questions.iter().map(|q| q.number).collect();
    let expected: Vec<u16> = (1..=128).collect();

    // -- Check
    assert_eq!(numbers, expected);

    Ok(())
}

#[test]
fn test_records_starred_questions_are_the_65_20_set() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let starred: Vec<u16> = records
        .questions
        .iter()
        .filter(|q| q.starred)
        .map(|q| q.number)
        .collect();

    // -- Check
    assert_eq!(
        starred,
        [
            2, 7, 12, 20, 30, 36, 38, 39, 44, 52, 61, 66, 74, 78, 86, 94, 113, 115, 121, 126
        ]
    );

    Ok(())
}

#[test]
fn test_records_carry_no_page_furniture() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let furnished: Vec<(u16, &str)> = records
        .questions
        .iter()
        .flat_map(|q| {
            std::iter::once((q.number, q.question.as_str())).chain(
                q.accepted_answers
                    .iter()
                    .map(|answer| (q.number, answer.as_str())),
            )
        })
        .filter(|(_, text)| {
            text.contains(" of 19 ") || text.trim_end().ends_with("uscis.gov/citizenship")
        })
        .collect();

    // -- Check
    assert!(
        furnished.is_empty(),
        "no record may carry page furniture: {furnished:?}"
    );

    Ok(())
}

#[test]
fn test_records_annotations_are_not_answers() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let annotating: Vec<u16> = records
        .questions
        .iter()
        .filter(|q| {
            q.accepted_answers
                .iter()
                .any(|answer| answer.contains(['[', ']']))
        })
        .map(|q| q.number)
        .collect();

    // -- Check
    for number in [23, 29, 61, 62] {
        assert_eq!(
            answers_of(&records, number)?,
            ["Answers will vary."],
            "question {number}"
        );
    }
    assert_eq!(
        answers_of(&records, 120)?,
        [
            "New York (Harbor)",
            "Liberty Island",
            "New Jersey",
            "near New York City",
            "on the Hudson (River)",
        ],
        "question 120: its bracket lists further Accepted Answers"
    );
    assert!(
        annotating.is_empty(),
        "no answer may still carry a bracket annotation: {annotating:?}"
    );

    Ok(())
}

#[test]
fn test_records_multi_answer_bullets_are_split() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec & Check
    assert_eq!(
        answers_of(&records, 16)?,
        [
            "Legislative",
            "executive",
            "judicial",
            "Congress",
            "president",
            "the courts",
        ]
    );
    assert_eq!(
        answers_of(&records, 19)?,
        ["Senate", "House (of Representatives)"]
    );

    Ok(())
}

#[test]
fn test_records_ordinals_are_repaired() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let split_ordinals: Vec<&str> = records
        .questions
        .iter()
        .flat_map(|q| q.accepted_answers.iter().map(String::as_str))
        .filter(|answer| contains_split_ordinal(answer))
        .collect();

    // -- Check
    for (number, answer) in [
        (37, "(Because of) the 22nd Amendment"),
        (71, "Required by the (U.S.) Constitution (16th Amendment)"),
        (98, "(With the) 15th Amendment"),
        (102, "(With the) 19th Amendment"),
        (107, "34th president of the United States"),
    ] {
        assert!(
            answers_of(&records, number)?.contains(&answer.to_string()),
            "question {number} should answer {answer:?}"
        );
    }
    assert!(
        split_ordinals.is_empty(),
        "no answer may keep a split ordinal: {split_ordinals:?}"
    );

    Ok(())
}

#[test]
fn test_records_answers_required_match_the_demands() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let multi_answer: Vec<(u16, u16)> = records
        .questions
        .iter()
        .filter(|q| q.answers_required > 1)
        .map(|q| (q.number, q.answers_required))
        .collect();
    let short: Vec<u16> = records
        .questions
        .iter()
        .filter(|q| usize::from(q.answers_required) > q.accepted_answers.len())
        .map(|q| q.number)
        .collect();

    // -- Check
    assert_eq!(
        multi_answer,
        [
            (10, 2),
            (16, 3),
            (19, 2),
            (48, 2),
            (65, 3),
            (67, 2),
            (69, 2),
            (81, 5),
            (126, 3)
        ]
    );
    assert!(
        short.is_empty(),
        "every question must offer at least the answers it demands: {short:?}"
    );

    Ok(())
}

#[test]
fn test_records_structure_is_the_documented_one() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let mut structure: Vec<(&str, &str, u16, u16)> = Vec::new();

    for question in &records.questions {
        let section = question.section.as_str();
        let topic = question.topic.as_str();

        match structure.last_mut() {
            Some((last_section, last_topic, _, last_number))
                if *last_section == section && *last_topic == topic =>
            {
                *last_number = question.number;
            }
            _ => structure.push((section, topic, question.number, question.number)),
        }
    }

    // -- Check
    assert_eq!(
        structure,
        [
            (
                "American Government",
                "Principles of American Government",
                1,
                15
            ),
            ("American Government", "System of Government", 16, 62),
            ("American Government", "Rights and Responsibilities", 63, 72),
            (
                "American History",
                "Colonial Period and Independence",
                73,
                89
            ),
            ("American History", "1800s", 90, 99),
            (
                "American History",
                "Recent American History and Other Important Historical Information",
                100,
                118,
            ),
            ("Symbols and Holidays", "Symbols", 119, 124),
            ("Symbols and Holidays", "Holidays", 125, 128),
        ]
    );

    Ok(())
}

// region:    --- Support

fn committed_records() -> Result<QuestionRecords> {
    Ok(serde_json::from_str(&read_committed(RECORDS_PATH)?)?)
}

fn question_of(records: &QuestionRecords, number: u16) -> Result<&Question> {
    records
        .questions
        .iter()
        .find(|question| question.number == number)
        .ok_or_else(|| format!("records should hold question {number}").into())
}

fn answers_of(records: &QuestionRecords, number: u16) -> Result<Vec<String>> {
    Ok(question_of(records, number)?.accepted_answers.clone())
}

/// The suffix of an ordinal the extraction split onto its own line, mid-answer.
fn contains_split_ordinal(answer: &str) -> bool {
    const SUFFIXES: [&str; 4] = ["st", "nd", "rd", "th"];

    answer
        .split_whitespace()
        .any(|word| SUFFIXES.contains(&word))
}

// endregion: --- Support
