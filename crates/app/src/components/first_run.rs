use dioxus::prelude::*;

use crate::screens::Onboarding;
use crate::storage;

/// What the first-run gate shows while it decides.
enum Gate {
    Loading,
    Failed(String),
    Onboarding,
    Ready,
}

/// The first-run gate: nothing renders until the Applicant has a Profile.
///
/// The Profile decides which Civics Test Questions the Applicant is accountable
/// for, so onboarding comes first; once a Profile is stored, the gate renders
/// the Applicant's screens.
#[component]
pub fn FirstRun(children: Element) -> Element {
    let mut profile = use_resource(storage::profile::load);

    let gate = match &*profile.read() {
        None => Gate::Loading,
        Some(Err(error)) => Gate::Failed(error.to_string()),
        Some(Ok(None)) => Gate::Onboarding,
        Some(Ok(Some(_))) => Gate::Ready,
    };

    match gate {
        Gate::Loading => rsx! {
            main { class: "screen",
                p { class: "loading", "Opening the Profile…" }
            }
        },
        Gate::Failed(problem) => rsx! {
            main { class: "screen",
                h1 { "USCIS N-400 Civics Study" }
                p { class: "problem", "The Profile could not be read: {problem}" }
            }
        },
        Gate::Onboarding => rsx! {
            Onboarding { on_saved: move |_| profile.restart() }
        },
        Gate::Ready => children,
    }
}
