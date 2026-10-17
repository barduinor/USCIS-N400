use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::{ProfileForm, ProfileReload, use_refresh_on_resume};
use crate::storage;

/// The Applicant's own facts, editable at any time, and their backup.
///
/// The Profile stays editable because the 65/20 flag and every Local Fact can
/// change: an edit takes effect on the next question and on Coverage's
/// denominator. Below it sit the export — a versioned JSON bundle of everything
/// — and the import, which replaces what is on this device after a
/// confirmation and never merges. Below them sits the reset: it empties every
/// store after a confirmation that offers the export first, and the first-run
/// gate then puts the app back on onboarding.
#[component]
pub fn Profile() -> Element {
    let mut stored = use_resource(storage::profile::load);
    let mut answers = use_signal(domain::Profile::default);
    let mut loaded = use_signal(|| false);
    let mut pending = use_signal(|| Option::<storage::bundle::Bundle>::None);
    let mut note = use_signal(|| Option::<String>::None);
    let mut problem = use_signal(|| Option::<String>::None);
    let mut working = use_signal(|| false);
    let mut confirming_reset = use_signal(|| false);
    let reload = use_context::<ProfileReload>();

    use_refresh_on_resume(move || stored.restart());

    // The form starts from what is stored, once, so an edit is never clobbered.
    use_effect(move || {
        if *loaded.peek() {
            return;
        }

        let Some(Ok(Some(profile))) = &*stored.read() else {
            return;
        };

        answers.set(profile.clone());
        loaded.set(true);
    });

    let save = move |_| {
        let profile = crate::components::profile_form::as_profile(&answers());

        if !profile.has_state() {
            problem.set(Some("A state is required.".to_string()));

            return;
        }

        working.set(true);
        note.set(None);
        problem.set(None);
        spawn(async move {
            match storage::profile::save(&profile).await {
                Ok(()) => note.set(Some("The Profile is saved.".to_string())),
                Err(error) => {
                    problem.set(Some(format!("The Profile could not be stored: {error}")))
                }
            }

            working.set(false);
        });
    };

    let export = move |_| {
        working.set(true);
        note.set(None);
        problem.set(None);
        spawn(async move {
            let exported: crate::Result<()> = async {
                let bundle = storage::bundle::Bundle::gather().await?;
                bundle.download()?;

                note.set(Some(
                    "The bundle is in your downloads, and it is the only copy of your progress outside this device.".to_string(),
                ));

                if let Err(error) = bundle.note_export().await {
                    problem.set(Some(format!("The export could not be noted: {error}")));
                }

                Ok(())
            }
            .await;

            if let Err(error) = exported {
                problem.set(Some(error.to_string()));
            }

            working.set(false);
        });
    };

    let replace = move |_| {
        let Some(bundle) = pending() else {
            return;
        };

        working.set(true);
        problem.set(None);
        spawn(async move {
            match bundle.replace().await {
                Ok(()) => {
                    answers.set(bundle.profile.clone());
                    loaded.set(true);
                    pending.set(None);
                    note.set(Some("The bundle is in place.".to_string()));
                }
                Err(error) => problem.set(Some(error.to_string())),
            }

            working.set(false);
        });
    };

    let open_reset = move |_| {
        note.set(None);
        problem.set(None);
        confirming_reset.set(true);
    };

    let cancel_reset = move |_| confirming_reset.set(false);

    let reset = move |_| {
        working.set(true);
        problem.set(None);
        spawn(async move {
            match storage::reset().await {
                Ok(()) => {
                    confirming_reset.set(false);
                    working.set(false);
                    reload.0.call(());
                }
                Err(error) => {
                    problem.set(Some(format!("This device could not be cleared: {error}")));
                    working.set(false);
                }
            }
        });
    };

    let body = match (&*stored.read(), loaded()) {
        (Some(Err(error)), _) => rsx! {
            p { class: "problem", "The Profile could not be read: {error}" }
        },
        (_, false) => rsx! {
            p { class: "loading", "Reading the Profile…" }
        },
        _ => {
            let working = working();

            rsx! {
                ProfileForm { profile: answers }

                button {
                    class: "primary",
                    r#type: "button",
                    disabled: working,
                    onclick: save,
                    "Save the Profile"
                }
            }
        }
    };

    let confirmation = match pending() {
        None => rsx! {},
        Some(bundle) => {
            let answers = count(bundle.answer_log.len(), "answer");
            let exams = count(bundle.exams.len(), "Exam");

            rsx! {
                p { class: "problem",
                    "Importing replaces everything on this device — the Profile, the Answer Log, and the Exam history. Nothing is merged, and what is here now goes."
                }
                p { class: "hint", "The bundle holds {answers} and {exams}." }
                button {
                    class: "primary",
                    r#type: "button",
                    disabled: working(),
                    onclick: replace,
                    "Replace what is on this device"
                }
            }
        }
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            {body}

            if let Some(problem) = problem() {
                p { class: "problem", "{problem}" }
            }
            if let Some(note) = note() {
                p { class: "hint", "{note}" }
            }

            section { class: "backup",
                h2 { "Backup" }
                p { class: "hint",
                    "The export is the only copy of your progress outside this device: keep it somewhere safe."
                }
                button {
                    class: "primary",
                    r#type: "button",
                    disabled: working(),
                    onclick: export,
                    "Export"
                }
                label { class: "field", r#for: "import",
                    span { class: "field-label", "Import a bundle" }
                    input {
                        id: "import",
                        r#type: "file",
                        accept: "application/json,.json",
                        onchange: move |event| {
                            let Some(file) = event.files().into_iter().next() else {
                                return;
                            };

                            working.set(true);
                            note.set(None);
                            problem.set(None);
                            spawn(async move {
                                match file.read_string().await {
                                    Ok(text) => match storage::bundle::Bundle::from_text(&text) {
                                        Ok(bundle) => {
                                            pending.set(Some(bundle));
                                            note.set(Some("That bundle can be read. Nothing has changed yet.".to_string()));
                                        }
                                        Err(error) => {
                                            pending.set(None);
                                            problem.set(Some(error.to_string()));
                                        }
                                    },
                                    Err(error) => {
                                        pending.set(None);
                                        problem.set(Some(format!("That file could not be read: {error}")));
                                    }
                                }

                                // The control keeps its file, and a browser fires
                                // `change` only when the value differs: without
                                // this, picking the same file again does nothing.
                                forget_chosen_file();
                                working.set(false);
                            });
                        },
                    }
                }
                {confirmation}
            }

            if loaded() {
                section { class: "reset",
                    h2 { "Reset" }
                    if confirming_reset() {
                        p { class: "problem",
                            "Resetting deletes the Profile, every answer in the Answer Log, and the entire Exam history from this device. It cannot be undone. An exported file is the only copy that survives."
                        }
                        p { class: "hint",
                            "The app itself stays installed, and it still opens offline. If you have no export yet, make one now."
                        }
                        div { class: "actions",
                            button {
                                class: "primary",
                                r#type: "button",
                                disabled: working(),
                                onclick: export,
                                "Export my progress first"
                            }
                            button {
                                class: "danger",
                                r#type: "button",
                                disabled: working(),
                                onclick: reset,
                                "Delete everything and reset"
                            }
                            button {
                                class: "quiet",
                                r#type: "button",
                                disabled: working(),
                                onclick: cancel_reset,
                                "Keep my progress"
                            }
                        }
                    } else {
                        p { class: "hint",
                            "Deletes the Profile, every answer in the Answer Log, and the entire Exam history from this device, then returns to onboarding. An exported file is the only copy that survives."
                        }
                        button {
                            class: "danger",
                            r#type: "button",
                            disabled: working(),
                            onclick: open_reset,
                            "Reset this device"
                        }
                    }
                }
            }

            nav { class: "menu",
                Link { to: crate::routes::Route::MainMenu {}, class: "menu-item", "Back to the menu" }
            }
        }
    }
}

/// A count with its noun, agreeing in number: `1 answer`, `4 answers`.
fn count(how_many: usize, noun: &str) -> String {
    if how_many == 1 {
        format!("1 {noun}")
    } else {
        format!("{how_many} {noun}s")
    }
}

/// Empties the file control, so the same file can be picked again.
fn forget_chosen_file() {
    let input = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("import"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok());

    if let Some(input) = input {
        input.set_value("");
    }
}
