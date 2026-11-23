//! Where `surface` and `available_if` end up.
//!
//! Neither changes help or parsing: the derive writes them into a fragment
//! and `winnow-args-spec` carries them into the command tree. Write the
//! fragments to a directory Cargo does not watch, then print them:
//!
//! ```text
//! WINNOW_ARGS_SPEC=$PWD/target/metadata-spec cargo check --example metadata
//! cargo run --example metadata -- target/metadata-spec
//! cargo run --example metadata -- target/metadata-spec --verbose check
//! ```

use std::path::Path;

use winnow_args::Args;
use winnow_args::Subcommand;
use winnow_args_spec::Catalog;
use winnow_args_spec::Item;

/// The program.
#[derive(Args, Debug)]
#[arg(
    name = "metadata",
    surface = "advanced",
    available_if("debug", "tests")
)]
struct Cli {
    /// A flag with metadata of its own.
    #[arg(long, surface = "internal", available_if = "nightly")]
    verbose: bool,
    #[arg(subcommand)]
    command: Command,
}

/// Its subcommands.
#[derive(Subcommand, Debug)]
enum Command {
    /// A unit, with its own metadata.
    #[arg(surface = "tests", available_if = "unix")]
    Check,
    /// A type with its own metadata: the variant replaces its surface and
    /// adds to its conditions.
    #[arg(surface = "operators", available_if("remote"))]
    Run(Run),
    /// Named fields: their options are this command's.
    #[arg(surface = "builders", available_if = "nightly")]
    Build {
        /// How many jobs.
        #[arg(long)]
        jobs: usize,
    },
}

/// The type a variant holds.
#[derive(Args, Debug)]
#[arg(surface = "default", available_if = "root")]
struct Run {
    /// When to run.
    #[arg(long)]
    when: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: metadata FRAGMENTS_DIR [COMMAND_LINE...]");
        std::process::exit(2);
    };
    let rest: Vec<_> = args.collect();
    let catalog = Catalog::load(Path::new(&dir))?;
    let command = catalog.stitch(&catalog.root()?)?;
    show(
        "command",
        &command.name,
        command.surface.as_deref(),
        &command.available_if,
    );
    for item in &command.items {
        if item.surface.is_none() && item.available_if.is_empty() {
            continue;
        }
        show(
            "flag",
            &label(item),
            item.surface.as_deref(),
            &item.available_if,
        );
    }
    for sub in &command.subcommands {
        show(
            "subcommand",
            &sub.name,
            sub.command.surface.as_deref(),
            &sub.command.available_if,
        );
    }
    if !rest.is_empty() {
        match Cli::parse_from(rest) {
            Ok(cli) => match cli.command {
                Command::Build { jobs } => println!("parsed: build with {jobs} jobs"),
                other => println!("parsed: {other:?}"),
            },
            Err(error) => println!("parse error: {:?}", error.kind()),
        }
    }
    Ok(())
}

/// The spelling an item is listed under.
fn label(item: &Item) -> String {
    item.long
        .clone()
        .or_else(|| item.short.map(|short| format!("-{short}")))
        .unwrap_or_else(|| item.value_name.clone().unwrap_or_default())
}

fn show(kind: &str, name: &str, surface: Option<&str>, available_if: &[String]) {
    println!("{kind} {name}");
    println!("  surface: {}", surface.unwrap_or("-"));
    println!("  available_if: {}", available_if.join(", "));
}
