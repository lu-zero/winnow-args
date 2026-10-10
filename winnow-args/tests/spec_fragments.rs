//! Fragments the derive writes when `WINNOW_ARGS_SPEC` is set at compile time.
//!
//! Unset, this test does nothing: the derive writes no files. `just check`
//! sets the variable for this test alone, and the assertions below read the
//! fragments back.

#![cfg(feature = "derive")]
// The structs exist so the derive writes them. The test reads the files.
#![allow(dead_code)]

use std::path::Path;

use winnow_args::{Args, Subcommand, ValueEnum};
use winnow_args_spec::Catalog;

/// Root.
#[derive(Args)]
#[arg(name = "spec-tool", long_only)]
struct SpecCli {
    /// say "hi"
    #[arg(positional)]
    names: Vec<String>,
    /// Verbose.
    #[arg(short, long, global)]
    verbose: bool,
    #[arg(flatten)]
    plain: SpecPlain,
    #[arg(subcommand)]
    command: SpecCommand,
}

/// Flags with no rules of their own.
#[derive(Args)]
struct SpecPlain {
    /// Quiet.
    #[arg(short, long)]
    quiet: bool,
}

/// Commands.
#[derive(Subcommand)]
enum SpecCommand {
    /// Run it.
    Run(SpecRun),
    /// Success.
    True,
}

/// One run.
#[derive(Args)]
struct SpecRun {
    /// When.
    #[arg(long)]
    when: SpecWhen,
}

/// When to run.
#[derive(ValueEnum)]
enum SpecWhen {
    Auto,
    Always,
}

#[test]
fn emitted_fragments_stitch() {
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    // The derive files a crate's fragments under its name.
    let crate_dir = Path::new(dir).join(env!("CARGO_CRATE_NAME"));
    let command = Catalog::load(&crate_dir)
        .unwrap_or_else(|error| panic!("{error}"))
        .stitch("SpecCli")
        .unwrap_or_else(|error| panic!("{error}"));
    let labels: Vec<_> = command
        .items
        .iter()
        .map(|item| item.value_name.as_deref().or(item.long.as_deref()).unwrap())
        .collect();
    assert_eq!(labels, ["NAMES", "verbose", "quiet"]);
    assert!(command.items[1].global);
    assert!(command.long_only);
    assert!(command.subcommand_required);
    assert_eq!(command.subcommands[0].name, "run");
    assert_eq!(
        command.subcommands[0].command.items[0].choices,
        ["auto", "always"]
    );
    assert_eq!(command.subcommands[1].name, "true");
    assert_eq!(command.name, "spec-tool");
    #[cfg(feature = "help-text")]
    {
        assert_eq!(command.items[0].help, "say \"hi\"");
        assert_eq!(command.about, "Root.");
    }
}
