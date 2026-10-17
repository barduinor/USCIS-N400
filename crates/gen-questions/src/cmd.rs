use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{Result, generate, test_updates};

/// The `gen-questions` command line.
#[derive(Debug, Parser)]
#[command(
    version,
    about,
    arg_required_else_help = true,
    after_help = "Run from the repository root: the default paths are relative to it."
)]
pub struct CliCmd {
    #[command(subcommand)]
    pub command: CliSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum CliSubCmd {
    /// Regenerates the committed Civics Test records from the USCIS question list.
    Generate(GenerateArgs),

    /// Fetches the four Test Update values and writes the committed record.
    GenerateTestUpdates(GenerateTestUpdatesArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// The USCIS question list to read.
    #[arg(long, default_value = generate::QUESTION_LIST_PATH)]
    pub pdf: PathBuf,

    /// The committed corrections to apply.
    #[arg(long, default_value = generate::OVERRIDES_PATH)]
    pub overrides: PathBuf,

    /// The records file to write.
    #[arg(long, default_value = generate::RECORDS_PATH)]
    pub records: PathBuf,
}

#[derive(Debug, Args)]
pub struct GenerateTestUpdatesArgs {
    /// A saved page to read instead of fetching it, for an offline run.
    #[arg(long)]
    pub html: Option<PathBuf>,

    /// The USCIS test-updates page to fetch.
    #[arg(long, default_value = test_updates::SOURCE_URL)]
    pub url: String,

    /// The Test Update record to write.
    #[arg(long, default_value = test_updates::TEST_UPDATES_PATH)]
    pub output: PathBuf,
}

/// Runs the parsed command line.
pub fn run() -> Result<()> {
    let cli = CliCmd::parse();

    match cli.command {
        CliSubCmd::Generate(args) => generate_records(args),
        CliSubCmd::GenerateTestUpdates(args) => generate_test_updates(args),
    }
}

/// Writes the committed records from the question list and the corrections.
fn generate_records(args: GenerateArgs) -> Result<()> {
    let pdf = std::fs::read(&args.pdf)?;
    let overrides = std::fs::read_to_string(&args.overrides)?;
    let records = generate::records_from_pdf(&pdf, &overrides)?;
    let json = generate::records_json(&records)?;

    std::fs::write(&args.records, json)?;

    println!(
        "wrote {} Civics Test Questions to {}",
        records.questions.len(),
        args.records.display()
    );

    Ok(())
}

/// Writes the committed Test Update record from the USCIS page.
fn generate_test_updates(args: GenerateTestUpdatesArgs) -> Result<()> {
    let html = match &args.html {
        Some(path) => std::fs::read_to_string(path)?,
        None => test_updates::fetch(&args.url)?,
    };
    let record = test_updates::record_from_html(&html, &args.url, &test_updates::today())?;
    let json = test_updates::record_json(&record)?;

    std::fs::write(&args.output, json)?;

    println!(
        "wrote the four Test Updates fetched {} from {} to {}",
        record.fetched,
        record.source,
        args.output.display()
    );

    Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn test_cmd_cli_cmd_verify() {
        // -- Exec & Check
        CliCmd::command().debug_assert();
    }
}

// endregion: --- Tests
