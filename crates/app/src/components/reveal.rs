use dioxus::prelude::*;

use domain::AcceptedAnswer;

use crate::storage;

/// The reveal under a verdict: the Accepted Answers, the Explanation when the
/// question carries one, and, after a wrong verdict, the Override.
///
/// The screen mounts this only once it has judged an answer, so nothing here is
/// on screen before a verdict. The Override marks the log's own entry and
/// reports back that this attempt counts correct; it never re-judges the answer
/// and never changes the Accepted Answers.
#[component]
pub fn Reveal(
    answers: Vec<AcceptedAnswer>,
    explanation: Option<String>,
    wrong: bool,
    overridden: bool,
    on_overridden: EventHandler<()>,
) -> Element {
    let mut revealed = use_signal(|| false);
    let mut overriding = use_signal(|| false);
    let mut problem = use_signal(|| Option::<String>::None);

    if !revealed() {
        return rsx! {
            button {
                class: "primary",
                onclick: move |_| revealed.set(true),
                "Show the answers"
            }
        };
    }

    rsx! {
        aside { class: "reveal",
            h2 { "Accepted Answers" }
            if answers.is_empty() {
                p { "Your Profile does not hold this answer yet." }
            } else {
                ul {
                    for answer in answers {
                        li { "{answer}" }
                    }
                }
            }
            if let Some(explanation) = explanation {
                h2 { "Explanation" }
                p { "{explanation}" }
            }
            if wrong {
                if overridden {
                    p { "Counts as correct for this attempt." }
                } else {
                    button {
                        class: "primary",
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
                }
            }
            if let Some(problem) = problem() {
                p { class: "problem", "{problem}" }
            }
        }
    }
}
