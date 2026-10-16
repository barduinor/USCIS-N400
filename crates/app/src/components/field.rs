use dioxus::prelude::*;

/// One labelled text field of a form.
///
/// The value lives with the screen that owns the form; this component only
/// renders it and reports what the Applicant types.
#[component]
pub fn Field(
    label: String,
    name: String,
    value: String,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "field", r#for: "{name}",
            span { class: "field-label", "{label}" }
            input {
                id: "{name}",
                name: "{name}",
                r#type: "text",
                autocomplete: "off",
                value: "{value}",
                oninput: move |event| on_input.call(event.value()),
            }
        }
    }
}
