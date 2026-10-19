//! The reviewed-verdict corpus: every wrong verdict from the 2026-10-17
//! export, labelled against the accepted answers it was judged against.
//!
//! The test is the acceptance criterion for ADR 0011: the 62 reviewed false
//! negatives must be accepted, and the 11 true negatives must stay wrong.

use std::path::{Path, PathBuf};

use domain::{AcceptedAnswer, QuestionRecords, Verdict, matching};
use serde::Deserialize;

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

/// The reviewed corpus, relative to the workspace root.
const FIXTURE: &str = "crates/domain/tests/fixtures/wrong_verdict_review.json";

/// The committed records, relative to the workspace root.
const RECORDS: &str = "crates/app/data/questions.json";

#[derive(Debug, Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    question: u16,
    typed: String,
    accepted: Vec<String>,
    overridden: bool,
    label: String,
    expected: String,
    note: String,
}

#[test]
fn test_matching_review_corpus_passes_the_reviewed_cases() -> Result<()> {
    // -- Setup & Fixtures
    let root = workspace_root();
    let fixture: Fixture = serde_json::from_str(&std::fs::read_to_string(root.join(FIXTURE))?)?;
    let records: QuestionRecords =
        serde_json::from_str(&std::fs::read_to_string(root.join(RECORDS))?)?;

    assert_eq!(
        fixture.cases.len(),
        73,
        "the corpus is every wrong verdict in the export"
    );
    assert_eq!(
        fixture
            .cases
            .iter()
            .filter(|case| case.label == "false_negative")
            .count(),
        62,
        "the reviewed false negatives"
    );
    assert_eq!(
        fixture
            .cases
            .iter()
            .filter(|case| case.label == "true_negative")
            .count(),
        11,
        "the control set"
    );

    // -- Exec & Check
    for case in &fixture.cases {
        let question = records
            .question(case.question)
            .ok_or_else(|| format!("the records lack question {}", case.question))?;
        let accepted = case
            .accepted
            .iter()
            .map(|answer| AcceptedAnswer::from(answer.as_str()))
            .collect::<Vec<_>>();
        let judged = matching::judge_reviewed(
            question,
            &accepted,
            records.alternatives(case.question),
            &case.typed,
        );
        let expected = match case.expected.as_str() {
            "correct" => Verdict::Correct,
            "wrong" => Verdict::Wrong,
            other => {
                return Err(format!(
                    "question {} has unknown expectation {other:?}",
                    case.question
                )
                .into());
            }
        };

        assert_eq!(
            judged, expected,
            "Q{} `{}` (overridden: {}, {})",
            case.question, case.typed, case.overridden, case.note
        );
    }

    Ok(())
}

/// The workspace root: this crate's manifest directory, two levels up.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
