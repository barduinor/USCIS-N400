//! One module per screen.

pub mod exam;
pub mod exam_result;
pub mod main_menu;
pub mod onboarding;
pub mod profile;
pub mod stats;
pub mod study;

pub use exam::Exam;
pub use exam_result::ExamResult;
pub use main_menu::MainMenu;
pub use onboarding::Onboarding;
pub use profile::Profile;
pub use stats::Stats;
pub use study::Study;
