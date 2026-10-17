use dioxus::prelude::*;

use domain::Coverage;

/// What Coverage counts, in the domain's own words.
const RULE: &str = "A question never answered wrong is Cleared by one correct \
                    answer; after a wrong answer it takes three correct answers \
                    in a row. Coverage is the share of the Civics Test Questions \
                    that are Cleared.";

/// The Coverage figure with the one line that says what it counts.
///
/// The main menu and the stats screen both render this, so the sentence cannot
/// drift between them. It names the rule rather than the figure: an Applicant
/// who has answered a question wrong sees that two correct answers in a row do
/// not yet Clear it, and why.
#[component]
pub fn CoverageFigure(coverage: Coverage) -> Element {
    rsx! {
        p { class: "coverage-figure", "{coverage}" }
        p { class: "coverage-rule", "{RULE}" }
    }
}
