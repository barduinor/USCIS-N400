use dioxus::prelude::*;

use domain::{Verdict, answers, matching, selection};

use crate::components::Field;
use crate::questions;
use crate::storage;

/// The endless study loop: one Civics Test Question at a time, judged as it is
/// answered, with the next one only when the Applicant asks for it.
///
/// No statistics appear here: Coverage lives on the main menu, and the
/// Applicant stops when they choose (GLOSSARY).
#[component]
pub fn Study() -> Element {
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

    let profile = use_resource(storage::profile::load);
    let mut log = use_resource(storage::answer_log::entries);
    let mut current = use_signal(|| Option::<u16>::None);
    let mut asked = use_signal(|| Option::<u16>::None);
    let mut typed = use_signal(String::new);
    let mut verdict = use_signal(|| Option::<Verdict>::None);
    let mut saving = use_signal(|| false);
    let mut problem = use_signal(|| Option::<String>::None);

    // The question on screen: the one the Applicant is answering, or the next
    // one chosen. It stays put once answered, so the verdict is never yanked
    // away.
    let shown = current()
        .and_then(|number| records.question(number))
        .or_else(|| {
            let (Some(Ok(Some(profile))), Some(Ok(log))) = (&*profile.read(), &*log.read()) else {
                return None;
            };

            selection::next(&records.accountable(profile), log, asked())
                .and_then(|question| records.question(question.number))
        });

    let body = match shown {
        Some(question) => match verdict() {
            Some(judged) => {
                let class = if judged == Verdict::Correct {
                    "verdict correct"
                } else {
                    "verdict wrong"
                };

                rsx! {
                    p { class: "question", "{question.question}" }
                    p { class: "{class}", "{judged}" }
                    if let Some(problem) = problem() {
                        p { class: "problem", "{problem}" }
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            current.set(None);
                            typed.set(String::new());
                            verdict.set(None);
                            problem.set(None);
                        },
                        "Next question"
                    }
                }
            }
            None => {
                let number = question.number;
                let accepted = match &*profile.read() {
                    Some(Ok(Some(profile))) => answers::accepted_answers(question, profile),
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

                            let answer = typed();
                            let judged = matching::judge(question, &accepted, &answer);

                            current.set(Some(number));
                            asked.set(Some(number));
                            saving.set(true);
                            problem.set(None);

                            spawn(async move {
                                match storage::answer_log::append(number, &answer, judged).await {
                                    Ok(_) => {
                                        log.restart();
                                        verdict.set(Some(judged));
                                    }
                                    Err(error) => problem.set(Some(error.to_string())),
                                }

                                saving.set(false);
                            });
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
        None => match (&*profile.read(), &*log.read()) {
            (Some(Err(error)), _) | (_, Some(Err(error))) => rsx! {
                p { class: "problem", "The study data could not be read: {error}" }
            },
            _ => rsx! {
                p { class: "loading", "Opening the Profile…" }
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
