use clap::Parser;

use crate::Result;

/// The `gen-questions` command line.
///
/// The tool turns the committed USCIS question list into the committed Civics
/// Test records. Its command surface arrives with that pipeline; the parsed
/// command is the boundary the pipeline hangs off.
#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help = true)]
pub struct CliCmd {}

/// Runs the parsed command line.
pub fn run() -> Result<()> {
    let _cli = CliCmd::parse();

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
