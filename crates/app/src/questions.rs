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

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use domain::{Profile, Verdict, answers, matching};

    use super::*;

    /// The Study screen's judgement step, over the embedded records: the
    /// question's Accepted Answers for the Applicant's Profile, then the
    /// matcher. Q7 is the amendments question; the fragments are the answers
    /// the rule has to keep wrong.
    #[test]
    fn test_questions_judges_the_amendments_question_and_the_fragments() -> Result<()> {
        // -- Setup & Fixtures
        let records = records()?;
        let profile = Profile::default();
        let cases = [
            (7, "27", Verdict::Correct),
            (28, "Connecticut Compromise", Verdict::Correct),
            (2, "us", Verdict::Wrong),
            (104, "1929", Verdict::Wrong),
            (28, "for small states", Verdict::Wrong),
        ];

        // -- Exec & Check
        for (number, typed, verdict) in cases {
            let question = records
                .question(number)
                .ok_or_else(|| format!("records should hold question {number}"))?;
            let accepted = answers::accepted_answers(question, &profile);

            assert_eq!(
                matching::judge(question, &accepted, typed),
                verdict,
                "question {number} must judge `{typed}` {verdict}"
            );
        }

        Ok(())
    }
}

// endregion: --- Tests
