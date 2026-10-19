//! The Study scope: every accountable question, or one Topic.

use domain::{Question, Topic};

/// Which Civics Test Questions a Study session may ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyScope {
    All,
    Topic(Topic),
}

impl StudyScope {
    /// Whether `question` belongs to this scope.
    pub fn includes(self, question: &Question) -> bool {
        match self {
            Self::All => true,
            Self::Topic(topic) => question.topic == topic,
        }
    }
}

/// The stable URL segment for a Topic.
pub fn slug(topic: Topic) -> &'static str {
    match topic {
        Topic::PrinciplesOfAmericanGovernment => "principles-of-american-government",
        Topic::SystemOfGovernment => "system-of-government",
        Topic::RightsAndResponsibilities => "rights-and-responsibilities",
        Topic::ColonialPeriodAndIndependence => "colonial-period-and-independence",
        Topic::EighteenHundreds => "1800s",
        Topic::RecentAmericanHistoryAndOtherImportantHistoricalInformation => {
            "recent-american-history-and-other-important-historical-information"
        }
        Topic::Symbols => "symbols",
        Topic::Holidays => "holidays",
    }
}

/// The Topic a URL segment names, if it is one of the eight.
pub fn from_slug(slug: &str) -> Option<Topic> {
    match slug {
        "principles-of-american-government" => Some(Topic::PrinciplesOfAmericanGovernment),
        "system-of-government" => Some(Topic::SystemOfGovernment),
        "rights-and-responsibilities" => Some(Topic::RightsAndResponsibilities),
        "colonial-period-and-independence" => Some(Topic::ColonialPeriodAndIndependence),
        "1800s" => Some(Topic::EighteenHundreds),
        "recent-american-history-and-other-important-historical-information" => {
            Some(Topic::RecentAmericanHistoryAndOtherImportantHistoricalInformation)
        }
        "symbols" => Some(Topic::Symbols),
        "holidays" => Some(Topic::Holidays),
        _ => None,
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use domain::Section;

    use super::*;

    #[test]
    fn test_study_scope_slugs_round_trip_every_topic() {
        // -- Setup & Fixtures
        let topics = Section::ALL
            .iter()
            .flat_map(|section| section.topics())
            .copied()
            .collect::<Vec<_>>();

        // -- Exec & Check
        assert_eq!(topics.len(), 8);
        for topic in topics {
            let slug = slug(topic);
            assert_eq!(
                from_slug(slug),
                Some(topic),
                "the slug {slug:?} must round-trip {topic:?}"
            );
        }
    }
}

// endregion: --- Tests
