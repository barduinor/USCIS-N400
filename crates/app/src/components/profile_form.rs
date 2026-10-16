use dioxus::prelude::*;

use domain::{Profile, TestUpdates};

use crate::components::Field;

/// The Applicant's own facts, as a form.
///
/// Onboarding captures it and the Profile screen edits it: the same fields in
/// the same order. The screen owns the Profile it edits; this component writes
/// into it. Every field except the state can wait, and a blank Local Fact or
/// Test Update stays *missing* rather than becoming an empty answer.
#[component]
pub fn ProfileForm(profile: Signal<Profile>) -> Element {
    let mut answers = profile;

    rsx! {
        Field {
            label: "State",
            name: "state",
            value: answers.read().state.clone(),
            on_input: move |value: String| answers.write().state = value,
        }
        Field {
            label: "Congressional district",
            name: "district",
            value: answers.read().district.clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().district = typed(&value),
        }
        Field {
            label: "U.S. senator",
            name: "senator-one",
            value: answers.read().senators[0].clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().senators[0] = typed(&value),
        }
        Field {
            label: "Other U.S. senator",
            name: "senator-two",
            value: answers.read().senators[1].clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().senators[1] = typed(&value),
        }
        Field {
            label: "U.S. representative",
            name: "representative",
            value: answers.read().representative.clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().representative = typed(&value),
        }
        Field {
            label: "Governor",
            name: "governor",
            value: answers.read().governor.clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().governor = typed(&value),
        }
        Field {
            label: "State capital",
            name: "state-capital",
            value: answers.read().state_capital.clone().unwrap_or_default(),
            on_input: move |value: String| answers.write().state_capital = typed(&value),
        }

        fieldset { class: "group",
            legend { "Test Updates" }
            Field {
                label: "President",
                name: "president",
                value: answers.read().test_updates.president.clone().unwrap_or_default(),
                on_input: move |value: String| answers.write().test_updates.president = typed(&value),
            }
            Field {
                label: "Vice President",
                name: "vice-president",
                value: answers.read().test_updates.vice_president.clone().unwrap_or_default(),
                on_input: move |value: String| answers.write().test_updates.vice_president = typed(&value),
            }
            Field {
                label: "Speaker of the House",
                name: "speaker-of-the-house",
                value: answers.read().test_updates.speaker_of_the_house.clone().unwrap_or_default(),
                on_input: move |value: String| {
                    answers.write().test_updates.speaker_of_the_house = typed(&value);
                },
            }
            Field {
                label: "Chief Justice",
                name: "chief-justice",
                value: answers.read().test_updates.chief_justice.clone().unwrap_or_default(),
                on_input: move |value: String| answers.write().test_updates.chief_justice = typed(&value),
            }
        }

        label { class: "check", r#for: "special-consideration",
            input {
                id: "special-consideration",
                r#type: "checkbox",
                checked: answers.read().special_consideration_65_20,
                onchange: move |event| answers.write().special_consideration_65_20 = event.checked(),
            }
            span { "65/20 Special Consideration" }
        }
        p { class: "hint",
            "Aged 65 or older, and a lawful permanent resident for 20 or more years."
        }
    }
}

/// What a field's text means while the Applicant is typing: blank is missing,
/// and anything else is kept exactly as typed, so a space mid-word survives the
/// keystroke that typed it.
fn typed(text: &str) -> Option<String> {
    (!text.trim().is_empty()).then(|| text.to_string())
}

/// The Profile these answers describe: every field trimmed, every blank one left
/// missing.
///
/// This is the boundary between the form and the Profile: a blank Local Fact or
/// Test Update is a *missing* value that leaves its question in play, which is
/// why blank never becomes an empty answer.
pub fn as_profile(answers: &Profile) -> Profile {
    let trimmed = |value: &Option<String>| -> Option<String> {
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };

    Profile {
        state: answers.state.trim().to_string(),
        district: trimmed(&answers.district),
        senators: [trimmed(&answers.senators[0]), trimmed(&answers.senators[1])],
        representative: trimmed(&answers.representative),
        governor: trimmed(&answers.governor),
        state_capital: trimmed(&answers.state_capital),
        test_updates: TestUpdates {
            president: trimmed(&answers.test_updates.president),
            vice_president: trimmed(&answers.test_updates.vice_president),
            speaker_of_the_house: trimmed(&answers.test_updates.speaker_of_the_house),
            chief_justice: trimmed(&answers.test_updates.chief_justice),
        },
        special_consideration_65_20: answers.special_consideration_65_20,
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_form_keeps_a_space_typed_mid_field() {
        // -- Exec & Check
        assert_eq!(typed("New "), Some("New ".to_string()));
        assert_eq!(typed("   "), None);
        assert_eq!(typed(""), None);
    }

    #[test]
    fn test_profile_form_as_profile_trims_and_keeps_blanks_missing() {
        // -- Setup & Fixtures
        let answers = Profile {
            state: "  New Jersey  ".to_string(),
            senators: [Some("Cory Booker ".to_string()), None],
            governor: Some("   ".to_string()),
            test_updates: TestUpdates {
                president: Some("  Example President".to_string()),
                ..TestUpdates::default()
            },
            special_consideration_65_20: true,
            ..Profile::default()
        };

        // -- Exec
        let profile = as_profile(&answers);

        // -- Check
        assert_eq!(profile.state, "New Jersey");
        assert_eq!(profile.senators[0], Some("Cory Booker".to_string()));
        assert_eq!(profile.senators[1], None);
        assert_eq!(profile.governor, None, "a blank field stays missing");
        assert_eq!(
            profile.test_updates.president,
            Some("Example President".to_string())
        );
        assert!(
            profile.special_consideration_65_20,
            "the form's flag reaches the Profile"
        );
    }
}

// endregion: --- Tests
