//! A relation that names a flag of a struct it flattens, two levels down.
//!
//!     cargo run --example flatten_rules -- --quiet --verbose  # conflict
//!     cargo run --example flatten_rules -- --check            # the flattened default suffices
//!     cargo run --example flatten_rules -- --quiet             # both flags are fine alone

use winnow_args::{Args, report};

/// Flags held by a struct nested two levels down.
#[derive(Args, Debug)]
struct Inner {
    /// More output.
    #[arg(long)]
    verbose: bool,
    /// A mode with a default.
    #[arg(long, default = "plain")]
    mode: String,
}

/// A struct that flattens `Inner`.
#[derive(Args, Debug)]
struct Middle {
    #[arg(flatten)]
    inner: Inner,
}

#[derive(Args, Debug)]
#[arg(name = "flatten-rules")]
struct Cli {
    /// Less output: conflicts with `Inner`'s `--verbose`.
    #[arg(long, conflicts("--verbose"))]
    quiet: bool,
    /// A check that needs the flattened `--mode`.
    #[arg(long, requires("--mode"))]
    check: bool,
    #[arg(flatten)]
    middle: Middle,
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match Cli::parse_from(&args) {
        Ok(cli) => println!("{cli:#?}"),
        Err(error) => std::process::exit(report(&error, "flatten-rules")),
    }
}
