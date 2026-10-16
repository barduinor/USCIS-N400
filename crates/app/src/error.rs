use derive_more::{Display, From};

pub type Result<T, E = Error> = core::result::Result<T, E>;

/// What the Applicant reads when a record on the device does not match the
/// shape this build reads: the storage that holds it, and the way back, in
/// place of the parser's complaint about a field they have never heard of.
const UNREADABLE_RECORD: &str = "the storage on this device holds something the app cannot read, \
                                 and importing a bundle from the Profile screen is the way back.";

/// What the Applicant reads when the device's storage itself cannot be used.
/// Their bundle is the only copy of their progress off the device, so it is the
/// way back here too.
const UNUSABLE_STORAGE: &str = "the storage on this device cannot be used, \
                                and importing a bundle from the Profile screen is the way back.";

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum Error {
    /// The application's own message. It is shown to the Applicant, so it reads
    /// as the message rather than as a debugged enum.
    #[display("{_0}")]
    #[from(String, &String, &str)]
    Custom(String),

    // -- Externals
    /// A stored record this build cannot read. `Debug` keeps the parser's
    /// complaint for a terminal; the Applicant reads the sentence above.
    #[display("{}", self::UNREADABLE_RECORD)]
    #[from]
    Json(serde_json::Error),

    /// The device's storage, when a read or a write against it fails.
    #[display("{}", self::UNUSABLE_STORAGE)]
    #[from]
    Storage(rexie::Error),
}

// region:    --- Custom

impl Error {
    pub fn custom(val: impl Into<String>) -> Self {
        Self::Custom(val.into())
    }

    pub fn custom_from_err(err: impl std::error::Error) -> Self {
        Self::Custom(err.to_string())
    }
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
