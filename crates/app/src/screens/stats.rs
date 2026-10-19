use dioxus::prelude::*;

use domain::{Coverage, Section};

use crate::components::{CoverageFigure, use_refresh_on_resume};
use crate::questions;
use crate::routes::Route;
use crate::storage;

/// Where the Applicant stands: Coverage over the whole scope, then each Section
/// — opening one shows its Topics — with the Exams taken below.
///
/// Every figure carries a percentage and a count, and the Sections and their
/// Topics partition the scope exactly: the counts add up to the figure above
/// them.
#[component]
pub fn Stats() -> Element {
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

    let mut profile = use_resource(storage::profile::load);
    let mut log = use_resource(storage::answer_log::entries_verified);
    let mut exams = use_resource(storage::exam::exams_verified);
    let mut opened = use_signal(|| Option::<Section>::None);

    use_refresh_on_resume(move || {
        profile.restart();
        log.restart();
        exams.restart();
    });

    let sections = Section::ALL
        .into_iter()
        .map(|section| {
            let is_open = opened() == Some(section);

            (
                section,
                is_open,
                if is_open {
                    "section-group is-open"
                } else {
                    "section-group"
                },
            )
        })
        .collect::<Vec<_>>();

    let breakdown = match (&*profile.read(), &*log.read()) {
        (Some(Ok(Some(profile))), Some(Ok(log))) => rsx! {
            section { class: "coverage",
                h2 { "Coverage" }
                CoverageFigure { coverage: Coverage::from_log(records, profile, log) }
            }

            ul { class: "sections",
                for (section, is_open, group_class) in sections {
                    li { class: group_class,
                        button {
                            class: "section-row",
                            r#type: "button",
                            aria_expanded: if is_open { "true" } else { "false" },
                            onclick: move |_| {
                                opened.set(if opened() == Some(section) { None } else { Some(section) });
                            },
                            span { class: "section-name", "{section}" }
                            span { class: "figure",
                                "{Coverage::of_section(records, profile, log, section)}"
                            }
                            span { class: "chevron", aria_hidden: "true", "›" }
                        }
                        if is_open {
                            div { class: "section-panel",
                                p { class: "topics-heading", "Topics" }
                                ul { class: "topics",
                                    for topic in section.topics() {
                                        li { class: "topic-row",
                                            span { "{topic}" }
                                            span { class: "figure",
                                                "{Coverage::of_topic(records, profile, log, *topic)}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        (Some(Err(error)), _) | (_, Some(Err(error))) => rsx! {
            p { class: "problem", "The progress could not be read: {error}" }
            div { class: "actions",
                button {
                    class: "primary",
                    r#type: "button",
                    onclick: move |_| {
                        profile.restart();
                        log.restart();
                    },
                    "Try again"
                }
                Link { to: Route::Profile {}, class: "menu-item", "Export from the Profile" }
            }
        },
        _ => rsx! {
            p { class: "loading", "Reading the Answer Log…" }
        },
    };

    let history = match &*exams.read() {
        None => rsx! {
            p { class: "loading", "Reading the Exams…" }
        },
        Some(Err(error)) => rsx! {
            p { class: "problem", "The Exams could not be read: {error}" }
            button {
                class: "primary",
                r#type: "button",
                onclick: move |_| exams.restart(),
                "Try again"
            }
        },
        Some(Ok(exams)) if exams.is_empty() => rsx! {
            p { class: "hint", "No Exam has been taken yet." }
        },
        Some(Ok(exams)) => rsx! {
            ul { class: "exams",
                for exam in exams.iter().rev() {
                    li { class: "exam-row",
                        span { "{exam.outcome}" }
                        span { class: "figure",
                            "{exam.correct_percentage()}% ({exam.correct_count} of {exam.questions.len()} correct)"
                        }
                        span { class: "track", "{exam.track}" }
                    }
                }
            }
        },
    };

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            {breakdown}

            section { class: "history",
                h2 { "Exam history" }
                {history}
            }

            nav { class: "menu",
                Link { to: Route::MainMenu {}, class: "menu-item", "Back to the menu" }
            }
        }
    }
}
