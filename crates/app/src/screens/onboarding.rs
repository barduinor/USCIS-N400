use dioxus::prelude::*;

use domain::Profile;

use crate::components::ProfileForm;
use crate::storage::profile;

/// The first run: the Applicant's own facts, before any Civics Test Question.
#[component]
pub fn Onboarding(on_saved: EventHandler<()>) -> Element {
    let answers = use_signal(Profile::default);
    let mut problem = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);

    let save = move |_| {
        let profile = crate::components::profile_form::as_profile(&answers());

        if !profile.has_state() {
            problem.set(Some("A state is required.".to_string()));

            return;
        }

        problem.set(None);
        saving.set(true);

        spawn(async move {
            match profile::save(&profile).await {
                Ok(()) => on_saved.call(()),
                Err(error) => {
                    problem.set(Some(format!("The Profile could not be stored: {error}")));
                    saving.set(false);
                }
            }
        });
    };

    let saving = saving();
    let problem = problem();
    let has_problem = problem.is_some();
    let problem_line = problem.unwrap_or_default();

    rsx! {
        main { class: "screen",
            h1 { "USCIS N-400 Civics Study" }
            p { class: "hint",
                "The Civics Test asks about your own state and district. Every field except the state can wait; the Profile stays editable."
            }

            ProfileForm { profile: answers }

            button {
                class: "primary",
                r#type: "button",
                disabled: saving,
                onclick: save,
                if saving {
                    "Storing the Profile…"
                } else {
                    "Save the Profile"
                }
            }

            if has_problem {
                p { class: "problem", "{problem_line}" }
            }
        }
    }
}
