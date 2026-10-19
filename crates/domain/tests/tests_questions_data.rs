//! The records contract: the committed records parse into the domain types.
//!
//! This sits outside the rule seams on purpose. It guards the artifact the
//! application embeds — 128 Civics Test Questions, 20 of them starred — so a
//! bad regeneration fails here rather than in the browser.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use domain::{
    AcceptedAnswer, AnswerLogEntry, Coverage, Exam, Profile, Question, QuestionRecords, Section,
    TestUpdates, Track, Verdict, answers, matching, selection,
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
fn test_questions_data_alternatives_attach_to_published_answers() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let mut seen_questions = BTreeSet::new();

    // -- Exec & Check
    for entry in &records.alternatives {
        assert!(
            seen_questions.insert(entry.number),
            "question {} is reviewed twice",
            entry.number
        );
        assert!(
            !entry.forms.is_empty(),
            "question {} lists no reviewed forms",
            entry.number
        );

        let question = records
            .question(entry.number)
            .ok_or_else(|| format!("alternatives name unknown question {}", entry.number))?;

        assert!(
            !answers::is_fact(entry.number),
            "question {} takes its answer from the Profile; alternatives do not attach to it",
            entry.number
        );

        let mut seen_forms = BTreeSet::new();
        for form in &entry.forms {
            assert!(
                !form.form.as_str().trim().is_empty(),
                "question {} has an empty reviewed form",
                entry.number
            );
            assert!(
                question
                    .accepted_answers
                    .iter()
                    .any(|answer| answer == &form.answer),
                "question {} does not publish answer {:?}",
                entry.number,
                form.answer
            );
            assert!(
                seen_forms.insert(form.form.as_str()),
                "question {} repeats reviewed form {:?}",
                entry.number,
                form.form
            );
        }
    }

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
fn test_questions_data_65_20_serves_only_the_starred_questions() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let reduced = Profile {
        special_consideration_65_20: true,
        ..Profile::default()
    };
    let accountable = records.accountable(&reduced);
    let mut log = Vec::new();
    let mut asked = Vec::new();

    // -- Exec
    for _ in 0..60 {
        let question = selection::next(&accountable, &log, None).expect("a starred candidate");

        asked.push(question.number);
        log.push(AnswerLogEntry {
            at: 1_770_000_000_000,
            question: question.number,
            answer: "Republic".to_string(),
            verdict: Verdict::Correct,
            overridden: false,
        });
    }

    // -- Check
    let starred: BTreeSet<u16> = records
        .questions
        .iter()
        .filter(|question| question.starred)
        .map(|question| question.number)
        .collect();

    assert_eq!(
        asked.iter().copied().collect::<BTreeSet<u16>>(),
        starred,
        "sixty questions of study serve the starred questions and nothing else"
    );

    Ok(())
}

#[test]
fn test_questions_data_a_wrong_answer_gets_its_gap_on_the_real_scopes() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let profiles = [
        Profile::default(),
        Profile {
            special_consideration_65_20: true,
            ..Profile::default()
        },
    ];

    // -- Exec & Check
    for profile in profiles {
        let accountable = records.accountable(&profile);
        let mut log = Vec::new();
        let mut asked = Vec::new();

        // Every answer wrong: the worst case for the gap the rules ask for.
        for _ in 0..400 {
            let question = selection::next(&accountable, &log, None).expect("a candidate");

            asked.push(question.number);
            log.push(AnswerLogEntry {
                at: 1_770_000_000_000,
                question: question.number,
                answer: "Republic".to_string(),
                verdict: Verdict::Wrong,
                overridden: false,
            });
        }

        let gap = smallest_gap(&asked).expect("a question comes back");

        assert!(
            gap >= 8,
            "a wrong answer came back after {gap} other questions with the 65/20 flag {}",
            profile.special_consideration_65_20
        );
    }

    Ok(())
}

/// The smallest distance between two appearances of one question.
fn smallest_gap(asked: &[u16]) -> Option<usize> {
    let mut gap = None;

    for (position, question) in asked.iter().enumerate() {
        if let Some(earlier) = asked[..position]
            .iter()
            .rposition(|earlier| earlier == question)
        {
            let distance = position - earlier;

            gap = Some(gap.map_or(distance, |gap: usize| gap.min(distance)));
        }
    }

    gap
}

#[test]
fn test_questions_data_exam_draws_its_own_tracks_scope() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let standard = Profile::default();
    let reduced = Profile {
        special_consideration_65_20: true,
        ..Profile::default()
    };
    let starred: BTreeSet<u16> = records
        .questions
        .iter()
        .filter(|question| question.starred)
        .map(|question| question.number)
        .collect();

    // Eight wrong then twelve correct asks the whole twenty-question draw; four
    // wrong then six correct asks the whole ten.
    let patterns = [
        (Track::Standard, standard, 8, 12),
        (Track::SpecialConsideration, reduced, 4, 6),
    ];

    // -- Exec & Check
    for (track, profile, wrongs, corrects) in patterns {
        let accountable = records.accountable(&profile);
        let mut exam = Exam::start(&accountable, track, 20_261_016);
        let mut verdicts = vec![Verdict::Wrong; wrongs];
        verdicts.extend(vec![Verdict::Correct; corrects]);

        for verdict in verdicts {
            exam.answer(verdict);
        }

        let record = exam.record(1_770_000_000_000).expect("a decided Exam");

        assert_eq!(
            record.questions.len(),
            track.questions(),
            "the whole draw is asked before the outcome is decided"
        );
        assert!(
            record.questions.iter().all(|number| records
                .accountable(&profile)
                .iter()
                .any(|question| question.number == *number)),
            "an Exam asks only the Applicant's accountable questions"
        );

        if track == Track::SpecialConsideration {
            assert!(
                record
                    .questions
                    .iter()
                    .all(|number| starred.contains(number)),
                "the 65/20 Exam draws from the starred questions and nothing else"
            );
        }
    }

    Ok(())
}

#[test]
fn test_questions_data_section_and_topic_figures_partition_coverage() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let profiles = [
        Profile::default(),
        Profile {
            special_consideration_65_20: true,
            ..Profile::default()
        },
    ];

    // One Cleared question in each Section, each in its own Topic, and all three
    // inside the 65/20 scope so both tracks report the same three.
    let mut log = Vec::new();
    for number in [2, 74, 126] {
        for _ in 0..3 {
            log.push(AnswerLogEntry {
                at: 1_770_000_000_000,
                question: number,
                answer: "Republic".to_string(),
                verdict: Verdict::Correct,
                overridden: false,
            });
        }
    }

    // -- Exec & Check
    for profile in profiles {
        let global = Coverage::from_log(&records, &profile, &log);
        let sections: Vec<Coverage> = Section::ALL
            .iter()
            .map(|section| Coverage::of_section(&records, &profile, &log, *section))
            .collect();
        let topics: Vec<Coverage> = Section::ALL
            .iter()
            .flat_map(|section| section.topics().iter().copied())
            .map(|topic| Coverage::of_topic(&records, &profile, &log, topic))
            .collect();

        assert_eq!(global.cleared(), 3, "three questions were Cleared");
        assert_eq!(
            sections
                .iter()
                .map(|coverage| coverage.cleared())
                .sum::<u16>(),
            global.cleared(),
            "the Sections' cleared counts sum to the global one"
        );
        assert_eq!(
            sections
                .iter()
                .map(|coverage| coverage.total())
                .sum::<u16>(),
            global.total(),
            "so do their totals"
        );
        assert_eq!(
            topics
                .iter()
                .map(|coverage| coverage.cleared())
                .sum::<u16>(),
            global.cleared(),
            "the Topics' cleared counts sum to the global one"
        );
        assert_eq!(
            topics.iter().map(|coverage| coverage.total()).sum::<u16>(),
            global.total(),
            "so do their totals"
        );
        assert_eq!(
            sections
                .iter()
                .filter(|coverage| coverage.cleared() == 1)
                .count(),
            3,
            "one Cleared question in each Section"
        );
    }

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
fn test_questions_data_judges_the_numeral_forms_of_the_digit_answers() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();

    // The ten Accepted Answers whose parenthetical numeral buys a digit form
    // (ADR 0004), by question number. Q7 is the amendments question; `27` is
    // the answer the Study screen has to get right.
    let numerals = [
        (7, "27"),
        (21, "100"),
        (22, "6 years"),
        (24, "435"),
        (25, "2 years"),
        (27, "2"),
        (36, "4 years"),
        (53, "9"),
        (54, "5"),
        (63, "citizens 18 and older"),
    ];

    // -- Exec & Check
    for (number, digit_form) in numerals {
        let question = records
            .question(number)
            .expect("a question the numeral answers name");
        let accepted = answers::accepted_answers(question, &filled);

        assert_eq!(
            matching::judge(question, &accepted, digit_form),
            Verdict::Correct,
            "question {number} must accept its digit form `{digit_form}`"
        );
    }

    Ok(())
}

#[test]
fn test_questions_data_keeps_a_fragments_words_wrong_on_the_real_records() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();

    // A fragment's words alone are never an answer, whichever side of the
    // answer the parentheses sit on (ADR 0004): the year Q104 prints, the
    // abbreviation Q2 prints, and one fragment of each other shape. The answer
    // is still accepted with the fragment dropped.
    let fragments = [
        (2, "us", "constitution"),
        (4, "example of", "social contract"),
        (28, "for small states", "equal representation"),
        (33, "congressional", "citizens in their district"),
        (40, "of the united states", "the vice president"),
        (63, "women and men can vote", "any citizen can vote"),
        (104, "1929", "the great crash"),
    ];

    // -- Exec & Check
    for (number, fragment, dropped) in fragments {
        let question = records
            .question(number)
            .expect("a question the fragment answers name");
        let accepted = answers::accepted_answers(question, &filled);

        assert_eq!(
            matching::judge(question, &accepted, fragment),
            Verdict::Wrong,
            "question {number} must not accept the fragment `{fragment}` alone"
        );
        assert_eq!(
            matching::judge(question, &accepted, dropped),
            Verdict::Correct,
            "question {number} must accept `{dropped}`, the fragment dropped"
        );
    }

    Ok(())
}

#[test]
fn test_questions_data_accepts_the_compromise_questions_other_name() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();
    let question = records.question(28).expect("the compromise question");
    let accepted = answers::accepted_answers(question, &filled);

    // -- Exec & Check
    assert!(
        question
            .accepted_answers
            .iter()
            .any(|answer| answer.as_str() == "Connecticut Compromise"),
        "Q28's other name is an Accepted Answer of its own, from the overrides"
    );
    assert_eq!(
        matching::judge(question, &accepted, "Connecticut Compromise"),
        Verdict::Correct,
        "Q28 must accept the compromise's other name"
    );
    assert_eq!(
        matching::judge(question, &accepted, "The Great Compromise"),
        Verdict::Correct,
        "Q28 keeps accepting the answer with the fragment dropped"
    );

    Ok(())
}

#[test]
fn test_questions_data_accepts_a_joining_of_the_statue_of_liberty_answers() -> Result<()> {
    // -- Setup & Fixtures
    let records = committed_records()?;
    let filled = filled_profile();
    let question = records
        .question(120)
        .expect("the Statue of Liberty question");
    let accepted = answers::accepted_answers(question, &filled);

    // Q120 takes one answer and lists five, so the Applicant may say two of
    // them in one line (ADR 0004) — the line an officer would hear as one
    // correct answer. The words that belong to no answer stay wrong.
    let joins = [
        "liberty island new york",
        "new jersey liberty island",
        "on the hudson river",
    ];
    let beside = ["not liberty island", "statue island", "the statue"];

    // -- Exec & Check
    for typed in joins {
        assert_eq!(
            matching::judge(question, &accepted, typed),
            Verdict::Correct,
            "Q120 must accept `{typed}`, a joining of its answers"
        );
    }
    for typed in beside {
        assert_eq!(
            matching::judge(question, &accepted, typed),
            Verdict::Wrong,
            "Q120 must not accept `{typed}`, whose words are no form of its answers"
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
