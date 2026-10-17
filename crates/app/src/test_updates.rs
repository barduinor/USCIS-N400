//! The Test Update values, embedded at compile time.
//!
//! `crates/app/data/test-updates.json` is generated from
//! `uscis.gov/citizenship/testupdates` by `gen-questions` and committed with
//! the values, the page they came from, and the day they were fetched. The
//! values arrive with the release; the application never fetches them at
//! runtime, so they are present offline.
//!
//! A blank Test Update field presents its shipped value, while the stored
//! Profile keeps only what the Applicant saved. Judging follows the same rule:
//! a stored value wins, and a shipped value stands in wherever the Applicant
//! stored none.

use std::sync::LazyLock;

use domain::{Profile, TestUpdatesRecord};

use crate::{Error, Result};

/// The record as committed by `gen-questions`.
const RECORD: &str = include_str!("../data/test-updates.json");

/// The record, parsed once. A failure here is a data file that no longer
/// matches the `domain` types, which the generation tests would have caught
/// first.
static PARSED: LazyLock<core::result::Result<TestUpdatesRecord, String>> =
    LazyLock::new(|| serde_json::from_str(RECORD).map_err(|error| error.to_string()));

/// The shipped Test Update record.
pub fn record() -> Result<&'static TestUpdatesRecord> {
    match &*PARSED {
        Ok(record) => Ok(record),
        Err(message) => Err(Error::custom(message.as_str())),
    }
}

/// The Profile the Civics Test Questions are judged against: the stored Profile
/// with every blank Test Update filled from the shipped record.
///
/// The stored Profile is never changed: the shipped values stand in only for
/// the duration of a judgement, so saving still writes only what the Applicant
/// gave.
pub fn effective_profile(profile: &Profile, record: &TestUpdatesRecord) -> Profile {
    profile.with_shipped_test_updates(&record.values)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use domain::{Profile, TestUpdates, Verdict, answers, matching};

    use super::*;
    use crate::questions;

    #[test]
    fn test_test_updates_record_carries_all_four_values() -> Result<()> {
        // -- Exec
        let record = record()?;

        // -- Check
        assert_eq!(
            record.source,
            "https://www.uscis.gov/citizenship/testupdates"
        );
        assert!(
            !record.fetched.is_empty(),
            "the record says when the values were fetched"
        );
        for (field, value) in [
            ("president", &record.values.president),
            ("vice president", &record.values.vice_president),
            ("speaker of the House", &record.values.speaker_of_the_house),
            ("chief justice", &record.values.chief_justice),
        ] {
            assert!(!value.trim().is_empty(), "the {field} must ship filled");
        }

        Ok(())
    }

    /// The four Test Updates judge their shipped values correct on a fresh
    /// Profile, while a Local Fact the Profile has not answered stays wrong.
    #[test]
    fn test_test_updates_shipped_values_judge_their_questions_correct() -> Result<()> {
        // -- Setup & Fixtures
        let records = questions::records()?;
        let shipped = record()?;
        let effective = effective_profile(&Profile::default(), shipped);
        let cases = [
            (38, shipped.values.president.as_str()),
            (39, shipped.values.vice_president.as_str()),
            (30, shipped.values.speaker_of_the_house.as_str()),
            (57, shipped.values.chief_justice.as_str()),
        ];

        // -- Exec & Check
        for (number, value) in cases {
            let question = records
                .question(number)
                .ok_or_else(|| format!("records should hold question {number}"))?;
            let accepted = answers::accepted_answers(question, &effective);

            assert_eq!(
                matching::judge(question, &accepted, value),
                Verdict::Correct,
                "question {number} must judge the shipped `{value}` correct"
            );
        }

        let governor = records
            .question(61)
            .ok_or("records should hold question 61")?;
        assert!(
            answers::accepted_answers(governor, &effective).is_empty(),
            "a missing Local Fact still accepts nothing"
        );

        Ok(())
    }

    #[test]
    fn test_test_updates_a_stored_value_wins_over_the_shipped_one() -> Result<()> {
        // -- Setup & Fixtures
        let profile = Profile {
            test_updates: TestUpdates {
                president: Some("Stored President".to_string()),
                ..TestUpdates::default()
            },
            ..Profile::default()
        };

        // -- Exec
        let effective = effective_profile(&profile, record()?);

        // -- Check
        assert_eq!(
            effective.test_updates.president.as_deref(),
            Some("Stored President")
        );
        assert_eq!(
            effective.test_updates.vice_president.as_deref(),
            Some(record()?.values.vice_president.as_str())
        );
        assert_eq!(
            profile.test_updates.president.as_deref(),
            Some("Stored President"),
            "the stored Profile still holds exactly what the Applicant saved"
        );

        Ok(())
    }
}

// endregion: --- Tests
