use derive_more::{Display, From};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum Error {
    /// The application's own message. It is shown to the Applicant, so it reads
    /// as the message rather than as a debugged enum.
    #[display("{_0}")]
    #[from(String, &String, &str)]
    Custom(String),

    // -- Externals
    #[from]
    Json(serde_json::Error),
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
