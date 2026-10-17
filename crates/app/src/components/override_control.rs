//! The Override control for one wrong verdict.
//!
//! It marks the Answer Log's last entry as the Applicant's correction and
//! reports back that this attempt counts correct. It never re-judges the
//! answer and never changes the Accepted Answers. The screen decides where the
//! control sits and only renders it after a wrong verdict.

use dioxus::prelude::*;

use crate::storage;

/// The control that overrides the last Answer Log entry.
#[component]
pub fn OverrideControl(overridden: bool, on_overridden: EventHandler<()>) -> Element {
    let mut overriding = use_signal(|| false);
    let mut problem = use_signal(|| Option::<String>::None);

    if overridden {
        return rsx! {
            p { class: "hint", "Counts as correct for this attempt." }
        };
    }

    rsx! {
        button {
            class: "quiet",
            r#type: "button",
            disabled: overriding(),
            onclick: move |_| {
                if overriding() {
                    return;
                }

                overriding.set(true);
                problem.set(None);
                spawn(async move {
                    match storage::answer_log::override_last().await {
                        Ok(()) => on_overridden.call(()),
                        Err(error) => problem.set(Some(error.to_string())),
                    }

                    overriding.set(false);
                });
            },
            "Override"
        }
        if let Some(problem) = problem() {
            p { class: "problem", "{problem}" }
        }
    }
}
