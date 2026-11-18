//! A failure shown with the command line and a caret under the byte it is
//! about, which `report_argv` prints in place of `Args::parse`'s plain report.
//!
//!     cargo run --example caret -- --jobs 4 src
//!     cargo run --example caret -- --jbs 4 src          # an unknown flag
//!     cargo run --example caret -- --jobs four src      # a value that is not a number
//!     cargo run --example caret -- -v --colour=red src  # a choice that does not exist

use winnow_args::{Args, ValueEnum, report_argv, words};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Colour {
    Auto,
    Always,
    Never,
}

/// Build a project.
#[derive(Args, Debug)]
#[arg(name = "caret")]
struct Build {
    /// Jobs at once.
    #[arg(short, long, default = "1")]
    jobs: u32,
    /// Say more.
    #[arg(short, long)]
    verbose: bool,
    /// When to use colour.
    #[arg(long, default = "auto")]
    colour: Colour,
    /// The source directory.
    #[arg(positional)]
    source: String,
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = words(&args);
    match Build::parse_words(&words) {
        Ok(build) => println!("{build:?}"),
        Err(error) => std::process::exit(report_argv(&error, "caret", &words)),
    }
}
