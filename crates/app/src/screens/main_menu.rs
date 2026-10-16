use dioxus::prelude::*;

use domain::Coverage;

use crate::questions;

/// The entry point: where the Applicant stands, and where they can go.
#[component]
pub fn MainMenu() -> Element {
    let records = match questions::records() {
        Ok(records) => records,
        Err(error) => {
            return rsx! {
                main { class: "screen",
                    h1 { "USCIS N-400 Civics Study" }
                    p { class: "problem", "The Civics Test Questions could not be read: {error}" }
                }
            };
        }
    };

    let coverage = Coverage::none(records);

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }

            section { class: "coverage",
                h2 { "Coverage" }
                p { class: "coverage-figure", "{coverage}" }
            }
        }
    }
}
