use dioxus::prelude::*;

use crate::install;
use crate::storage;

/// The one-time line for iOS, before the Applicant studies: the app has to be
/// put on the home screen first, because the installed app keeps its own
/// storage, separate from the browser tab (ADR 0003).
///
/// It is shown once per browsing context. The database's own record holds that
/// it was shown, written as the line appears, so a reload does not bring it
/// back and a dismissal is the Applicant's to make.
#[component]
pub fn InstallHint() -> Element {
    let mut shown = use_signal(|| false);

    use_future(move || async move {
        if !install::needs_hint() {
            return;
        }

        // A record that cannot be written shows the line once more, which
        // repeats a courtesy rather than losing it.
        if let Ok(false) = storage::hint::seen().await {
            let _ = storage::hint::mark_seen().await;
            shown.set(true);
        }
    });

    if !shown() {
        return rsx! {};
    }

    rsx! {
        aside { class: "install-hint",
            p {
                "Before studying, put this app on your home screen: Share, then Add to Home Screen. \
                 The installed app keeps its own storage, separate from this tab, so the progress \
                 you make here stays here."
            }
            button { r#type: "button", onclick: move |_| shown.set(false), "Dismiss" }
        }
    }
}
