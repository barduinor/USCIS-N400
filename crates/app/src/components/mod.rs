//! Pieces shared between screens.

// region:    --- Modules

pub mod answers;
pub mod coverage_figure;
pub mod field;
pub mod first_run;
pub mod install_hint;
pub mod override_control;
pub mod profile_form;
pub mod refresh;
pub mod state;
pub mod unload;

pub use answers::Answers;
pub use coverage_figure::CoverageFigure;
pub use field::Field;
pub use first_run::{FirstRun, ProfileReload};
pub use install_hint::InstallHint;
pub use override_control::OverrideControl;
pub use profile_form::ProfileForm;
pub use refresh::{ProgressEpoch, RefreshOnResume, use_refresh_on_resume};
pub use state::StateField;
pub use unload::use_before_unload;

// endregion: --- Modules
