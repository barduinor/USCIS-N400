use dioxus::prelude::*;

use domain::{Profile, TestUpdates};

use crate::components::Field;
use crate::storage::profile;

/// What the onboarding form holds while the Applicant fills it in.
///
/// Every field starts blank, and a blank Local Fact or Test Update stays
/// missing.
#[derive(Debug, Clone, Default, PartialEq)]
struct Form {
    state: String,
    district: String,
    senator_one: String,
    senator_two: String,
    representative: String,
    governor: String,
    state_capital: String,
    president: String,
    vice_president: String,
    speaker_of_the_house: String,
    chief_justice: String,
    special_consideration_65_20: bool,
}

impl Form {
    /// The Profile these answers describe.
    fn into_profile(self) -> Profile {
        Profile {
            state: self.state.trim().to_string(),
            district: missing_if_blank(&self.district),
            senators: [
                missing_if_blank(&self.senator_one),
                missing_if_blank(&self.senator_two),
            ],
            representative: missing_if_blank(&self.representative),
            governor: missing_if_blank(&self.governor),
            state_capital: missing_if_blank(&self.state_capital),
            test_updates: TestUpdates {
                president: missing_if_blank(&self.president),
                vice_president: missing_if_blank(&self.vice_president),
                speaker_of_the_house: missing_if_blank(&self.speaker_of_the_house),
                chief_justice: missing_if_blank(&self.chief_justice),
            },
            special_consideration_65_20: self.special_consideration_65_20,
        }
    }
}

/// The first run: the Applicant's own facts, before any Civics Test Question.
#[component]
pub fn Onboarding(on_saved: EventHandler<()>) -> Element {
    let mut form = use_signal(Form::default);
    let mut problem = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);

    let save = move |_| {
        let profile = form().into_profile();

        if !profile.has_state() {
            problem.set(Some("A state is required.".to_string()));

            return;
        }

        problem.set(None);
        saving.set(true);

        spawn(async move {
            match profile::save(&profile).await {
                Ok(()) => on_saved.call(()),
                Err(error) => {
                    problem.set(Some(format!("The Profile could not be stored: {error}")));
                    saving.set(false);
                }
            }
        });
    };

    let answers = form();
    let saving = saving();
    let problem = problem();
    let has_problem = problem.is_some();
    let problem_line = problem.unwrap_or_default();

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            p { class: "hint",
                "The Civics Test asks about your own state and district. Every field except the state can wait; the Profile stays editable."
            }

            Field {
                label: "State",
                name: "state",
                value: answers.state.clone(),
                on_input: move |value| form.write().state = value,
            }
            Field {
                label: "Congressional district",
                name: "district",
                value: answers.district.clone(),
                on_input: move |value| form.write().district = value,
            }
            Field {
                label: "U.S. senator",
                name: "senator-one",
                value: answers.senator_one.clone(),
                on_input: move |value| form.write().senator_one = value,
            }
            Field {
                label: "Other U.S. senator",
                name: "senator-two",
                value: answers.senator_two.clone(),
                on_input: move |value| form.write().senator_two = value,
            }
            Field {
                label: "U.S. representative",
                name: "representative",
                value: answers.representative.clone(),
                on_input: move |value| form.write().representative = value,
            }
            Field {
                label: "Governor",
                name: "governor",
                value: answers.governor.clone(),
                on_input: move |value| form.write().governor = value,
            }
            Field {
                label: "State capital",
                name: "state-capital",
                value: answers.state_capital.clone(),
                on_input: move |value| form.write().state_capital = value,
            }

            fieldset { class: "group",
                legend { "Test Updates" }
                Field {
                    label: "President",
                    name: "president",
                    value: answers.president.clone(),
                    on_input: move |value| form.write().president = value,
                }
                Field {
                    label: "Vice President",
                    name: "vice-president",
                    value: answers.vice_president.clone(),
                    on_input: move |value| form.write().vice_president = value,
                }
                Field {
                    label: "Speaker of the House",
                    name: "speaker-of-the-house",
                    value: answers.speaker_of_the_house.clone(),
                    on_input: move |value| form.write().speaker_of_the_house = value,
                }
                Field {
                    label: "Chief Justice",
                    name: "chief-justice",
                    value: answers.chief_justice.clone(),
                    on_input: move |value| form.write().chief_justice = value,
                }
            }

            label { class: "check", r#for: "special-consideration",
                input {
                    id: "special-consideration",
                    r#type: "checkbox",
                    checked: answers.special_consideration_65_20,
                    onchange: move |event| form.write().special_consideration_65_20 = event.checked(),
                }
                span { "65/20 Special Consideration" }
            }
            p { class: "hint",
                "Aged 65 or older, and a lawful permanent resident for 20 or more years."
            }

            button {
                class: "primary",
                r#type: "button",
                disabled: saving,
                onclick: save,
                if saving {
                    "Storing the Profile…"
                } else {
                    "Save the Profile"
                }
            }

            if has_problem {
                p { class: "problem", "{problem_line}" }
            }
        }
    }
}

// region:    --- Support

/// The value behind a form field, or `None` where the Applicant left it blank.
///
/// A blank Local Fact or Test Update is a *missing* value: it leaves its
/// question in play until the Applicant fills it in, which is why blank never
/// becomes an empty answer.
fn missing_if_blank(field: &str) -> Option<String> {
    let value = field.trim();

    (!value.is_empty()).then(|| value.to_string())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_form_blank_fields_stay_missing() {
        // -- Setup & Fixtures
        let form = Form {
            state: "  New Jersey ".to_string(),
            district: "  ".to_string(),
            senator_one: "Cory Booker".to_string(),
            ..Form::default()
        };

        // -- Exec
        let profile = form.into_profile();

        // -- Check
        assert_eq!(profile.state, "New Jersey");
        assert_eq!(profile.district, None);
        assert_eq!(profile.senators[0], Some("Cory Booker".to_string()));
        assert_eq!(profile.senators[1], None);
        assert_eq!(profile.representative, None);
        assert_eq!(profile.test_updates, TestUpdates::default());
        assert!(!profile.special_consideration_65_20);
    }

    #[test]
    fn test_onboarding_missing_if_blank_holds_a_missing_value() {
        // -- Exec & Check
        assert_eq!(missing_if_blank("  "), None);
        assert_eq!(missing_if_blank(""), None);
        assert_eq!(missing_if_blank("  Albany "), Some("Albany".to_string()));
    }

    #[test]
    fn test_onboarding_form_keeps_the_flags_answer() {
        // -- Setup & Fixtures
        let form = Form {
            state: "New Jersey".to_string(),
            special_consideration_65_20: true,
            ..Form::default()
        };

        // -- Exec
        let profile = form.into_profile();

        // -- Check
        assert!(profile.special_consideration_65_20);
    }
}

// endregion: --- Tests
