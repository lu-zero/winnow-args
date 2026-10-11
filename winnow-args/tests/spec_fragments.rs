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
    /// The library's own choices, which no derive writes.
    #[arg(long)]
    completions: Option<winnow_args::complete::Shell>,
}

/// When to run.
#[derive(ValueEnum)]
enum SpecWhen {
    Auto,
    Always,
}

// Two files, and a type named `Opts` in each: the module a variant names
// says which.
#[path = "spec/add.rs"]
mod add;
#[path = "spec/remove.rs"]
mod remove;

/// Changes.
#[derive(Subcommand)]
enum SpecChange {
    Add(add::Opts),
    Remove(remove::Opts),
    /// List them.
    #[arg(spec = "SpecListOpts")]
    List(list::Opts),
    #[arg(spec = "SpecListOpts")]
    Ls(Listing),
}

// A third `Opts`, in this file: a name of its own, stated where it is held,
// also through an alias.
mod list {
    use winnow_args::Args;

    #[derive(Args)]
    #[arg(spec = "SpecListOpts")]
    pub struct Opts {
        /// Every one.
        #[arg(long)]
        all: bool,
    }
}

type Listing = list::Opts;

#[test]
fn types_of_one_name_are_told_apart() {
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    let catalog = Catalog::load(Path::new(dir)).unwrap_or_else(|error| panic!("{error}"));
    let command = catalog
        .stitch("SpecChange")
        .unwrap_or_else(|error| panic!("{error}"));
    let flags: Vec<_> = command
        .subcommands
        .iter()
        .map(|sub| sub.command.items[0].long.as_deref().unwrap())
        .collect();
    assert_eq!(flags, ["force", "recursive", "all", "all"]);
    // Neither `Opts` is a command of its own, and each can be asked for.
    assert_eq!(catalog.roots(), ["SpecChange", "SpecCli"]);
    let one = catalog.stitch("remove::Opts").unwrap();
    assert_eq!(one.items[0].long.as_deref(), Some("recursive"));
    let error = catalog.stitch("Opts").unwrap_err().to_string();
    assert!(
        error.contains("add::Opts") && error.contains("remove::Opts"),
        "{error}"
    );
}

#[test]
fn emitted_fragments_stitch() {
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    let command = Catalog::load(Path::new(dir))
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
    assert_eq!(
        command.subcommands[0].command.items[1].choices,
        <winnow_args::complete::Shell as winnow_args::FromArg>::CHOICES
    );
    assert_eq!(command.subcommands[1].name, "true");
    assert_eq!(command.name, "spec-tool");
    #[cfg(feature = "help-text")]
    {
        assert_eq!(command.items[0].help, "say \"hi\"");
        assert_eq!(command.about, "Root.");
    }
}
