//! The Override control for one wrong verdict.
//!
//! It offers the Applicant's correction and reports the request to the parent.
//! The parent performs it: Study marks the Answer Log's last entry, and the
//! Exam amends its own score and saved record. The control never re-judges the
//! answer, never changes the Accepted Answers, and never reaches storage
//! itself. The parent owns the storage call and shows its errors.

use dioxus::prelude::*;

/// The control that requests an Override of the last verdict.
///
/// `overridden` shows the confirmation instead of the button, `busy` disables
/// the button while the parent's Override is in flight, and `on_override` asks
/// the parent to perform it.
#[component]
pub fn OverrideControl(overridden: bool, busy: bool, on_override: EventHandler<()>) -> Element {
    if overridden {
        return rsx! {
            p { class: "hint", "Counts as correct for this attempt." }
        };
    }

    rsx! {
        button {
            class: "quiet",
            r#type: "button",
            disabled: busy,
            onclick: move |_| on_override.call(()),
            "Override"
        }
    }
}
