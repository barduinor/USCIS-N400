//! What the app has already shown the Applicant, in the database's own record.
//!
//! ADR 0007 gives the meta store the database's bookkeeping. It is per device
//! and per browsing context, which is the scope a one-time notice needs: an app
//! installed to the home screen keeps its own storage, separate from the tab it
//! was installed from.

use super::{META_STORE, now, read, write};
use crate::Result;

/// The key the install hint is noted under.
const INSTALL_HINT_KEY: &str = "install_hint";

/// Whether the install hint has been shown here.
pub async fn seen() -> Result<bool> {
    Ok(read(META_STORE, INSTALL_HINT_KEY).await?.is_some())
}

/// Notes that the install hint has been shown, so it is not shown again.
pub async fn mark_seen() -> Result<()> {
    write(META_STORE, INSTALL_HINT_KEY, &now().to_string()).await
}
