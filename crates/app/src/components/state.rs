//! The State field: the Applicant's state, chosen from the states rather than
//! typed.
//!
//! Onboarding and the Profile screen both render this field through the shared
//! Profile form, so the two cannot offer different lists. What is stored stays
//! the state's name, so a Profile written before the picker existed still
//! loads.

use dioxus::prelude::*;

/// The fifty states and the District of Columbia, as the picker offers them.
///
/// The District has no U.S. senators and no governor: those Local Facts stay
/// blank for it, and a blank Local Fact leaves its question in play
/// (GLOSSARY).
pub const STATES: [&str; 51] = [
    "Alabama",
    "Alaska",
    "Arizona",
    "Arkansas",
    "California",
    "Colorado",
    "Connecticut",
    "Delaware",
    "District of Columbia",
    "Florida",
    "Georgia",
    "Hawaii",
    "Idaho",
    "Illinois",
    "Indiana",
    "Iowa",
    "Kansas",
    "Kentucky",
    "Louisiana",
    "Maine",
    "Maryland",
    "Massachusetts",
    "Michigan",
    "Minnesota",
    "Mississippi",
    "Missouri",
    "Montana",
    "Nebraska",
    "Nevada",
    "New Hampshire",
    "New Jersey",
    "New Mexico",
    "New York",
    "North Carolina",
    "North Dakota",
    "Ohio",
    "Oklahoma",
    "Oregon",
    "Pennsylvania",
    "Rhode Island",
    "South Carolina",
    "South Dakota",
    "Tennessee",
    "Texas",
    "Utah",
    "Vermont",
    "Virginia",
    "Washington",
    "West Virginia",
    "Wisconsin",
    "Wyoming",
];

/// One labelled State field of a form.
///
/// The value lives with the screen that owns the form, like every other field;
/// this component only renders the states and reports the choice.
#[component]
pub fn StateField(value: String, on_input: EventHandler<String>) -> Element {
    rsx! {
        label { class: "field", r#for: "state",
            span { class: "field-label", "State" }
            select {
                id: "state",
                name: "state",
                onchange: move |event| on_input.call(event.value()),
                option { value: "", selected: value.is_empty(), "Choose a state" }
                for state in choices(&value) {
                    option { value: state, selected: state == value, "{state}" }
                }
            }
        }
    }
}

/// The entries the picker offers for `value`: the states, and the value itself
/// when it is not one of them.
///
/// A Profile written before the picker existed holds whatever was typed; it
/// stays chosen here, so loading and saving such a Profile never drops or
/// rewrites its state. Correcting it is what the picker is for.
fn choices(value: &str) -> Vec<&str> {
    let mut choices = STATES.to_vec();

    if !value.trim().is_empty() && !STATES.contains(&value) {
        choices.push(value);
    }

    choices
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_states_holds_the_fifty_states_and_the_district() {
        // -- Exec & Check
        assert_eq!(STATES.len(), 51, "the fifty states and the District");
        assert!(
            STATES.contains(&"District of Columbia"),
            "the District is offered beside the states"
        );
        assert!(
            STATES.windows(2).all(|pair| pair[0] < pair[1]),
            "the picker reads alphabetically, with no entry twice"
        );
    }

    #[test]
    fn test_state_choices_offers_a_state_it_already_lists() {
        // -- Exec & Check
        assert_eq!(
            choices("New Jersey"),
            STATES.to_vec(),
            "a state the picker lists is not offered twice"
        );
        assert_eq!(choices(""), STATES.to_vec(), "no state means no entry");
        assert_eq!(choices("   "), STATES.to_vec(), "a blank state means none");
    }

    #[test]
    fn test_state_choices_keeps_a_stored_state_it_does_not_list() {
        // -- Setup & Fixtures
        let mut expected = STATES.to_vec();
        expected.push("new jersey");

        // -- Exec & Check
        assert_eq!(
            choices("new jersey"),
            expected,
            "a Profile written before the picker keeps its state on screen"
        );
    }
}

// endregion: --- Tests
