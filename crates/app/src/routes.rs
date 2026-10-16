use dioxus::prelude::*;

use crate::components::FirstRun;
use crate::screens::{MainMenu, Study};

/// Every screen the Applicant can reach.
///
/// The screens land with the tickets that give them their behavior; the main
/// menu is where they all hang off.
#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    MainMenu {},

    #[route("/study")]
    Study {},
}

/// The application root: the route tree, behind the first-run gate.
#[component]
pub fn App() -> Element {
    rsx! {
        FirstRun {
            Router::<Route> {}
        }
    }
}
