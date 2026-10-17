//! The Test Update values, from the USCIS page to the committed record.
//!
//! The four national officeholders live on `uscis.gov/citizenship/testupdates`,
//! the page M-1778 points at for answers that change. One host-side fetch at
//! release time writes the values, the page they came from, and the fetch date
//! into the shipped data; the application never fetches them at runtime.
//!
//! Extraction is deliberately strict: the 2025 accordion is read by its heading,
//! each value by the question number the 2025 instrument fixes, and every one of
//! the four must be present and non-empty. A page that does not yield all four
//! is an error, never a partial record.

use domain::{TestUpdateValues, TestUpdatesRecord};

use crate::{Error, Result};

/// The page USCIS publishes the current Test Updates on.
pub const SOURCE_URL: &str = "https://www.uscis.gov/citizenship/testupdates";

/// The committed Test Update record this crate writes.
pub const TEST_UPDATES_PATH: &str = "crates/app/data/test-updates.json";

/// The heading that opens the 2025 test's accordion on the page.
const PANEL_HEADING: &str = "Civics Test (2025 Naturalization Civics Test) Updates";

/// The 2025 test's own question numbers for the four Test Updates (M-1778).
const PRESIDENT_QUESTION: u16 = 38;
const VICE_PRESIDENT_QUESTION: u16 = 39;
const SPEAKER_QUESTION: u16 = 30;
const CHIEF_JUSTICE_QUESTION: u16 = 57;

/// How the generator identifies itself to USCIS.
const USER_AGENT: &str =
    "USCIS-N400 test-updates generator (+https://github.com/barduinor/USCIS-N400)";

/// Fetches the test-updates page.
pub fn fetch(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(core::time::Duration::from_secs(30))
        .build()
        .map_err(Error::custom_from_err)?;
    let response = client.get(url).send().map_err(Error::custom_from_err)?;
    let status = response.status();

    if !status.is_success() {
        return Err(Error::from(format!(
            "the test-updates page answered with HTTP {status}"
        )));
    }

    response.text().map_err(Error::custom_from_err)
}

/// Reads the four Test Updates out of the page's 2025 accordion.
pub fn record_from_html(html: &str, source: &str, fetched: &str) -> Result<TestUpdatesRecord> {
    let panel = panel(html)?;
    let president = first_answer(panel, PRESIDENT_QUESTION)?;
    let vice_president = first_answer(panel, VICE_PRESIDENT_QUESTION)?;
    let speaker_of_the_house = first_answer(panel, SPEAKER_QUESTION)?;
    let chief_justice = first_answer(panel, CHIEF_JUSTICE_QUESTION)?;

    Ok(TestUpdatesRecord {
        source: source.to_string(),
        fetched: fetched.to_string(),
        values: TestUpdateValues {
            president,
            vice_president,
            speaker_of_the_house,
            chief_justice,
        },
    })
}

/// Serializes the record exactly as `crates/app/data/test-updates.json` is
/// committed.
pub fn record_json(record: &TestUpdatesRecord) -> Result<String> {
    let mut json = serde_json::to_string_pretty(record)?;

    json.push('\n');

    Ok(json)
}

/// The day the page is read, as `YYYY-MM-DD`.
pub fn today() -> String {
    let date = time::OffsetDateTime::now_utc().date();

    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

// region:    --- Support

/// The 2025 test's accordion panel, from its heading to its closing tag.
fn panel(html: &str) -> Result<&str> {
    let heading = html.find(PANEL_HEADING).ok_or_else(|| {
        Error::from(format!(
            "the page no longer carries the heading `{PANEL_HEADING}`"
        ))
    })?;
    let opening = "<div class=\"accordion__panel\">";
    let opening = html[heading..]
        .find(opening)
        .map(|offset| heading + offset + opening.len())
        .ok_or("the 2025 heading is not followed by an accordion panel")?;

    panel_end(&html[opening..])
        .map(|end| &html[opening..opening + end])
        .ok_or_else(|| Error::from("the 2025 accordion panel is not closed"))
}

/// Where the panel opened at the start of `panel` closes, counting nested
/// `div` elements.
fn panel_end(panel: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut rest = panel;

    while let Some(open) = rest.find('<') {
        rest = &rest[open..];

        let end = rest.find('>')?;
        let tag = &rest[1..end];

        if tag.starts_with("div") || tag.starts_with("div ") {
            depth += 1;
        } else if tag.starts_with("/div") {
            depth -= 1;

            if depth == 0 {
                return Some(panel.len() - rest.len());
            }
        }

        rest = &rest[end + 1..];
    }

    None
}

/// The first Accepted Answer listed under one question in the panel.
///
/// The question's number is the 2025 instrument's own, so no question text is
/// matched. A missing question, a question without an answer list, or an empty
/// or placeholder answer is an error.
fn first_answer(panel: &str, number: u16) -> Result<String> {
    let marker = format!("<p><strong>{number}.");
    let occurrences = panel.matches(&marker).count();

    if occurrences != 1 {
        return Err(Error::from(format!(
            "question {number} appears {occurrences} times in the 2025 Test Updates; it must \
             appear once"
        )));
    }

    let question = &panel[panel.find(&marker).ok_or_else(|| {
        Error::from(format!("question {number} is not in the 2025 Test Updates"))
    })?..];
    let rest = question
        .find("</strong>")
        .map(|end| &question[end + "</strong>".len()..])
        .ok_or_else(|| Error::from(format!("question {number} has no closing bold tag")))?;
    let list_start = rest
        .find("<ul>")
        .ok_or_else(|| Error::from(format!("question {number} has no answer list")))?;

    if rest[..list_start].contains("<p") {
        return Err(Error::from(format!(
            "question {number} carries no answer list of its own"
        )));
    }

    let list = &rest[list_start + "<ul>".len()..];
    let item_start = list
        .find("<li>")
        .ok_or_else(|| Error::from(format!("question {number} lists no Accepted Answers")))?;
    let item = &list[item_start + "<li>".len()..];
    let item_end = item
        .find("</li>")
        .ok_or_else(|| Error::from(format!("question {number}'s first answer is not closed")))?;
    let value = text_of(&item[..item_end]);

    if value.is_empty() {
        return Err(Error::from(format!(
            "question {number}'s first Accepted Answer is empty"
        )));
    }
    if is_placeholder(&value) {
        return Err(Error::from(format!(
            "question {number}'s first Accepted Answer is a placeholder: {value}"
        )));
    }

    Ok(value)
}

/// The visible text of an HTML fragment, with tags removed, entities decoded,
/// and whitespace collapsed.
fn text_of(fragment: &str) -> String {
    let mut text = String::with_capacity(fragment.len());
    let mut in_tag = false;

    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }

    decode_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The characters an HTML entity stands for, where it names one.
fn decode_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(ampersand) = rest.find('&') {
        decoded.push_str(&rest[..ampersand]);

        let entity = &rest[ampersand..];
        let Some(end) = entity.find(';') else {
            decoded.push('&');
            rest = &entity[1..];

            continue;
        };

        match decode_entity(&entity[1..end]) {
            Some(character) => decoded.push(character),
            None => {
                decoded.push('&');
                decoded.push_str(&entity[1..end]);
                decoded.push(';');
            }
        }

        rest = &entity[end + 1..];
    }

    decoded.push_str(rest);

    decoded
}

/// The character one entity body (`amp`, `#39`, or `#x27`) stands for.
fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => {
            let digits = entity.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse().ok()?,
            };

            char::from_u32(code)
        }
    }
}

/// Whether USCIS left a pointer or a blank where the value belongs.
fn is_placeholder(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();

    lowered.starts_with("answers will vary")
        || lowered.contains("testupdates")
        || value.contains(['[', ']'])
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    /// The 2025 accordion, with the 2008 accordion before it: the values and
    /// the question numbers differ between the two instruments, so reading the
    /// wrong panel would be visible.
    const PAGE: &str = r#"<html><body>
        <h4 class="accordion__header">Civics Test (2008 Version) Updates</h4>
        <div class="accordion__panel">
            <p><strong>28. What is the name of the President of the United States now?*</strong></p>
            <ul><li>Old President</li></ul>
            <p><strong>40. Who is the Chief Justice of the United States now?</strong></p>
            <ul><li>Old Chief Justice</li></ul>
        </div>
        <h4 class="accordion__header">Civics Test (2025 Naturalization Civics Test) Updates</h4>
        <div class="accordion__panel">
            <p><strong>23. Who is one of your state's U.S. Senators now?*</strong></p>
            <ul><li>Answers will vary. Visit <a href="https://www.congress.gov/members">senate.gov</a>.</li></ul>
            <p><strong>30. What is the name of the Speaker of the House of Representatives now?</strong></p>
            <ul><li>Mike&nbsp;Johnson</li><li>Johnson</li></ul>
            <p><strong>38. What is the name of the President of the United States now?</strong>*</p>
            <ul><li>Donald J. Trump</li><li>Donald Trump</li><li>Trump</li></ul>
            <p><strong>39. What is the name of the Vice President of the United States now?</strong></p>
            <ul><li>JD Vance</li><li>Vance</li></ul>
            <p><strong>57. Who is the Chief Justice of the United States now</strong>?</p>
            <ul><li>John&nbsp;Roberts</li><li>John G. Roberts, Jr.</li></ul>
        </div>
    </body></html>"#;

    #[test]
    fn test_test_updates_reads_the_2025_accordion() -> Result<()> {
        // -- Exec
        let record = record_from_html(PAGE, SOURCE_URL, "2026-10-16")?;

        // -- Check
        assert_eq!(record.source, SOURCE_URL);
        assert_eq!(record.fetched, "2026-10-16");
        assert_eq!(
            record.values,
            TestUpdateValues {
                president: "Donald J. Trump".to_string(),
                vice_president: "JD Vance".to_string(),
                speaker_of_the_house: "Mike Johnson".to_string(),
                chief_justice: "John Roberts".to_string(),
            },
            "only the 2025 panel answers, and only the first listed answer"
        );

        Ok(())
    }

    #[test]
    fn test_test_updates_refuses_a_partial_page() {
        // -- Setup & Fixtures
        let without_chief_justice = PAGE.replace(
            "<p><strong>57. Who is the Chief Justice of the United States now</strong>?</p>\n            <ul><li>John&nbsp;Roberts</li><li>John G. Roberts, Jr.</li></ul>",
            "",
        );

        // -- Exec
        let result = record_from_html(&without_chief_justice, SOURCE_URL, "2026-10-16");

        // -- Check
        assert!(
            result.is_err(),
            "a page missing one of the four values must not produce a record"
        );
    }

    #[test]
    fn test_test_updates_refuses_a_placeholder_answer() {
        // -- Setup & Fixtures
        let placeholder = PAGE.replace("Donald J. Trump", "Answers will vary.");

        // -- Exec
        let result = record_from_html(&placeholder, SOURCE_URL, "2026-10-16");

        // -- Check
        assert!(
            result.is_err(),
            "a pointer in place of the value must not become the shipped value"
        );
    }

    #[test]
    fn test_test_updates_refuses_an_empty_answer() {
        // -- Setup & Fixtures
        let empty = PAGE.replace("Donald J. Trump", "   ");

        // -- Exec
        let result = record_from_html(&empty, SOURCE_URL, "2026-10-16");

        // -- Check
        assert!(
            result.is_err(),
            "an empty answer must not become the shipped value"
        );
    }

    #[test]
    fn test_test_updates_refuses_a_page_without_the_2025_heading() {
        // -- Exec
        let result = record_from_html("<html></html>", SOURCE_URL, "2026-10-16");

        // -- Check
        assert!(result.is_err(), "a page without the 2025 panel must fail");
    }

    #[test]
    fn test_test_updates_decodes_html_entities() {
        // -- Exec & Check
        assert_eq!(text_of("Mike&nbsp;Johnson"), "Mike Johnson");
        assert_eq!(text_of("&#39;Quoted&#39; &amp; true"), "'Quoted' & true");
        assert_eq!(text_of("<span>John</span> Roberts"), "John Roberts");
    }

    #[test]
    fn test_test_updates_today_is_an_iso_date() {
        // -- Exec
        let today = today();

        // -- Check
        assert_eq!(today.len(), 10);
        assert!(today.as_bytes()[4] == b'-' && today.as_bytes()[7] == b'-');
        assert!(
            today
                .split('-')
                .all(|part| part.chars().all(|character| character.is_ascii_digit()))
        );
    }
}

// endregion: --- Tests
