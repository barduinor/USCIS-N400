//! The Accepted Answers under a verdict, with the Explanation on request.
//!
//! The answers appear as soon as an answer has been judged, whether it was
//! right or wrong. The Explanation stays behind its own control: the answers
//! are what the Applicant compares against, and the Explanation is longer
//! reading for when they want it.

use dioxus::prelude::*;
use domain::AcceptedAnswer;

/// One question's Accepted Answers, and its Explanation when one exists.
#[component]
pub fn Answers(answers: Vec<AcceptedAnswer>, explanation: Option<String>) -> Element {
    let mut explanation_shown = use_signal(|| false);

    rsx! {
        aside { class: "answers",
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
                if explanation_shown() {
                    h2 { "Explanation" }
                    p { "{explanation}" }
                } else {
                    button {
                        class: "quiet",
                        r#type: "button",
                        onclick: move |_| explanation_shown.set(true),
                        "Show the explanation"
                    }
                }
            }
        }
    }
}
