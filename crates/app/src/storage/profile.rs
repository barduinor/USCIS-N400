//! The Applicant's Profile, as one record.

use domain::Profile;

use crate::Result;
use crate::storage::{self, PROFILE_STORE};

/// The key of the Profile record: there is one.
pub(super) const PROFILE_KEY: &str = "profile";

/// The Profile as this layer stores it: the domain type's own shape (ADR 0007).
pub fn to_record(profile: &Profile) -> Result<String> {
    Ok(serde_json::to_string(profile)?)
}

/// The Profile a stored record holds.
pub fn from_record(record: &str) -> Result<Profile> {
    Ok(serde_json::from_str(record)?)
}

/// Reads the stored Profile, if onboarding has happened on this device.
pub async fn load() -> Result<Option<Profile>> {
    let record = storage::read(PROFILE_STORE, PROFILE_KEY).await?;

    record.map(|record| from_record(&record)).transpose()
}

/// Stores the Profile, replacing whatever was there.
pub async fn save(profile: &Profile) -> Result<()> {
    storage::write(PROFILE_STORE, PROFILE_KEY, &to_record(profile)?).await
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use domain::TestUpdates;

    use super::*;

    #[test]
    fn test_storage_profile_records_round_trip_an_answered_profile() -> Result<()> {
        // -- Setup & Fixtures
        let profile = Profile {
            state: "New Jersey".to_string(),
            district: Some("7th".to_string()),
            senators: [Some("Cory Booker".to_string()), None],
            representative: Some("Thomas Kean Jr.".to_string()),
            governor: None,
            state_capital: Some("Trenton".to_string()),
            test_updates: TestUpdates {
                president: Some("Example President".to_string()),
                vice_president: None,
                speaker_of_the_house: Some("Example Speaker".to_string()),
                chief_justice: None,
            },
            special_consideration_65_20: true,
        };

        // -- Exec
        let stored = to_record(&profile)?;
        let restored = from_record(&stored)?;

        // -- Check
        assert_eq!(restored, profile);

        Ok(())
    }

    #[test]
    fn test_storage_profile_loads_a_profile_stored_before_the_shipped_values() -> Result<()> {
        // -- Setup & Fixtures
        // The record a device already held: the same Profile shape, with the
        // Test Updates the Applicant typed. Nothing about the shipped values is
        // stored, so loading it must leave every typed value untouched.
        let record = r#"{
            "state": "New Jersey",
            "district": "7th",
            "senators": ["Cory Booker", "Andy Kim"],
            "representative": "Thomas Kean Jr.",
            "governor": "Mikie Sherrill",
            "state_capital": "Trenton",
            "test_updates": {
                "president": "Old Stored President",
                "vice_president": "Old Stored Vice President",
                "speaker_of_the_house": "Old Stored Speaker",
                "chief_justice": "Old Stored Chief Justice"
            },
            "special_consideration_65_20": false
        }"#;

        // -- Exec
        let profile = from_record(record)?;

        // -- Check
        assert_eq!(profile.state, "New Jersey");
        assert_eq!(
            profile.test_updates,
            TestUpdates {
                president: Some("Old Stored President".to_string()),
                vice_president: Some("Old Stored Vice President".to_string()),
                speaker_of_the_house: Some("Old Stored Speaker".to_string()),
                chief_justice: Some("Old Stored Chief Justice".to_string()),
            }
        );
        assert_eq!(profile.governor.as_deref(), Some("Mikie Sherrill"));

        Ok(())
    }

    #[test]
    fn test_storage_profile_from_record_rejects_another_shape() {
        // -- Exec
        let result = from_record("{\"state\": 7}");

        // -- Check
        assert!(
            result.is_err(),
            "a record that is not a Profile must not pass"
        );
    }
}

// endregion: --- Tests
