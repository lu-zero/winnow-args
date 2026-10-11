//! Generators for the sources this repository derives from other trees.
//!
//! ```text
//! cargo run -p xtask -- gen mold MOLD_DIR
//! cargo run -p xtask -- gen examples BRUSH_DIR MOLD_DIR
//! cargo run -p xtask -- gen mise-shadow
//! ```
//!
//! They work on the text, not on a syntax tree: what they move between trees
//! (mold's option bodies, brush's builtins, their comments) is kept verbatim.

use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use winnow_args::{Args, Subcommand};

mod examples;
mod mise;
mod mold;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Development tasks of winnow-args.
#[derive(Args)]
#[arg(name = "xtask")]
struct Cli {
    #[arg(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Regenerate a generated source.
    Gen(Generate),
}

#[derive(Args)]
struct Generate {
    #[arg(subcommand)]
    what: Generated,
}

#[derive(Subcommand)]
enum Generated {
    /// mold's `--features winnow-args` parser, from its `origin/main` one.
    Mold(Mold),
    /// `examples/{brush_builtins,ld}.rs`, from the two ports.
    Examples(Examples),
    /// `benchmarks/shadows/mise-wa`, from usage's mise shadow.
    MiseShadow,
}

#[derive(Args)]
struct Mold {
    /// A mold checkout; its `src/cmdline.rs` is rewritten.
    #[arg(positional, value_name = "MOLD_DIR")]
    mold: PathBuf,
}

#[derive(Args)]
struct Examples {
    /// brush's `winnow-port` checkout.
    #[arg(positional, value_name = "BRUSH_DIR")]
    brush: PathBuf,
    /// mold's `winnow-args-cmdline` checkout.
    #[arg(positional, value_name = "MOLD_DIR")]
    mold: PathBuf,
}

fn main() {
    let Task::Gen(Generate { what }) = Cli::parse().task;
    let done = match what {
        Generated::Mold(Mold { mold }) => mold::generate(&mold),
        Generated::Examples(Examples { brush, mold }) => examples::generate(&brush, &mold),
        Generated::MiseShadow => mise::generate(),
    };
    if let Err(error) = done {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

/// The workspace's root.
fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace")
}

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a valid pattern")
}

/// Where `needle` starts in `text`; `what` names it in the error.
fn index_of(text: &str, needle: &str, what: &str) -> Result<usize> {
    text.find(needle)
        .ok_or_else(|| format!("no {what} (`{needle}`)").into())
}

fn rustfmt(files: &[&Path]) -> Result<()> {
    let status = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .args(files)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("rustfmt: {status}").into())
    }
}
