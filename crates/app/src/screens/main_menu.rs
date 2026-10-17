use dioxus::prelude::*;

use domain::Coverage;

use crate::components::{CoverageFigure, use_refresh_on_resume};
use crate::questions;
use crate::routes::Route;
use crate::storage;

/// The entry point: where the Applicant stands, and where they can go.
///
/// Coverage is read from the Answer Log every time the screen opens, so the
/// figure moves as questions are Cleared and survives a reload.
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

    let mut profile = use_resource(storage::profile::load);
    let mut log = use_resource(storage::answer_log::entries_verified);

    use_refresh_on_resume(move || {
        profile.restart();
        log.restart();
    });

    let figure = match (&*profile.read(), &*log.read()) {
        (Some(Ok(Some(profile))), Some(Ok(log))) => {
            let coverage = Coverage::from_log(records, profile, log);

            rsx! {
                CoverageFigure { coverage }
            }
        }
        (Some(Ok(None)), _) => rsx! {
            p { class: "problem", "The Profile is missing, so there is nothing to measure." }
        },
        (Some(Err(error)), _) | (_, Some(Err(error))) => rsx! {
            p { class: "problem", "The progress could not be read: {error}" }
            div { class: "actions",
                button {
                    class: "primary",
                    r#type: "button",
                    onclick: move |_| {
                        profile.restart();
                        log.restart();
                    },
                    "Try again"
                }
                Link { to: Route::Profile {}, class: "menu-item", "Export from the Profile" }
            }
        },
        _ => rsx! {
            p { class: "loading", "Reading the Answer Log…" }
        },
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }

            section { class: "coverage",
                h2 { "Coverage" }
                {figure}
            }

            nav { class: "menu",
                Link { to: Route::Study {}, class: "menu-item", "Study" }
                Link { to: Route::Exam {}, class: "menu-item", "Exam" }
                Link { to: Route::Stats {}, class: "menu-item", "Stats" }
                Link { to: Route::Profile {}, class: "menu-item", "Profile" }
            }
        }
    }
}
