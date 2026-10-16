use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{Result, generate};

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

/// Runs the parsed command line.
pub fn run() -> Result<()> {
    let cli = CliCmd::parse();

    match cli.command {
        CliSubCmd::Generate(args) => generate_records(args),
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
