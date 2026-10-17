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
    let mut log = use_resource(storage::answer_log::entries_verified);
    let mut exams = use_resource(storage::exam::exams_verified);
    let mut answers = use_signal(domain::Profile::default);
    let mut loaded = use_signal(|| false);
    let mut pending = use_signal(|| Option::<storage::bundle::Bundle>::None);
    let mut note = use_signal(|| Option::<String>::None);
    let mut problem = use_signal(|| Option::<String>::None);
    let mut working = use_signal(|| false);
    let mut confirming_clear_answers = use_signal(|| false);
    let mut confirming_clear_exams = use_signal(|| false);
    let mut confirming_reset = use_signal(|| false);
    let reload = use_context::<ProfileReload>();

    use_refresh_on_resume(move || {
        stored.restart();
        log.restart();
        exams.restart();
    });

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

    let keep = move |_| {
        pending.set(None);
        problem.set(None);
        note.set(Some(
            "Nothing was imported; this device's data is unchanged.".to_string(),
        ));
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
                    log.restart();
                    exams.restart();
                    note.set(Some("The bundle is in place.".to_string()));
                }
                Err(error) => problem.set(Some(error.to_string())),
            }

            working.set(false);
        });
    };

    let merge = move |_| {
        let Some(bundle) = pending() else {
            return;
        };

        working.set(true);
        problem.set(None);
        spawn(async move {
            match bundle.merge().await {
                Ok(()) => {
                    pending.set(None);
                    log.restart();
                    exams.restart();
                    note.set(Some(
                        "The bundle's answers and Exams are merged; this device's Profile stays."
                            .to_string(),
                    ));
                }
                Err(error) => problem.set(Some(error.to_string())),
            }

            working.set(false);
        });
    };

    let open_clear_answers = move |_| {
        note.set(None);
        problem.set(None);
        confirming_clear_answers.set(true);
    };

    let cancel_clear_answers = move |_| confirming_clear_answers.set(false);

    let clear_answers = move |_| {
        working.set(true);
        problem.set(None);
        note.set(None);
        spawn(async move {
            match storage::answer_log::clear().await {
                Ok(()) => {
                    log.restart();
                    confirming_clear_answers.set(false);
                    note.set(Some(
                        "The study answers are cleared; Coverage is now 0%.".to_string(),
                    ));
                }
                Err(error) => problem.set(Some(format!(
                    "The study answers could not be cleared: {error}"
                ))),
            }

            working.set(false);
        });
    };

    let open_clear_exams = move |_| {
        note.set(None);
        problem.set(None);
        confirming_clear_exams.set(true);
    };

    let cancel_clear_exams = move |_| confirming_clear_exams.set(false);

    let clear_exams = move |_| {
        working.set(true);
        problem.set(None);
        note.set(None);
        spawn(async move {
            match storage::exam::clear().await {
                Ok(()) => {
                    exams.restart();
                    confirming_clear_exams.set(false);
                    note.set(Some("The Exam history is cleared.".to_string()));
                }
                Err(error) => problem.set(Some(format!(
                    "The Exam history could not be cleared: {error}"
                ))),
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

    let local_progress = match (&*log.read(), &*exams.read()) {
        (Some(Ok(answers)), Some(Ok(exams))) => Some((answers.len(), exams.len())),
        _ => None,
    };
    let local_read_failed =
        matches!(&*log.read(), Some(Err(_))) || matches!(&*exams.read(), Some(Err(_)));

    let confirmation = match pending() {
        None => rsx! {},
        Some(bundle) => {
            let bundle_answers = count(bundle.answer_log.len(), "answer");
            let bundle_exams = count(bundle.exams.len(), "Exam");

            match local_progress {
                None if local_read_failed => rsx! {
                    p { class: "problem",
                        "This device's progress could not be read, so the import is paused. No data has changed."
                    }
                    button {
                        class: "quiet",
                        r#type: "button",
                        onclick: keep,
                        "Keep this device's data"
                    }
                    p { class: "hint", "Reload the app, then choose the bundle again." }
                },
                None => rsx! {
                    p { class: "loading", "Reading this device's progress…" }
                },
                Some((0, 0)) => rsx! {
                    p { class: "problem",
                        "Importing replaces the Profile, the Answer Log, and the Exam history. Nothing is merged, and what is here now goes."
                    }
                    p { class: "hint", "The bundle holds {bundle_answers} and {bundle_exams}." }
                    div { class: "actions",
                        button {
                            class: "quiet",
                            r#type: "button",
                            disabled: working(),
                            onclick: keep,
                            "Cancel import"
                        }
                        button {
                            class: "primary",
                            r#type: "button",
                            disabled: working(),
                            onclick: replace,
                            "Replace what is on this device"
                        }
                    }
                },
                Some((answers, exams)) => {
                    let local_answers = count(answers, "answer");
                    let local_exams = count(exams, "Exam");

                    rsx! {
                        p { class: "problem",
                            "This device already holds {local_answers} and {local_exams}. Choose what happens to them."
                        }
                        p { class: "hint", "The bundle holds {bundle_answers} and {bundle_exams}." }
                        div { class: "actions",
                            button {
                                class: "quiet",
                                r#type: "button",
                                disabled: working(),
                                onclick: keep,
                                "Keep this device's data"
                            }
                            button {
                                class: "danger",
                                r#type: "button",
                                disabled: working(),
                                onclick: replace,
                                "Replace this device's data"
                            }
                            button {
                                class: "primary",
                                r#type: "button",
                                disabled: working(),
                                onclick: merge,
                                "Merge the bundle into this device"
                            }
                        }
                    }
                }
            }
        }
    };

    let answers_state = match &*log.read() {
        None => "Reading the study answers…".to_string(),
        Some(Err(error)) => format!("The study answers could not be read: {error}"),
        Some(Ok(entries)) if entries.is_empty() => "No study answers are stored.".to_string(),
        Some(Ok(entries)) => format!("{} stored.", count(entries.len(), "answer")),
    };
    let exams_state = match &*exams.read() {
        None => "Reading the Exam history…".to_string(),
        Some(Err(error)) => format!("The Exam history could not be read: {error}"),
        Some(Ok(exams)) if exams.is_empty() => "No Exam is stored.".to_string(),
        Some(Ok(exams)) => format!("{} stored.", count(exams.len(), "Exam")),
    };
    let answers_clearable = matches!(&*log.read(), Some(Ok(entries)) if !entries.is_empty());
    let exams_clearable = matches!(&*exams.read(), Some(Ok(exams)) if !exams.is_empty());

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
                section { class: "clear",
                    h2 { "Clear" }
                    p { class: "hint",
                        "These delete one history at a time. The Profile and the other history stay."
                    }

                    div { class: "clear-row",
                        h3 { "Study answers" }
                        p { class: "hint", "{answers_state}" }
                        if confirming_clear_answers() {
                            p { class: "problem",
                                "Clearing the study answers deletes every answer in the Answer Log. Coverage goes to 0%. The Profile and the Exam history stay. This cannot be undone."
                            }
                            div { class: "actions",
                                button {
                                    class: "danger",
                                    r#type: "button",
                                    disabled: working(),
                                    onclick: clear_answers,
                                    "Delete my study answers"
                                }
                                button {
                                    class: "quiet",
                                    r#type: "button",
                                    disabled: working(),
                                    onclick: cancel_clear_answers,
                                    "Keep my answers"
                                }
                            }
                        } else {
                            button {
                                class: "danger",
                                r#type: "button",
                                disabled: working() || !answers_clearable,
                                onclick: open_clear_answers,
                                "Clear study answers"
                            }
                        }
                    }

                    div { class: "clear-row",
                        h3 { "Exam history" }
                        p { class: "hint", "{exams_state}" }
                        if confirming_clear_exams() {
                            p { class: "problem",
                                "Clearing the Exam history deletes every Exam record. Coverage stays. The Profile and the Answer Log stay. This cannot be undone."
                            }
                            div { class: "actions",
                                button {
                                    class: "danger",
                                    r#type: "button",
                                    disabled: working(),
                                    onclick: clear_exams,
                                    "Delete my Exam history"
                                }
                                button {
                                    class: "quiet",
                                    r#type: "button",
                                    disabled: working(),
                                    onclick: cancel_clear_exams,
                                    "Keep my Exam history"
                                }
                            }
                        } else {
                            button {
                                class: "danger",
                                r#type: "button",
                                disabled: working() || !exams_clearable,
                                onclick: open_clear_exams,
                                "Clear Exam history"
                            }
                        }
                    }
                }

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
