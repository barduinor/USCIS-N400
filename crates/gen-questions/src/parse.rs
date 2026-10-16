//! Extraction of the Civics Test Questions from the M-1778 text.
//!
//! Extraction is mechanical: the Section and Topic headings, the numbered
//! questions, the answer bullets, the asterisks that mark the 65/20 Special
//! Consideration questions, the required-answer counts, page furniture, and the
//! lines the source wrapped. The questions themselves are `domain`'s type; this
//! module only decides where each field's text comes from. Judgment calls about
//! USCIS's own text — the square-bracket annotations and the comma lists —
//! belong to `overrides`, which is handed the questions this module extracts.

use domain::{AcceptedAnswer, Question, Section, Topic};

use crate::{Error, Result};

/// Parses the M-1778 text into Civics Test Questions, in document order.
pub fn parse_text(text: &str) -> Result<Vec<Question>> {
    let mut parser = Parser::default();

    for line in text.lines() {
        parser.push_line(line)?;
    }

    parser.finish()
}

// region:    --- Support

/// The line-by-line state machine that walks the question list.
#[derive(Debug, Default)]
struct Parser {
    section: Option<Section>,
    topic: Option<Topic>,
    questions: Vec<Question>,
    current: Option<Question>,
    last_target: LastTarget,
    after_blank: bool,
}

/// What a wrapped line continues.
#[derive(Debug, Default)]
enum LastTarget {
    #[default]
    None,
    Question,
    Answer(usize),
}

/// How a wrapped line joins the line it continues.
#[derive(Debug, Clone, Copy)]
enum Join {
    /// No gap: the line is a superscript suffix such as `th Amendment`.
    Tight,

    /// One space: the source wrapped a sentence.
    Spaced,
}

/// The words that say how many Accepted Answers a question demands, and what
/// they demand. USCIS underlines the word in the question; the underline is not
/// text, so the word is read from the sentence that carries it.
const COUNT_WORDS: [(&str, u16); 4] = [("one", 1), ("two", 2), ("three", 3), ("five", 5)];

/// The sentences that carry the demanded count.
const COUNT_PHRASES: [&str; 5] = ["name ", "describe ", "what is ", "what are ", "what was "];

/// The Sections of the 2025 civics test, as M-1778 prints their headings.
const SECTIONS: [(&str, Section); 3] = [
    ("AMERICAN GOVERNMENT", Section::AmericanGovernment),
    ("AMERICAN HISTORY", Section::AmericanHistory),
    ("SYMBOLS AND HOLIDAYS", Section::SymbolsAndHolidays),
];

const PAGE_FOOTER: &str = "uscis.gov/citizenship";

const ANSWER_BULLET: char = '•';

impl Parser {
    /// Classifies one source line and folds it into the question being built.
    fn push_line(&mut self, raw_line: &str) -> Result<()> {
        let line = strip_page_footer(raw_line.trim());

        if line.is_empty() {
            self.after_blank = true;

            return Ok(());
        }
        if is_page_footer(line) {
            return Ok(());
        }
        if let Some(section) = section_heading(line)? {
            self.close_question()?;
            self.section = Some(section);
            self.after_blank = false;

            return Ok(());
        }
        if let Some(topic) = topic_heading(line)? {
            self.close_question()?;
            self.topic = Some(topic);
            self.after_blank = false;

            return Ok(());
        }
        if let Some((number, text)) = question_start(line) {
            self.close_question()?;
            self.start_question(number, text)?;
            self.after_blank = false;

            return Ok(());
        }
        if let Some(answer) = answer_bullet(line) {
            self.push_answer(answer)?;
            self.after_blank = false;

            return Ok(());
        }

        self.push_continuation(line);
        self.after_blank = false;

        Ok(())
    }

    /// Opens a question; the asterisk of the 65/20 questions sits at its end.
    fn start_question(&mut self, number: u16, text: &str) -> Result<()> {
        let (text, starred) = match text.strip_suffix('*') {
            Some(text) => (text.trim_end(), true),
            None => (text, false),
        };

        let section = self.section.ok_or("question before any Section heading")?;
        let topic = self.topic.ok_or("question before any Topic heading")?;

        if topic.section() != section {
            return Err(format!("topic {topic} sits outside Section {section}").into());
        }

        self.current = Some(Question {
            number,
            section,
            topic,
            question: text.to_string(),
            accepted_answers: Vec::new(),
            answers_required: answers_required(text),
            starred,
            explanation: None,
        });
        self.last_target = LastTarget::Question;

        Ok(())
    }

    /// Adds one Accepted Answer to the open question.
    fn push_answer(&mut self, answer: &str) -> Result<()> {
        let index = {
            let question = self
                .current
                .as_mut()
                .ok_or("answer bullet before any question")?;

            question.accepted_answers.push(AcceptedAnswer::from(answer));
            question.accepted_answers.len() - 1
        };

        self.last_target = LastTarget::Answer(index);

        Ok(())
    }

    /// Folds a wrapped line into the question text or the answer it continues.
    ///
    /// A line that follows a blank line continues nothing: inside a question
    /// block that is USCIS prose such as the tribe-list note, and outside one
    /// it is the document's preamble. Both are not records.
    fn push_continuation(&mut self, line: &str) {
        if self.after_blank {
            return;
        }

        let join = if is_ordinal_suffix(line) {
            Join::Tight
        } else {
            Join::Spaced
        };
        let Some(question) = self.current.as_mut() else {
            return;
        };

        match self.last_target {
            LastTarget::Question => {
                question.question = join.apply(&question.question, line);
            }
            LastTarget::Answer(index) => {
                if let Some(answer) = question.accepted_answers.get_mut(index) {
                    *answer = AcceptedAnswer::from(join.apply(answer.as_str(), line));
                }
            }
            LastTarget::None => {}
        }
    }

    /// Closes the open question, checking that it carries its answers.
    fn close_question(&mut self) -> Result<()> {
        let Some(question) = self.current.take() else {
            return Ok(());
        };

        if question.accepted_answers.is_empty() {
            return Err(format!("question {} has no Accepted Answers", question.number).into());
        }
        if let Some(previous) = self.questions.last()
            && question.number <= previous.number
        {
            return Err(format!(
                "question {} is out of order, after question {}",
                question.number, previous.number
            )
            .into());
        }

        self.questions.push(question);
        self.last_target = LastTarget::None;

        Ok(())
    }

    /// Closes the last question and returns everything extracted.
    fn finish(mut self) -> Result<Vec<Question>> {
        self.close_question()?;

        if self.questions.is_empty() {
            return Err("no Civics Test Questions found".into());
        }

        Ok(self.questions)
    }
}

impl Join {
    fn apply(self, text: &str, addition: &str) -> String {
        match self {
            Join::Tight => format!("{text}{addition}"),
            Join::Spaced => format!("{text} {addition}"),
        }
    }
}

/// Returns the Section behind an all-caps heading line, if the line is one.
fn section_heading(line: &str) -> Result<Option<Section>> {
    let is_heading = line.contains(char::is_alphabetic)
        && line
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_whitespace());

    if !is_heading {
        return Ok(None);
    }

    SECTIONS
        .iter()
        .find(|(printed, _)| *printed == line)
        .map(|(_, section)| Some(*section))
        .ok_or_else(|| Error::from(format!("unknown Section heading: {line}")))
}

/// Returns the Topic behind a `A: Topic` heading line, if the line is one.
fn topic_heading(line: &str) -> Result<Option<Topic>> {
    let Some((letter, name)) = line.split_once(':') else {
        return Ok(None);
    };

    if letter.len() != 1 || !letter.chars().all(|c| c.is_ascii_uppercase()) {
        return Ok(None);
    }

    let name = name.trim();

    if name.is_empty() {
        return Ok(None);
    }

    topic_named(name)
        .map(Some)
        .ok_or_else(|| Error::from(format!("unknown Topic heading: {line}")))
}

/// The Topic USCIS names `name`, if the 2025 test has one.
fn topic_named(name: &str) -> Option<Topic> {
    Section::ALL
        .iter()
        .flat_map(|section| section.topics())
        .copied()
        .find(|topic| topic.to_string() == name)
}

/// Returns the number and text of a line that opens a question.
fn question_start(line: &str) -> Option<(u16, &str)> {
    let digits_end = line.find(|c: char| !c.is_ascii_digit())?;
    let (digits, rest) = line.split_at(digits_end);

    let text = rest
        .strip_prefix('.')?
        .strip_prefix(char::is_whitespace)?
        .trim();

    if digits.is_empty() || text.is_empty() {
        return None;
    }

    Some((digits.parse().ok()?, text))
}

/// Returns the text of an answer bullet line, if the line is one.
fn answer_bullet(line: &str) -> Option<&str> {
    let answer = line.strip_prefix(ANSWER_BULLET)?.trim();

    (!answer.is_empty()).then_some(answer)
}

/// Answers a question demands, from the sentence that carries the count word.
///
/// A question that carries no count word demands one answer, and a mention
/// inside a statement (`There are three branches of government. Why?`) is not a
/// demand, which is why only the demand sentences are read.
fn answers_required(question: &str) -> u16 {
    let text = question.to_lowercase();

    for phrase in COUNT_PHRASES {
        let Some((_, rest)) = text.split_once(phrase) else {
            continue;
        };
        let rest = rest.trim_start();
        let rest = rest.strip_prefix("the ").unwrap_or(rest);
        let Some(word) = rest.split_whitespace().next() else {
            continue;
        };
        let word = word.trim_end_matches(|c: char| !c.is_ascii_alphabetic());

        if let Some((_, count)) = COUNT_WORDS.iter().find(|(count, _)| *count == word) {
            return *count;
        }
    }

    1
}

/// The page furniture M-1778 prints at the foot of every page.
fn is_page_footer(line: &str) -> bool {
    let Some(prefix) = line.strip_suffix(PAGE_FOOTER) else {
        return false;
    };
    let Some((page, total)) = prefix.trim().split_once(" of ") else {
        return false;
    };

    is_page_number(page) && is_page_number(total)
}

/// Cuts the page furniture off a line, where the extraction merged it into the
/// line it followed: `• The Civil War 14 of 19 uscis.gov/citizenship`.
fn strip_page_footer(line: &str) -> &str {
    let Some(head) = line.strip_suffix(PAGE_FOOTER) else {
        return line;
    };
    let Some((head, total)) = head.trim_end().rsplit_once(' ') else {
        return line;
    };
    let Some((head, of)) = head.rsplit_once(' ') else {
        return line;
    };
    let Some((content, page)) = head.rsplit_once(' ') else {
        return line;
    };

    if of != "of" || !is_page_number(page) || !is_page_number(total) {
        return line;
    }

    content.trim_end()
}

/// The counter in a page footer, as in `14 of 19`.
fn is_page_number(part: &str) -> bool {
    !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())
}

/// A superscript's letters, which the extraction puts on their own line.
fn is_ordinal_suffix(line: &str) -> bool {
    const SUFFIXES: [&str; 4] = ["st", "nd", "rd", "th"];

    SUFFIXES.iter().any(|suffix| {
        line.strip_prefix(suffix)
            .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric()))
    })
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;

    /// An excerpt of the committed M-1778 text, byte for byte as the extraction
    /// produces it: two-space and three-space question numbers, an asterisk at
    /// the end of a starred question, page footers alone on their line and
    /// merged into the line before, wrapped lines, a superscript split onto its
    /// own line, and the tribe-list prose note.
    const M1778_EXCERPT: &str = "\
2 of 19 uscis.gov/citizenship

AMERICAN GOVERNMENT

A: Principles of American Government

1.   What is the form of government of the United States?

• Republic

• Constitution-based federal republic

2.   What is the supreme law of the land? *
• (U.S.) Constitution

 3 of 19 uscis.gov/citizenship

B: System of Government

16.   Name the three branches of government.

• Legislative, executive, and judicial

• Congress, president, and the courts

97.  What amendment says all persons born or naturalized in the United States,  
        and subject to the jurisdiction thereof, are U.S. citizens?

• 14th Amendment

107.   Dwight Eisenhower is famous for many things. Name one.

• 34
th president of the United States

• Signed the Federal-Aid Highway Act of 1956 (Created the Interstate System) 

117.   Name one American Indian tribe in the United States.

• Tuscarora

For a complete list of tribes, please visit bia.gov.

118.   Name one example of an American innovation.

• Light bulb 18 of 19 uscis.gov/citizenship
";

    #[test]
    fn test_parse_text_questions_and_answers() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let numbers: Vec<u16> = questions.iter().map(|q| q.number).collect();

        assert_eq!(numbers, [1, 2, 16, 97, 107, 117, 118]);

        let first = questions.first().ok_or("question 1 should be extracted")?;

        assert_eq!(first.section, Section::AmericanGovernment);
        assert_eq!(first.topic, Topic::PrinciplesOfAmericanGovernment);
        assert_eq!(
            first.question,
            "What is the form of government of the United States?"
        );
        assert_eq!(
            first.accepted_answers,
            [
                AcceptedAnswer::from("Republic"),
                AcceptedAnswer::from("Constitution-based federal republic"),
            ]
        );
        assert_eq!(first.answers_required, 1);
        assert!(!first.starred);
        assert_eq!(first.explanation, None);

        Ok(())
    }

    #[test]
    fn test_parse_text_starred_question() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let starred: Vec<u16> = questions
            .iter()
            .filter(|q| q.starred)
            .map(|q| q.number)
            .collect();

        assert_eq!(starred, [2]);

        let second = questions.get(1).ok_or("question 2 should be extracted")?;

        assert_eq!(second.question, "What is the supreme law of the land?");
        assert_eq!(
            second.accepted_answers,
            [AcceptedAnswer::from("(U.S.) Constitution")]
        );

        Ok(())
    }

    #[test]
    fn test_parse_text_wrapped_question_joins() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let wrapped = questions
            .iter()
            .find(|q| q.number == 97)
            .ok_or("question 97 should be extracted")?;

        assert_eq!(
            wrapped.question,
            "What amendment says all persons born or naturalized in the United States, and subject to the jurisdiction thereof, are U.S. citizens?"
        );

        Ok(())
    }

    #[test]
    fn test_parse_text_superscript_suffix_joins_tight() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let question = questions
            .iter()
            .find(|q| q.number == 107)
            .ok_or("question 107 should be extracted")?;

        assert_eq!(
            question.accepted_answers,
            [
                AcceptedAnswer::from("34th president of the United States"),
                AcceptedAnswer::from(
                    "Signed the Federal-Aid Highway Act of 1956 (Created the Interstate System)"
                ),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_text_prose_paragraph_is_not_an_answer() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let question = questions
            .iter()
            .find(|q| q.number == 117)
            .ok_or("question 117 should be extracted")?;

        assert_eq!(
            question.accepted_answers,
            [AcceptedAnswer::from("Tuscarora")]
        );

        Ok(())
    }

    #[test]
    fn test_parse_answers_required_counts() {
        // -- Setup & Fixtures
        let cases = [
            ("Name one thing the U.S. Constitution does.", 1),
            (
                "Name two important ideas from the Declaration of Independence and the U.S. Constitution.",
                2,
            ),
            ("There are three branches of government. Why?", 1),
            ("Name the three branches of government.", 3),
            ("What are the two parts of the U.S. Congress?", 2),
            ("What are two Cabinet-level positions?", 2),
            (
                "What are three rights of everyone living in the United States?",
                3,
            ),
            (
                "There are four amendments to the U.S. Constitution about who can vote. Describe one of them.",
                1,
            ),
            (
                "It is important for all men age 18 through 25 to register for the Selective Service. Name one reason why.",
                1,
            ),
            ("What is one way Americans can serve their country?", 1),
            (
                "During the Cold War, what was one main concern of the United States?",
                1,
            ),
            ("There were 13 original states. Name five.", 5),
            ("How many amendments does the U.S. Constitution have?", 1),
            ("Why does each state have two senators?", 1),
        ];

        // -- Exec & Check
        for (question, expected) in cases {
            assert_eq!(answers_required(question), expected, "question: {question}");
        }
    }

    #[test]
    fn test_parse_text_merged_page_footer_is_not_an_answer() -> Result<()> {
        // -- Exec
        let questions = parse_text(M1778_EXCERPT)?;

        // -- Check
        let question = questions
            .iter()
            .find(|q| q.number == 118)
            .ok_or("question 118 should be extracted")?;

        assert_eq!(
            question.accepted_answers,
            [AcceptedAnswer::from("Light bulb")]
        );

        Ok(())
    }

    #[test]
    fn test_parse_text_unknown_topic_heading_errors() {
        // -- Setup & Fixtures
        let text =
            "AMERICAN GOVERNMENT\n\nA: Principles of Something Else\n\n1.   What?\n\n• Yes\n";

        // -- Exec
        let result = parse_text(text);

        // -- Check
        assert!(
            result.is_err(),
            "a Topic the 2025 test does not publish must not pass"
        );
    }

    #[test]
    fn test_parse_text_topic_outside_its_section_errors() {
        // -- Setup & Fixtures
        let text =
            "AMERICAN GOVERNMENT\n\nA: Colonial Period and Independence\n\n1.   What?\n\n• Yes\n";

        // -- Exec
        let result = parse_text(text);

        // -- Check
        assert!(
            result.is_err(),
            "a Topic filed under the wrong Section must not pass"
        );
    }

    #[test]
    fn test_parse_text_unknown_section_heading_errors() {
        // -- Setup & Fixtures
        let text = "AMERICAN COLONIES\n\nA: Elsewhere\n\n1.   What?\n\n• Yes\n";

        // -- Exec
        let result = parse_text(text);

        // -- Check
        assert!(result.is_err(), "an unknown Section heading must not pass");
    }

    #[test]
    fn test_parse_text_question_without_answers_errors() {
        // -- Setup & Fixtures
        let text = "AMERICAN GOVERNMENT\n\nA: Principles of American Government\n\n1.   What is the form of government of the United States?\n";

        // -- Exec
        let result = parse_text(text);

        // -- Check
        assert!(
            result.is_err(),
            "a question without Accepted Answers must not pass"
        );
    }
}

// endregion: --- Tests
