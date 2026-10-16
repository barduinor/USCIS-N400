use dioxus::prelude::*;

use domain::Coverage;

/// What Coverage counts, in the domain's own words.
const RULE: &str = "A question is Cleared by three correct answers in a row, \
                    and Coverage is the share of the Civics Test Questions \
                    that are Cleared.";

/// The Coverage figure with the one line that says what it counts.
///
/// The main menu and the stats screen both render this, so the sentence cannot
/// drift between them. It names the rule rather than the figure: an Applicant
/// who answered a question correctly once and still reads `0%` sees that a
/// question is Cleared by three correct answers in a row.
#[component]
pub fn CoverageFigure(coverage: Coverage) -> Element {
    rsx! {
        p { class: "coverage-figure", "{coverage}" }
        p { class: "coverage-rule", "{RULE}" }
    }
}
