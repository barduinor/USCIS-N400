use dioxus::prelude::*;

use domain::{ExamRecord, Track, Verdict, answers, matching};

use crate::components::{Answers, Field};
use crate::questions;
use crate::routes::Route;
use crate::storage;

/// One Exam: the officer's questions, with the current Exam's progress.
///
/// The screen shows a progress bar for the Exam position against the draw's
/// maximum, the right and wrong counts, and the official question number
/// directly above the question text. The bar can stop short of its maximum:
/// the outcome ends the Exam, not the draw. After each answer it shows the
/// verdict, the answer the Applicant gave as they wrote it, and the Accepted
/// Answers, so the two can be compared; the Explanation appears when the
/// Applicant asks. It then waits for the Applicant: the next question for a
/// running Exam, or the result for the one that just ended. Answers go to the
/// Exam, never to the Answer Log.
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
    let mut answered = use_signal(String::new);
    let mut feedback = use_signal(|| Option::<(u16, Verdict)>::None);
    let mut finished = use_signal(|| Option::<ExamRecord>::None);
    let mut saved = use_signal(|| false);
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

    // Writes the finished Exam's record, once. The result screen opens only
    // when asked; the record is already on the device, so leaving the feedback
    // screen keeps the Exam. A failed write keeps the record for a retry.
    let save = use_callback(move |navigate: bool| {
        let Some(record) = finished() else {
            return;
        };

        if saving() {
            return;
        }

        if saved() {
            if navigate {
                navigator.push(Route::ExamResult {});
            }

            return;
        }

        saving.set(true);
        problem.set(None);
        spawn(async move {
            match storage::exam::append(&record).await {
                Ok(()) => {
                    saved.set(true);

                    if navigate {
                        navigator.push(Route::ExamResult {});
                    }
                }
                Err(error) => problem.set(Some(error.to_string())),
            }

            saving.set(false);
        });
    });

    let sitting = exam.read().clone();
    let feedback_value = feedback();
    let shown_number = feedback_value
        .map(|(number, _)| number)
        .or_else(|| sitting.as_ref().and_then(|sitting| sitting.current()));
    let accepted = match &*profile.read() {
        Some(Ok(Some(profile))) => shown_number
            .and_then(|number| records.question(number))
            .map(|question| {
                answers::accepted_answers(
                    question,
                    &crate::test_updates::effective_profile(profile, test_updates),
                )
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let body = match (&*profile.read(), sitting) {
        (Some(Err(error)), _) => rsx! {
            p { class: "problem", "The Profile could not be read: {error}" }
        },
        (_, None) => rsx! {
            p { class: "loading", "Setting the Exam…" }
        },
        (_, Some(sitting)) => {
            let current = sitting.current();
            let in_feedback = feedback_value.is_some();
            let mut sequence = sitting.asked();

            if !in_feedback && let Some(number) = current {
                sequence.push(number);
            }

            let position = sequence.len();
            let total = sitting.track().questions();

            rsx! {
                if shown_number.is_some() {
                    div { class: "exam-progress",
                        div { class: "exam-status",
                            span { "Exam progress" }
                            span { "Right {sitting.correct()} · Wrong {sitting.wrong()}" }
                        }
                        progress { value: position, max: total }
                        p { class: "exam-position", "{position} of {total}" }
                    }
                }

                if let Some((number, judged)) = feedback_value {
                    if let Some(question) = records.question(number) {
                        div { class: "question-block",
                            p { class: "question-number", "Question {number}" }
                            p { class: "question", "{question.question}" }
                        }
                    }
                    p {
                        class: if judged == Verdict::Correct { "verdict correct" } else { "verdict wrong" },
                        "{judged}"
                    }
                    div { class: "answer",
                        span { class: "field-label", "Your answer" }
                        p { "{answered}" }
                    }
                    if let Some(question) = records.question(number) {
                        Answers {
                            answers: accepted,
                            explanation: question.explanation.clone(),
                        }
                    }
                    if let Some(problem) = problem() {
                        p { class: "problem", "{problem}" }
                    }
                    div { class: "actions",
                        if finished().is_some() {
                            button {
                                class: "primary",
                                disabled: saving(),
                                onclick: move |_| save.call(true),
                                if saving() {
                                    "Saving the Exam…"
                                } else {
                                    "See result"
                                }
                            }
                        } else {
                            button {
                                class: "primary",
                                onclick: move |_| {
                                    feedback.set(None);
                                    answered.set(String::new());
                                    problem.set(None);
                                },
                                "Next question"
                            }
                        }
                    }
                } else if let Some(number) = current {
                    match records.question(number) {
                        None => rsx! {
                            p { class: "problem", "Civics Test Question {number} could not be read." }
                        },
                        Some(question) => {
                            rsx! {
                                div { class: "question-block",
                                    p { class: "question-number", "Question {number}" }
                                    p { class: "question", "{question.question}" }
                                }
                                if let Some(problem) = problem() {
                                    p { class: "problem", "{problem}" }
                                }
                                form {
                                    onsubmit: move |event| {
                                        event.prevent_default();

                                        if saving() || typed().trim().is_empty() {
                                            return;
                                        }

                                        let judged = matching::judge_reviewed(
                                            question,
                                            &accepted,
                                            records.alternatives(number),
                                            &typed(),
                                        );
                                        let produced = {
                                            let mut exam = exam.write();
                                            let running = exam.as_mut();

                                            running.and_then(|running| {
                                                running.answer(judged);
                                                running.record(storage::now())
                                            })
                                        };

                                        let ended = produced.is_some();

                                        answered.set(typed());
                                        typed.set(String::new());
                                        problem.set(None);
                                        feedback.set(Some((number, judged)));
                                        finished.set(produced);

                                        if ended {
                                            save.call(false);
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
                    }
                } else {
                    if let Some(problem) = problem() {
                        p { class: "problem", "{problem}" }
                    }
                    if finished().is_some() {
                        button {
                            class: "primary",
                            disabled: saving(),
                            onclick: move |_| save.call(true),
                            "See result"
                        }
                    } else {
                        p { class: "problem", "This Exam could not be set." }
                    }
                    nav { class: "menu",
                        Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
                    }
                }
            }
        }
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            {body}
        }
    }
}
