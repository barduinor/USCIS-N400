use dioxus::prelude::*;

use crate::components::{FirstRun, InstallHint, ProgressEpoch, RefreshOnResume};
use crate::screens::{Exam, ExamResult, MainMenu, Profile, Stats, Study};
use crate::storage;

/// Every screen the Applicant can reach.
///
/// The screens land with the tickets that give them their behavior; the main
/// menu is where they all hang off.
#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    MainMenu {},

    #[route("/study")]
    Study {},

    #[route("/exam")]
    Exam {},

    #[route("/exam/result")]
    ExamResult {},

    #[route("/stats")]
    Stats {},

    #[route("/profile")]
    Profile {},
}

/// The application root: the one-time install line, then the route tree behind
/// the first-run gate.
#[component]
pub fn App() -> Element {
    let epoch = use_signal(|| 0u32);
    use_context_provider(|| ProgressEpoch(epoch));

    use_effect(move || {
        spawn(async {
            storage::request_persistent_storage().await;
        });
    });

    rsx! {
        RefreshOnResume {}
        InstallHint {}
        FirstRun { Router::<Route> {} }
    }
}
