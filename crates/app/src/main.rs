// region:    --- Modules

mod error;
mod questions;
mod routes;
mod screens;

pub use error::{Error, Result};
pub use routes::App;

// endregion: --- Modules

fn main() -> Result<()> {
    dioxus::launch(App);

    Ok(())
}
