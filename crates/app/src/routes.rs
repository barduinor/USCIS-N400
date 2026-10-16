use dioxus::prelude::*;

use crate::screens::MainMenu;

/// Every screen the Applicant can reach.
///
/// The screens land with the tickets that give them their behavior; the main
/// menu is where they all hang off.
#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    MainMenu {},
}

/// The application root: the route tree, and nothing else.
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
