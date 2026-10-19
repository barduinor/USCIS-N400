use dioxus::prelude::*;

use domain::{ExamRecord, Track, Verdict, answers, matching};

use crate::components::{Answers, Field, OverrideControl, use_before_unload};
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
/// Applicant asks. After a wrong verdict it offers the Override, which counts
/// that answer as correct in the Exam's own score, outcome, and saved record.
/// It then waits for the Applicant: the next question for a running Exam, or
/// the result for the one that just ended. Answers go to the Exam, never to
/// the Answer Log.
///
/// The running Exam has no in-app exit: the result screen is the way out, and
/// the answering and feedback states offer only the next step. The error and
/// loading states carry a `Back to the menu` link, so a failed start is not a
/// trap. While an Exam is in progress a `beforeunload` prompt asks before the
/// page is left; it is best effort, since the browser's back button and tab
/// close cannot be blocked, and an Exam left that way is discarded.
#[component]
pub fn Exam() -> Element {
    let records = match questions::records() {
        Ok(records) => records,
        Err(error) => {
            return rsx! {
                main { class: "screen",
                    h1 { "USCIS N-400 Civics Study" }
                    p { class: "problem", "The Civics Test Questions could not be read: {error}" }
                    {back_to_menu()}
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
                    {back_to_menu()}
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
    let mut overridden = use_signal(|| Option::<u16>::None);
    let mut overriding = use_signal(|| false);
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

    // The browser asks before leaving a page with a running Exam.
    use_before_unload(move || exam.read().is_some() && finished.read().is_none());

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
            {back_to_menu()}
        },
        (_, None) => rsx! {
            p { class: "loading", "Setting the Exam…" }
            {back_to_menu()}
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
                                disabled: saving() || overriding(),
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
                                disabled: overriding(),
                                onclick: move |_| {
                                    feedback.set(None);
                                    answered.set(String::new());
                                    overridden.set(None);
                                    problem.set(None);
                                },
                                "Next question"
                            }
                        }
                    }
                    if judged == Verdict::Wrong {
                        div { class: "override",
                            OverrideControl {
                                counts_correct: overridden() == Some(number),
                                overriding: saving() || overriding(),
                                on_override: move |_| {
                                    if saving() || overriding() {
                                        return;
                                    }

                                    let Some((number, judged)) = feedback() else {
                                        return;
                                    };
                                    if judged != Verdict::Wrong || overridden() == Some(number) {
                                        return;
                                    }

                                    let Some(mut running) = exam.read().clone() else {
                                        return;
                                    };
                                    let previous = finished();
                                    if !running.override_last() {
                                        return;
                                    }
                                    let record = running.record(
                                        previous
                                            .as_ref()
                                            .map_or_else(storage::now, |record| record.at),
                                    );
                                    let was_saved = saved();

                                    overriding.set(true);
                                    problem.set(None);
                                    spawn(async move {
                                        let result = match (&record, &previous) {
                                            (Some(record), Some(_)) if was_saved => {
                                                storage::exam::replace_latest(record).await
                                            }
                                            (Some(record), _) => {
                                                storage::exam::append(record).await
                                            }
                                            (None, Some(_)) if was_saved => {
                                                storage::exam::remove_latest().await
                                            }
                                            (None, _) => Ok(()),
                                        };

                                        match result {
                                            Ok(()) => {
                                                exam.set(Some(running));
                                                match record {
                                                    Some(record) => {
                                                        finished.set(Some(record));
                                                        saved.set(true);
                                                        overridden.set(Some(number));
                                                    }
                                                    None if previous.is_some() => {
                                                        finished.set(None);
                                                        saved.set(false);
                                                        overridden.set(None);
                                                        feedback.set(None);
                                                        typed.set(String::new());
                                                        answered.set(String::new());
                                                    }
                                                    None => {
                                                        overridden.set(Some(number));
                                                    }
                                                }
                                                problem.set(None);
                                            }
                                            Err(error) => problem.set(Some(error.to_string())),
                                        }

                                        overriding.set(false);
                                    });
                                },
                            }
                        }
                    }
                } else if let Some(number) = current {
                    match records.question(number) {
                        None => rsx! {
                            p { class: "problem", "Civics Test Question {number} could not be read." }
                            {back_to_menu()}
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

                                        if saving() || overriding() || typed().trim().is_empty() {
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
                                        overridden.set(None);
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
                                        disabled: saving() || overriding() || typed().trim().is_empty(),
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
                            disabled: saving() || overriding(),
                            onclick: move |_| save.call(true),
                            "See result"
                        }
                    } else {
                        p { class: "problem", "This Exam could not be set." }
                    }
                    {back_to_menu()}
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

/// The way out of a failed or loading Exam.
fn back_to_menu() -> Element {
    rsx! {
        nav { class: "menu",
            Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
        }
    }
}
