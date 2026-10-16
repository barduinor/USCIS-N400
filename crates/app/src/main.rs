// region:    --- Modules

mod components;
mod error;
mod questions;
mod routes;
mod screens;
mod storage;

pub use error::{Error, Result};
pub use routes::App;

// endregion: --- Modules

fn main() -> Result<()> {
    dioxus::launch(App);

    Ok(())
}
