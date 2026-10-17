//! Pieces shared between screens.

// region:    --- Modules

pub mod coverage_figure;
pub mod field;
pub mod first_run;
pub mod install_hint;
pub mod profile_form;
pub mod refresh;
pub mod reveal;
pub mod state;

pub use coverage_figure::CoverageFigure;
pub use field::Field;
pub use first_run::{FirstRun, ProfileReload};
pub use install_hint::InstallHint;
pub use profile_form::ProfileForm;
pub use refresh::{ProgressEpoch, RefreshOnResume, use_refresh_on_resume};
pub use reveal::Reveal;
pub use state::StateField;

// endregion: --- Modules
