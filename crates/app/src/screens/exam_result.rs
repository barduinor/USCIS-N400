use dioxus::prelude::*;

use crate::routes::Route;
use crate::storage;

/// What the Exam just sat came to: its outcome, its score, and the
/// administration it ran under.
///
/// The record is read back from storage, so the result survives a reload.
#[component]
pub fn ExamResult() -> Element {
    let latest = use_resource(storage::exam::latest);

    let body = match &*latest.read() {
        None => rsx! {
            p { class: "loading", "Reading the Exam…" }
        },
        Some(Err(error)) => rsx! {
            p { class: "problem", "The Exam could not be read: {error}" }
        },
        Some(Ok(None)) => rsx! {
            p { class: "problem", "No Exam has been taken yet." }
        },
        Some(Ok(Some(record))) => rsx! {
            p { class: "outcome", "{record.outcome}" }
            p { class: "score", "{record.correct_count} of {record.questions.len()} correct" }
            p { class: "track", "{record.track}" }
        },
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            {body}

            nav { class: "menu",
                Link { to: Route::Exam {}, class: "menu-item", "Take another Exam" }
                Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
            }
        }
    }
}
