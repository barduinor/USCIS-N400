use dioxus::prelude::*;

use crate::components::{FirstRun, InstallHint, ProgressEpoch, RefreshOnResume};
use crate::screens::{Exam, ExamResult, MainMenu, Profile, Stats, StudyScreen};
use crate::storage;
use crate::study_scope::{self, StudyScope};

/// Study every accountable Civics Test Question.
#[component]
fn Study() -> Element {
    rsx! {
        StudyScreen { study: StudyScope::All }
    }
}

/// Study one Civics Test Topic.
#[component]
fn StudyTopic(topic: String) -> Element {
    let Some(topic) = study_scope::from_slug(&topic) else {
        return rsx! {
            main { class: "screen",
                nav { class: "study-exit",
                    Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
                    Link { to: Route::Study {}, class: "menu-item", "All topics" }
                    Link { to: Route::Stats {}, class: "menu-item", "Change topic" }
                }
                h1 { "USCIS N-400 Civics Study" }
                p { class: "problem", "That Topic is not one of the eight Civics Test Topics." }
            }
        };
    };

    let key = study_scope::slug(topic);

    rsx! {
        StudyScreen { key: "{key}", study: StudyScope::Topic(topic) }
    }
}

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

    #[route("/study/:topic")]
    StudyTopic { topic: String },

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

    use_hook(move || {
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
