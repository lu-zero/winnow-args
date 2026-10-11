//! Fragments the derive writes when `WINNOW_ARGS_SPEC` is set at compile time.
//!
//! Unset, this test does nothing: the derive writes no files. `just check`
//! sets the variable for this test alone, and the assertions below read the
//! fragments back.

#![cfg(feature = "derive")]
// The structs exist so the derive writes them. The test reads the files.
#![allow(dead_code)]

use std::net::IpAddr;
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
    /// Not the enum of that name below: the `use` above says which it is.
    #[arg(long)]
    bind: Option<IpAddr>,
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

mod places {
    /// A type of the standard library has this name too.
    #[derive(winnow_args::ValueEnum)]
    pub enum IpAddr {
        Home,
        Away,
    }

    #[derive(winnow_args::Args)]
    pub struct SpecPlace {
        #[arg(long)]
        place: IpAddr,
    }
}

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
    assert_eq!(
        catalog.roots(),
        [
            "SpecBoth",
            "SpecChange",
            "SpecCli",
            "SpecDupA",
            "SpecDupB",
            "SpecPages",
            "SpecPlace"
        ]
    );
    // `self::SpecInner` and `super::Opts` of an inline module, below a `use<>`.
    let both = catalog.stitch("SpecBoth").unwrap();
    let flags: Vec<_> = both.items.iter().map(|item| item.long.as_deref()).collect();
    assert_eq!(flags, [Some("both"), Some("force")]);
    let one = catalog.stitch("remove::Opts").unwrap();
    assert_eq!(one.items[0].long.as_deref(), Some("recursive"));
    let error = catalog.stitch("Opts").unwrap_err().to_string();
    assert!(
        error.contains("add::Opts") && error.contains("remove::Opts"),
        "{error}"
    );
}

/// What a page says beyond its lists.
#[derive(Args)]
#[arg(
    name = "spec-pages",
    default_subcommand = "m",
    effect = "write",
    example("spec-pages -p world", header = "Try it", help = "Merges nothing."),
    example = "spec-pages query",
    heading("Roots", help = "Said by the command."),
    output("pretty", default, help = "For people"),
    exit_code(0, "done")
)]
struct SpecPages {
    #[arg(flatten)]
    roots: SpecRoots,
    #[arg(subcommand)]
    command: Option<SpecPagesCommand>,
}

#[derive(Args)]
#[arg(
    next_help_heading = "Roots",
    heading("Roots", help = "Said by the group."),
    heading("Other", help = "Only the group says this."),
    output("json", framing = "json"),
    exit_code(64, "no such root")
)]
struct SpecRoots {
    /// The tree.
    #[arg(long)]
    root: Option<String>,
    /// Elsewhere.
    #[arg(
        long,
        help_heading = "Other",
        effect = "destructive",
        warning = "Not for use."
    )]
    wipe: bool,
}

#[derive(Subcommand)]
enum SpecPagesCommand {
    /// Merge.
    #[arg(alias_hidden = "m")]
    Merge,
    /// Query.
    #[arg(example = "spec-pages query zlib", effect = "read")]
    Query {
        /// What to ask about.
        #[arg(positional)]
        atom: String,
        #[arg(flatten)]
        roots: SpecRoots,
    },
}

#[test]
fn what_a_page_says_beyond_its_lists_is_in_the_fragments() {
    use winnow_args_spec::{Effect, ExitCode};
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    let catalog = Catalog::load(Path::new(dir)).unwrap_or_else(|error| panic!("{error}"));
    let command = catalog
        .stitch("SpecPages")
        .unwrap_or_else(|error| panic!("{error}"));
    // Named by a spelling help does not show, and found all the same.
    assert_eq!(command.default_subcommand.as_deref(), Some("m"));
    assert!(command.is_default(&command.subcommands[0]));
    assert!(!command.is_default(&command.subcommands[1]));
    assert_eq!(command.effect, Some(Effect::Write));
    let examples: Vec<_> = command.examples.iter().map(|e| e.code.as_str()).collect();
    assert_eq!(examples, ["spec-pages -p world", "spec-pages query"]);
    assert_eq!(command.examples[0].header, "Try it");
    // A flattened struct's fields are under its heading, unless they name one.
    assert_eq!(command.items[0].heading.as_deref(), Some("Roots"));
    assert_eq!(command.items[1].heading.as_deref(), Some("Other"));
    assert_eq!(command.items[1].effect, Some(Effect::Destructive));
    // Its outputs and exit codes are the command's too.
    let outputs: Vec<_> = command.outputs.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(outputs, ["pretty", "json"]);
    assert!(command.outputs[0].default);
    assert_eq!(command.outputs[1].framing, "json");
    let codes: Vec<u8> = command.exit_codes.iter().map(|e| e.code).collect();
    assert_eq!(codes, [0, 64]);
    assert!(matches!(&command.exit_codes[1], ExitCode { code: 64, .. }));
    // A variant with fields is a command of its own.
    let query = &command.subcommands[1].command;
    assert_eq!(query.name, "query");
    assert_eq!(query.effect, Some(Effect::Read));
    assert_eq!(query.examples[0].code, "spec-pages query zlib");
    let labels: Vec<_> = query
        .items
        .iter()
        .map(|item| item.value_name.as_deref().or(item.long.as_deref()).unwrap())
        .collect();
    assert_eq!(labels, ["ATOM", "ROOT", "wipe"]);
    #[cfg(feature = "help-text")]
    {
        assert_eq!(command.examples[0].help, "Merges nothing.");
        // What the command says of a heading is read first.
        assert_eq!(command.heading_help("Roots"), Some("Said by the command."));
        assert_eq!(
            command.heading_help("Other"),
            Some("Only the group says this.")
        );
        assert_eq!(command.items[1].warning, "Not for use.");
        assert_eq!(command.exit_codes[1].help, "no such root");
    }
}

/// Two enums of one name in a file, each with a variant that has fields:
/// the name each states is their variants' too.
mod dup_a {
    #[derive(winnow_args::Subcommand)]
    #[arg(spec = "SpecDupA")]
    pub enum SpecDup {
        Build {
            #[arg(long)]
            alpha: bool,
        },
    }
}

mod dup_b {
    #[derive(winnow_args::Subcommand)]
    #[arg(spec = "SpecDupB")]
    pub enum SpecDup {
        Build {
            #[arg(long)]
            beta: bool,
        },
        #[arg(spec = "SpecDupTest")]
        Test {
            #[arg(long)]
            gamma: bool,
        },
    }
}

#[test]
fn variants_with_fields_of_enums_of_one_name_are_told_apart() {
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    let catalog = Catalog::load(Path::new(dir)).unwrap_or_else(|error| panic!("{error}"));
    let flags = |root: &str| -> Vec<String> {
        let command = catalog
            .stitch(root)
            .unwrap_or_else(|error| panic!("{error}"));
        let subs = command.subcommands.iter();
        subs.map(|sub| sub.command.items[0].long.clone().unwrap())
            .collect()
    };
    assert_eq!(flags("SpecDupA"), ["alpha"]);
    assert_eq!(flags("SpecDupB"), ["beta", "gamma"]);
}

#[test]
fn emitted_fragments_stitch() {
    let Some(dir) = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty()) else {
        return;
    };
    let catalog = Catalog::load(Path::new(dir)).unwrap_or_else(|error| panic!("{error}"));
    let command = catalog
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
    assert!(command.subcommands[0].command.items[1].choices.is_empty());
    assert_eq!(
        command.subcommands[0].command.items[2].choices,
        <winnow_args::complete::Shell as winnow_args::FromArg>::CHOICES
    );
    let place = catalog.stitch("SpecPlace").unwrap();
    assert_eq!(place.items[0].choices, ["home", "away"]);
    assert_eq!(command.subcommands[1].name, "true");
    assert_eq!(command.name, "spec-tool");
    #[cfg(feature = "help-text")]
    {
        assert_eq!(command.items[0].help, "say \"hi\"");
        assert_eq!(command.about, "Root.");
    }
}
