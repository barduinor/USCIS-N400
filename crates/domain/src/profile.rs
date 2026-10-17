//! The Applicant's own facts, which the Local Fact and Test Update questions
//! take their answers from.
//!
//! Onboarding establishes the Profile before the first Civics Test Question,
//! and the Profile screen edits it later. A missing value leaves its question
//! in play, judged wrong, until the Applicant fills it in.

use serde::{Deserialize, Serialize};

/// The Applicant's own facts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// The state the Applicant lives in: the one required field, because it
    /// decides which Local Facts the test expects.
    pub state: String,

    /// The congressional district of the Applicant's U.S. representative seat.
    pub district: Option<String>,

    /// The state's two U.S. senators.
    pub senators: [Option<String>; 2],

    /// The U.S. representative for the Applicant's district.
    pub representative: Option<String>,

    /// The state's governor.
    pub governor: Option<String>,

    /// The state capital.
    pub state_capital: Option<String>,

    /// The four officeholders whose names change with elections and
    /// appointments, as USCIS publishes them.
    pub test_updates: TestUpdates,

    /// Whether the 65/20 Special Consideration applies: study narrows to the 20
    /// starred questions, and the Exam runs the ten-question administration.
    pub special_consideration_65_20: bool,
}

/// The four Test Updates USCIS publishes at uscis.gov/citizenship/testupdates.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestUpdates {
    pub president: Option<String>,
    pub vice_president: Option<String>,
    pub speaker_of_the_house: Option<String>,
    pub chief_justice: Option<String>,
}

/// The Test Updates as the committed shipped data carries them: every value
/// present, because the generation step refuses a partial set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestUpdateValues {
    pub president: String,
    pub vice_president: String,
    pub speaker_of_the_house: String,
    pub chief_justice: String,
}

/// The committed Test Update data: the four values, the page they came from,
/// and the day they were fetched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestUpdatesRecord {
    /// The page the values were fetched from.
    pub source: String,

    /// The day the values were fetched, as `YYYY-MM-DD`.
    pub fetched: String,

    pub values: TestUpdateValues,
}

impl Profile {
    /// Whether the Applicant has given the one required fact.
    ///
    /// Without a state there is nothing to narrow the Local Facts to, so no
    /// screen that depends on the Profile may render.
    pub fn has_state(&self) -> bool {
        !self.state.trim().is_empty()
    }

    /// The Profile the Test Update questions are judged against: the Applicant's
    /// own values, with the shipped value standing in wherever they stored none.
    ///
    /// This never changes what is stored: the shipped values fill gaps at
    /// judging time, and the Profile keeps only what the Applicant saved.
    pub fn with_shipped_test_updates(&self, shipped: &TestUpdateValues) -> Profile {
        Profile {
            test_updates: TestUpdates {
                president: stored_or_shipped(&self.test_updates.president, &shipped.president),
                vice_president: stored_or_shipped(
                    &self.test_updates.vice_president,
                    &shipped.vice_president,
                ),
                speaker_of_the_house: stored_or_shipped(
                    &self.test_updates.speaker_of_the_house,
                    &shipped.speaker_of_the_house,
                ),
                chief_justice: stored_or_shipped(
                    &self.test_updates.chief_justice,
                    &shipped.chief_justice,
                ),
            },
            ..self.clone()
        }
    }
}

/// The Applicant's value where they gave one, and the shipped value otherwise.
fn stored_or_shipped(stored: &Option<String>, shipped: &str) -> Option<String> {
    stored
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| Some(shipped.to_string()))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    #[test]
    fn test_profile_has_state_needs_the_one_required_fact() {
        // -- Setup & Fixtures
        let answered = Profile {
            state: " New Jersey ".to_string(),
            ..Profile::default()
        };

        // -- Exec & Check
        assert!(answered.has_state());
        assert!(!Profile::default().has_state());
        assert!(
            !Profile {
                state: "   ".to_string(),
                ..Profile::default()
            }
            .has_state()
        );
    }

    #[test]
    fn test_profile_with_shipped_test_updates_fills_only_the_blanks() {
        // -- Setup & Fixtures
        let profile = Profile {
            test_updates: TestUpdates {
                president: Some("Applicant's President".to_string()),
                vice_president: None,
                speaker_of_the_house: Some("   ".to_string()),
                chief_justice: Some("Applicant's Chief Justice".to_string()),
            },
            ..Profile::default()
        };
        let shipped = TestUpdateValues {
            president: "Shipped President".to_string(),
            vice_president: "Shipped Vice President".to_string(),
            speaker_of_the_house: "Shipped Speaker".to_string(),
            chief_justice: "Shipped Chief Justice".to_string(),
        };

        // -- Exec
        let effective = profile.with_shipped_test_updates(&shipped);

        // -- Check
        assert_eq!(
            effective.test_updates.president,
            Some("Applicant's President".to_string()),
            "a stored value wins"
        );
        assert_eq!(
            effective.test_updates.vice_president,
            Some("Shipped Vice President".to_string()),
            "a missing value takes the shipped one"
        );
        assert_eq!(
            effective.test_updates.speaker_of_the_house,
            Some("Shipped Speaker".to_string()),
            "a blank value is missing, not an answer"
        );
        assert_eq!(
            effective.test_updates.chief_justice,
            Some("Applicant's Chief Justice".to_string()),
            "a stored value wins even beside a shipped one"
        );
        assert_eq!(
            profile.test_updates.vice_president, None,
            "filling the effective Profile never writes behind the Applicant"
        );
    }

    #[test]
    fn test_profile_serde_round_trips_a_filled_profile() -> Result<()> {
        // -- Setup & Fixtures
        let profile = Profile {
            state: "New Jersey".to_string(),
            district: Some("7th".to_string()),
            senators: [
                Some("Cory Booker".to_string()),
                Some("Andy Kim".to_string()),
            ],
            representative: Some("Thomas Kean Jr.".to_string()),
            governor: Some("Mikie Sherrill".to_string()),
            state_capital: Some("Trenton".to_string()),
            test_updates: TestUpdates {
                president: Some("Example President".to_string()),
                vice_president: Some("Example Vice President".to_string()),
                speaker_of_the_house: Some("Example Speaker".to_string()),
                chief_justice: Some("Example Chief Justice".to_string()),
            },
            special_consideration_65_20: true,
        };

        // -- Exec
        let json = serde_json::to_string(&profile)?;
        let restored: Profile = serde_json::from_str(&json)?;

        // -- Check
        assert_eq!(restored, profile);

        Ok(())
    }
}

// endregion: --- Tests
