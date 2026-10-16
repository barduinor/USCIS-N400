use dioxus::prelude::*;

/// The application root.
///
/// The route tree and the screens hang off this component as they land; the
/// Applicant reaches every screen from the main menu.
#[component]
pub fn App() -> Element {
    rsx! {
        main { h1 { "USCIS N-400 Civics Study" } }
    }
}
