use dioxus::prelude::*;

use domain::{Verdict, answers, matching, selection};

use crate::components::{Field, Reveal, use_refresh_on_resume};
use crate::questions;
use crate::routes::Route;
use crate::storage;

/// The endless study loop: one Civics Test Question at a time, judged as it is
/// answered, with the next one only when the Applicant asks for it.
///
/// The question carries its official number, so the Applicant can place it in
/// M-1778. The number is the record's own, not a position in the current
/// scope: narrowing to the 20 starred questions leaves it unchanged.
///
/// The verdict carries the answer the Applicant gave, as they wrote it, so it
/// can be read against the Accepted Answers; the field itself empties for the
/// next question. After a verdict the Applicant can ask for the reveal: the
/// Accepted Answers, and after a wrong verdict the Override inside the revealed
/// panel, which counts that attempt as correct and never touches the Accepted
/// Answers.
///
/// The way out sits above the question, on every state of the screen, so it
/// needs no scrolling on a phone. It is routing only: each answer is written as
/// it is given, so leaving changes no figure.
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
                    nav { class: "study-exit",
                        Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
                    }
                    h1 { "USCIS N-400 Civics Study" }
                    p { class: "problem", "The Civics Test Questions could not be read: {error}" }
                }
            };
        }
    };

    let mut profile = use_resource(storage::profile::load);
    let mut log = use_resource(storage::answer_log::entries_verified);
    let test_updates = match crate::test_updates::record() {
        Ok(record) => record,
        Err(error) => {
            return rsx! {
                main { class: "screen",
                    nav { class: "study-exit",
                        Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
                    }
                    h1 { "USCIS N-400 Civics Study" }
                    p { class: "problem", "The Test Update values could not be read: {error}" }
                }
            };
        }
    };
    let mut current = use_signal(|| Option::<u16>::None);
    let mut asked = use_signal(|| Option::<u16>::None);
    let mut typed = use_signal(String::new);
    let mut answered = use_signal(String::new);
    let mut verdict = use_signal(|| Option::<Verdict>::None);
    let mut overridden = use_signal(|| Option::<u16>::None);
    let mut saving = use_signal(|| false);
    let mut problem = use_signal(|| Option::<String>::None);

    use_refresh_on_resume(move || {
        profile.restart();
        log.restart();
    });

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
                let number = question.number;
                let class = if judged == Verdict::Correct {
                    "verdict correct"
                } else {
                    "verdict wrong"
                };
                let accepted = match &*profile.read() {
                    Some(Ok(Some(profile))) => answers::accepted_answers(
                        question,
                        &crate::test_updates::effective_profile(profile, test_updates),
                    ),
                    _ => Vec::new(),
                };
                let counts_correct = overridden() == Some(number);

                rsx! {
                    p { class: "question", "{question.question}" }
                    p { class: "{class}", "{judged}" }
                    div { class: "answer",
                        span { class: "field-label", "Your answer" }
                        p { "{answered}" }
                    }
                    if let Some(problem) = problem() {
                        p { class: "problem", "{problem}" }
                    }
                    div { class: "actions",
                        Reveal {
                            answers: accepted,
                            explanation: question.explanation.clone(),
                            wrong: judged == Verdict::Wrong,
                            overridden: counts_correct,
                            on_overridden: move |_| {
                                // The log the draw reads now holds the entry as it
                                // was before the Override.
                                log.restart();
                                overridden.set(Some(number));
                            },
                        }
                        button {
                            class: "primary",
                            onclick: move |_| {
                                current.set(None);
                                typed.set(String::new());
                                answered.set(String::new());
                                verdict.set(None);
                                overridden.set(None);
                                problem.set(None);
                            },
                            "Next question"
                        }
                    }
                }
            }
            None => {
                let number = question.number;
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
                                        answered.set(answer);
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
            nav { class: "study-exit",
                Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
            }
            h1 { "USCIS N-400 Civics Study" }
            if let Some(question) = shown {
                p { class: "question-number", "Question {question.number}" }
            }
            {body}
        }
    }
}
