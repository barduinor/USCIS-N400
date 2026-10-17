use dioxus::prelude::*;

use domain::{ExamRecord, Track, answers, matching};

use crate::components::Field;
use crate::questions;
use crate::routes::Route;
use crate::storage;

/// One Exam: the officer's questions, and nothing else.
///
/// The screen shows only the question. No counter and no verdict appear here,
/// because either would reveal the result before the result screen (GLOSSARY),
/// and answers go to the Exam, never to the Answer Log.
#[component]
pub fn Exam() -> Element {
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

    let navigator = navigator();
    let profile = use_resource(storage::profile::load);
    let test_updates = match crate::test_updates::record() {
        Ok(record) => record,
        Err(error) => {
            return rsx! {
                main { class: "screen",
                    h1 { "USCIS N-400 Civics Study" }
                    p { class: "problem", "The Test Update values could not be read: {error}" }
                }
            };
        }
    };
    let mut exam = use_signal(|| Option::<domain::Exam>::None);
    let mut typed = use_signal(String::new);
    let mut finished = use_signal(|| Option::<ExamRecord>::None);
    let mut saving = use_signal(|| false);
    let mut problem = use_signal(|| Option::<String>::None);

    // The Exam is set once the Profile is in hand: it fixes the administration.
    use_effect(move || {
        if exam.peek().is_some() {
            return;
        }

        let Some(Ok(Some(profile))) = &*profile.read() else {
            return;
        };

        let candidates = records.accountable(profile);
        exam.set(Some(domain::Exam::start(
            &candidates,
            Track::of(profile),
            storage::now(),
        )));
    });

    // Writes the finished Exam's record, and moves to the result. A failed write
    // keeps the record, so the Applicant can try it again.
    let save = use_callback(move |()| {
        let Some(record) = finished() else {
            return;
        };

        if saving() {
            return;
        }

        saving.set(true);
        problem.set(None);
        spawn(async move {
            match storage::exam::append(&record).await {
                Ok(()) => {
                    navigator.push(Route::ExamResult {});
                }
                Err(error) => problem.set(Some(error.to_string())),
            }

            saving.set(false);
        });
    });

    let sitting = exam.read().clone();
    let body = match (&*profile.read(), sitting) {
        (Some(Err(error)), _) => rsx! {
            p { class: "problem", "The Profile could not be read: {error}" }
        },
        (_, None) => rsx! {
            p { class: "loading", "Setting the Exam…" }
        },
        (_, Some(sitting)) => match sitting.current() {
            None => rsx! {
                if let Some(problem) = problem() {
                    p { class: "problem", "{problem}" }
                }
                button {
                    class: "primary",
                    disabled: saving(),
                    onclick: move |_| save.call(()),
                    "Save the Exam"
                }
                nav { class: "menu",
                    Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
                }
            },
            Some(number) => match records.question(number) {
                None => rsx! {
                    p { class: "problem", "Civics Test Question {number} could not be read." }
                },
                Some(question) => {
                    let accepted = match &*profile.read() {
                        Some(Ok(Some(profile))) => answers::accepted_answers(
                            question,
                            &crate::test_updates::effective_profile(profile, test_updates),
                        ),
                        _ => Vec::new(),
                    };

                    rsx! {
                        p { class: "question", "{question.question}" }
                        if let Some(problem) = problem() {
                            p { class: "problem", "{problem}" }
                        }
                        form {
                            onsubmit: move |event| {
                                event.prevent_default();

                                if saving() || typed().trim().is_empty() {
                                    return;
                                }

                                let judged = matching::judge(question, &accepted, &typed());
                                let produced = {
                                    let mut exam = exam.write();
                                    let running = exam.as_mut();

                                    running.and_then(|running| {
                                        running.answer(judged);
                                        running.record(storage::now())
                                    })
                                };

                                typed.set(String::new());
                                problem.set(None);

                                // The officer stops the moment the outcome is
                                // fixed: the record goes down, and the result
                                // screen takes over.
                                if let Some(record) = produced {
                                    finished.set(Some(record));
                                    save.call(());
                                }
                            },
                            Field {
                                label: "Your answer",
                                name: "answer",
                                value: typed(),
                                on_input: move |value| typed.set(value),
                            }
                            button {
                                class: "primary",
                                r#type: "submit",
                                disabled: saving() || typed().trim().is_empty(),
                                "Answer"
                            }
                        }
                    }
                }
            },
        },
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            {body}
        }
    }
}
