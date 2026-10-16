//! The records contract: the committed records parse into the domain types.
//!
//! This sits outside the rule seams on purpose. It guards the artifact the
//! application embeds — 128 Civics Test Questions, 20 of them starred — so a
//! bad regeneration fails here rather than in the browser.

use std::path::{Path, PathBuf};

use domain::{
    AcceptedAnswer, Profile, Question, QuestionRecords, Section, TestUpdates, Verdict, answers,
    matching,
};

type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

/// The committed records, relative to the workspace root.
const RECORDS: &str = "crates/app/data/questions.json";

/// The 2025 test's size, and the 65/20 Special Consideration's scope.
const QUESTIONS: usize = 128;
const STARRED: usize = 20;

/// The 2025 test's Topics, across its three Sections.
const TOPICS: usize = 8;

#[test]
fn test_questions_data_parses_the_committed_records() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let parsed = records.questions.len();

    // -- Check
    assert_eq!(
        parsed, QUESTIONS,
        "the committed records must hold the 128 Civics Test Questions"
    );

    Ok(())
}

#[test]
fn test_questions_data_starred_questions_are_the_65_20_scope() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec
    let starred = records.questions.iter().filter(|q| q.starred).count();

    // -- Check
    assert_eq!(
        starred, STARRED,
        "exactly 20 questions carry the asterisk the 65/20 Special Consideration studies"
    );

    Ok(())
}

#[test]
fn test_questions_data_topics_sit_in_their_sections() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec & Check
    let mut sections: Vec<Section> = Vec::new();
    let mut topics = Vec::new();

    for question in &records.questions {
        if !sections.contains(&question.section) {
            sections.push(question.section);
        }
        if !topics.contains(&question.topic) {
            topics.push(question.topic);
        }

        assert_eq!(
            question.topic.section(),
            question.section,
            "question {} is filed under a Topic from another Section",
            question.number
        );
    }

    assert_eq!(
        sections,
        Section::ALL,
        "the records must cover the three Sections, in the test's order"
    );
    assert_eq!(
        topics.len(),
        TOPICS,
        "the records must cover the eight Topics"
    );

    Ok(())
}

#[test]
fn test_questions_data_accountable_scope_matches_the_records() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let reduced = Profile {
        special_consideration_65_20: true,
        ..Profile::default()
    };

    // -- Exec
    let all = records.accountable(&Profile::default());
    let accountable = records.accountable(&reduced);

    // -- Check
    assert_eq!(all.len(), QUESTIONS);
    assert_eq!(accountable.len(), STARRED);
    assert!(
        accountable.iter().all(|question| question.starred),
        "the 65/20 scope is the starred questions and nothing else"
    );

    Ok(())
}

#[test]
fn test_questions_data_fact_questions_are_exactly_those_with_placeholders() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();

    // -- Exec & Check
    for question in &records.questions {
        let with_values = answers::accepted_answers(question, &filled);
        let without_values = answers::accepted_answers(question, &Profile::default());

        assert!(
            !with_values.is_empty(),
            "question {} still accepts nothing with the Profile filled in",
            question.number
        );
        assert_eq!(
            without_values.is_empty(),
            publishes_placeholder(question),
            "question {} is mapped to a Profile value the wrong way round",
            question.number
        );
    }

    Ok(())
}

#[test]
fn test_questions_data_fact_questions_accept_their_own_profile_value() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();

    // -- Exec & Check
    // The value each question asks about, read off the question's own text.
    let answers = [
        (23, "Cory Booker"),
        (29, "Thomas Kean Jr."),
        (30, "Example Speaker"),
        (38, "Example President"),
        (39, "Example Vice President"),
        (57, "Example Chief Justice"),
        (61, "Mikie Sherrill"),
        (62, "Trenton"),
    ];

    for (number, value) in answers {
        let question = records
            .question(number)
            .expect("a question the fact table names");
        let accepted = answers::accepted_answers(question, &filled);

        assert_eq!(
            matching::judge(question, &accepted, value),
            Verdict::Correct,
            "question {number} must accept the Profile value it asks about"
        );
    }

    Ok(())
}

#[test]
fn test_questions_data_judges_the_published_answers_correct() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();

    // -- Exec & Check
    for question in &records.questions {
        let accepted = answers::accepted_answers(question, &filled);
        let demands = question.answers_required;
        let menus = accepted.iter().any(publishes_alternatives);

        // Each published answer stands on its own for a question that asks for
        // one, and is one short of the count for a question that asks for more.
        // An answer that publishes a menu of alternatives is not itself an
        // answer (ADR 0004), so it has no spelling of its own to judge.
        for answer in accepted
            .iter()
            .filter(|answer| !publishes_alternatives(answer))
        {
            assert_eq!(
                matching::judge(question, &accepted, answer.as_str()),
                if demands > 1 {
                    Verdict::Wrong
                } else {
                    Verdict::Correct
                },
                "question {} must judge `{answer}`",
                question.number
            );
        }

        // A multi-item question is answered by its published answers together.
        if demands > 1 && !menus {
            let published = accepted
                .iter()
                .map(AcceptedAnswer::as_str)
                .collect::<Vec<_>>()
                .join(", ");

            assert_eq!(
                matching::judge(question, &accepted, &published),
                Verdict::Correct,
                "question {} must accept the answers it publishes",
                question.number
            );
        }
        assert_eq!(
            matching::judge(question, &accepted, "definitely not an answer"),
            Verdict::Wrong,
            "question {} must reject a phrasing no key lists",
            question.number
        );
    }

    Ok(())
}

#[test]
fn test_questions_data_holds_no_extraction_artifacts() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;

    // -- Exec & Check
    for question in &records.questions {
        for answer in &question.accepted_answers {
            let answer = answer.as_str();

            assert!(
                !answer.contains(['[', ']']),
                "question {} still holds a square-bracket annotation",
                question.number
            );
            assert!(
                !holds_split_ordinal(answer),
                "question {} still holds `19 th` where the answer is `19th`",
                question.number
            );
        }
    }

    Ok(())
}

// region:    --- Support

/// A Profile holding every Local Fact and Test Update the questions ask for.
fn filled_profile() -> Profile {
    Profile {
        state: "New Jersey".to_string(),
        district: Some("7th".to_string()),
        senators: [
            Some("Cory Booker".to_string()),
            Some("Andy Kim".to_string()),
        ],
        representative: Some("Thomas Kean Jr.".to_string()),
        governor: Some("Mikie Sherrill".to_string()),
        state_capital: Some("Trenton".to_string()),
        test_updates: TestUpdates {
            president: Some("Example President".to_string()),
            vice_president: Some("Example Vice President".to_string()),
            speaker_of_the_house: Some("Example Speaker".to_string()),
            chief_justice: Some("Example Chief Justice".to_string()),
        },
        special_consideration_65_20: false,
    }
}

/// Whether USCIS publishes a placeholder here, because the answer depends on
/// the Applicant's own facts: `Answers will vary.` for a Local Fact, or a
/// pointer to the test-updates page for a Test Update.
fn publishes_placeholder(question: &Question) -> bool {
    question.accepted_answers.iter().all(|answer| {
        let answer = answer.as_str();

        answer.starts_with("Answers will vary") || answer.contains("testupdates")
    })
}

/// Whether USCIS publishes a menu of alternatives here rather than one answer:
/// `or` and `/` inside parentheses spell alternatives (ADR 0004).
fn publishes_alternatives(answer: &AcceptedAnswer) -> bool {
    let answer = answer.as_str();

    answer.match_indices('(').any(|(open, _)| {
        let Some(close) = answer[open..].find(')') else {
            return false;
        };
        let parenthetical = &answer[open + 1..open + close];

        parenthetical.contains(" or ")
            || parenthetical.contains('/')
            || parenthetical.starts_with("or ")
    })
}

/// Whether the text holds an ordinal with the space an extraction leaves in it.
fn holds_split_ordinal(text: &str) -> bool {
    let words = text.split(' ').collect::<Vec<_>>();

    words.windows(2).any(|pair| {
        !pair[0].is_empty()
            && pair[0].chars().all(|character| character.is_ascii_digit())
            && matches!(pair[1], "th" | "nd" | "rd" | "st")
    })
}

fn committed_records() -> Result<QuestionRecords> {
    let path = workspace_root().join(RECORDS);
    let json = std::fs::read_to_string(path)?;

    Ok(serde_json::from_str(&json)?)
}

/// The workspace root: this crate's manifest directory, two levels up.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// endregion: --- Support
