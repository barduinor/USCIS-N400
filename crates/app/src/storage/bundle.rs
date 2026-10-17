//! The export bundle: the Applicant's whole saved state in one JSON text.
//!
//! ADR 0007: one bundle carrying the schema version, when it was exported, the
//! Profile, the Answer Log, and the Exam records. Import validates the shape,
//! refuses a schema newer than this build reads, and replaces local state after
//! confirmation; it never merges. The bundle is the Applicant's only backup and
//! their only way to move between devices.

use domain::{AnswerLogEntry, ExamRecord, Profile};
use rexie::{Transaction, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use super::{
    ANSWER_LOG_STORE, EXAMS_STORE, META_STORE, PROFILE_STORE, database, now, write,
    write_store_count,
};
use crate::storage::{answer_log, exam, profile};
use crate::{Error, Result};

/// The export schema this build reads and writes (ADR 0007: it starts at 1).
pub const SCHEMA_VERSION: u32 = 1;

/// The meta key the last export is noted under.
const LAST_EXPORT_KEY: &str = "last_export";

/// What the Applicant reads when the file they chose is not a bundle: a parse
/// failure is the parser's, and it never reaches them.
const NOT_A_BUNDLE: &str =
    "this file is not a bundle the app can read; import a bundle the app exported.";

/// The whole of the Applicant's state, as one bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bundle {
    pub schema_version: u32,

    /// When the export was written, in milliseconds since the Unix epoch.
    pub exported_at: u64,

    pub profile: Profile,
    pub answer_log: Vec<AnswerLogEntry>,
    pub exams: Vec<ExamRecord>,
}

impl Bundle {
    /// Gathers the Applicant's state as it stands, ready to export.
    pub async fn gather() -> Result<Self> {
        let Some(profile) = profile::load().await? else {
            return Err(Error::custom("there is no Profile to export"));
        };

        Ok(Self {
            schema_version: SCHEMA_VERSION,
            exported_at: now(),
            profile,
            answer_log: answer_log::entries_verified().await?,
            exams: exam::exams_verified().await?,
        })
    }

    /// The text an export writes: readable, so a backup can be inspected.
    pub fn to_text(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Notes the export in the database's own record (ADR 0007: the meta record
    /// holds when the Applicant last exported).
    pub async fn note_export(&self) -> Result<()> {
        write(META_STORE, LAST_EXPORT_KEY, &self.exported_at.to_string()).await
    }

    /// The bundle a text holds, or why it cannot be read.
    pub fn from_text(text: &str) -> Result<Self> {
        // The schema version is read before the rest, so a bundle from a newer
        // build says so even where this build cannot parse its records.
        let envelope: serde_json::Value =
            serde_json::from_str(text).map_err(|_| Error::custom(NOT_A_BUNDLE))?;
        let version = envelope
            .get("schema_version")
            .and_then(serde_json::Value::as_u64);

        if let Some(version) = version
            && version > u64::from(SCHEMA_VERSION)
        {
            return Err(Error::custom(
                format!(
                    "this bundle was written by a newer version of the app: it is schema {version}, \
                     and this one reads schema {SCHEMA_VERSION}. Update the app, or export from \
                     the version that wrote it."
                )
                .as_str(),
            ));
        }

        serde_json::from_str(text).map_err(|_| Error::custom(NOT_A_BUNDLE))
    }

    /// Hands the bundle to the device as a file: the Applicant's backup, and the
    /// only copy of their progress that leaves this browser's storage. A step
    /// the browser refuses becomes [`Error::Download`], so the Applicant reads
    /// the app's own sentence whatever the browser's words were.
    pub fn download(&self) -> Result<()> {
        let text = self.to_text()?;
        let name = format!("uscis-n400-progress-{}.json", date(self.exported_at));

        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(&text));
        let blob = web_sys::Blob::new_with_str_sequence(&parts)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| Error::custom("this page has no document"))?;
        // The cast hands the element back when it is not an anchor, so it is
        // that element the browser refused with.
        let anchor = document
            .create_element("a")?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|element| Error::Download(element.into()))?;

        anchor.set_href(&url);
        anchor.set_download(&name);
        anchor.click();
        web_sys::Url::revoke_object_url(&url)?;

        Ok(())
    }

    /// Replaces the local state with this bundle's, after the Applicant has
    /// confirmed it. Nothing is merged: what is here goes.
    pub async fn replace(&self) -> Result<()> {
        let database = database().await?;
        let transaction = database.transaction(
            &[PROFILE_STORE, ANSWER_LOG_STORE, EXAMS_STORE, META_STORE],
            TransactionMode::ReadWrite,
        )?;

        transaction
            .store(PROFILE_STORE)?
            .put(
                &JsValue::from_str(&profile::to_record(&self.profile)?),
                Some(&JsValue::from_str(profile::PROFILE_KEY)),
            )
            .await?;
        write_all(&transaction, ANSWER_LOG_STORE, &self.answer_log).await?;
        write_all(&transaction, EXAMS_STORE, &self.exams).await?;
        write_store_count(&transaction, ANSWER_LOG_STORE, answer_log::COUNT_KEY).await?;
        write_store_count(&transaction, EXAMS_STORE, exam::COUNT_KEY).await?;
        transaction.commit().await?;

        Ok(())
    }
}

/// The day a timestamp falls on, as `2026-10-16`, for a file name.
fn date(at: u64) -> String {
    js_sys::Date::new(&JsValue::from_f64(at as f64))
        .to_iso_string()
        .as_string()
        .map(|text| text.chars().take(10).collect())
        .unwrap_or_else(|| "export".to_string())
}

/// Empties one store in this transaction and writes these records into it, in
/// order.
async fn write_all<T: Serialize>(
    transaction: &Transaction,
    store_name: &str,
    records: &[T],
) -> Result<()> {
    let store = transaction.store(store_name)?;
    store.clear().await?;

    for record in records {
        store
            .add(&JsValue::from_str(&serde_json::to_string(record)?), None)
            .await?;
    }

    Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>; // For tests.

    use super::*;
    use domain::{Outcome, TestUpdates, Track, Verdict};

    /// One whole bundle, as gather would hand it over.
    fn bundle() -> Bundle {
        Bundle {
            schema_version: SCHEMA_VERSION,
            exported_at: 1_770_000_000_000,
            profile: Profile {
                state: "New Jersey".to_string(),
                governor: Some("Mikie Sherrill".to_string()),
                test_updates: TestUpdates::default(),
                special_consideration_65_20: true,
                ..Profile::default()
            },
            answer_log: vec![AnswerLogEntry {
                at: 1_770_000_000_000,
                question: 2,
                answer: "U.S. Constitution".to_string(),
                verdict: Verdict::Correct,
                overridden: false,
            }],
            exams: vec![ExamRecord {
                at: 1_770_000_000_000,
                track: Track::SpecialConsideration,
                questions: vec![2, 7, 12],
                correct_count: 2,
                outcome: Outcome::Passed,
            }],
        }
    }

    #[test]
    fn test_bundle_round_trips_the_whole_state() -> Result<()> {
        // -- Setup & Fixtures
        let bundle = bundle();

        // -- Exec
        let text = bundle.to_text()?;
        let read_back = Bundle::from_text(&text)?;

        // -- Check
        assert_eq!(read_back, bundle);
        assert!(
            text.contains("\"schema_version\": 1"),
            "the exported text carries its schema version"
        );

        Ok(())
    }

    #[test]
    fn test_bundle_refuses_a_newer_schema_with_a_clear_message() -> Result<()> {
        // -- Setup & Fixtures
        let text = bundle()
            .to_text()?
            .replace("\"schema_version\": 1", "\"schema_version\": 2");

        // -- Exec
        let error = Bundle::from_text(&text).expect_err("a newer schema is refused");

        // -- Check
        let message = error.to_string();
        assert!(
            message.contains("newer version")
                && message.contains("schema 2")
                && message.contains("schema 1"),
            "the message says what happened: {message}"
        );

        Ok(())
    }

    #[test]
    fn test_bundle_refuses_a_text_that_is_not_a_bundle() -> Result<()> {
        // -- Exec & Check
        assert!(Bundle::from_text("not json at all").is_err());
        assert!(
            Bundle::from_text("{\"schema_version\": 1}").is_err(),
            "a version alone is not a bundle"
        );

        Ok(())
    }

    /// A step of the download the browser refuses: the Profile screen reads the
    /// app's own sentence, never the browser's `JsValue(...)`, and the sentence
    /// says the export can be tried again and what the file is worth.
    #[test]
    fn test_failed_export_reads_as_the_apps_own_sentence() {
        // -- Setup & Fixtures
        // Every failing `web_sys` call hands back a `JsValue`; the host can
        // hold only its constants, and one stands in for the browser's refusal.
        let refusal = JsValue::NULL;

        // -- Exec
        let error = Error::from(refusal);

        // -- Check
        assert_eq!(
            error.to_string(),
            "the export did not go through; try it again, and keep the file it produces: it is \
             the only copy of your progress outside this device."
        );
    }
}

// endregion: --- Tests
