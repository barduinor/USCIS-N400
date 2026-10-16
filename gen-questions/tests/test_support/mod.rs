//! Support for the integration tests: the committed files, read from the
//! workspace root.
//!
//! Note: must be imported in each test file.

#![allow(unused)] // For test support

use std::path::{Path, PathBuf};

pub type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// The workspace root: this crate's manifest directory, one level up.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Reads one of the committed files, named by a `generate` path constant.
pub fn read_committed(path: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(workspace_root().join(path))?)
}

/// Reads one of the committed files as bytes.
pub fn read_committed_bytes(path: &str) -> TestResult<Vec<u8>> {
    Ok(std::fs::read(workspace_root().join(path))?)
}
