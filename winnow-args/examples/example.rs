//! `example -v/--verbose -p/--path=PATH`, written twice: once with the
//! combinators and once with the derive. Both print what they parsed.
//!
//!     cargo run --example example -- -v --path=/tmp
//!     cargo run --example example -- -vp /tmp
//!     cargo run --example example --features terminal-size -- -h

use std::path::PathBuf;

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow_args::combinator::{Named, args, short};
use winnow_args::{Args, Argv, Error, report, words};

#[derive(Debug, PartialEq)]
struct Example {
    verbose: bool,
    path: Option<PathBuf>,
}

const VERBOSE: Named = short('v').long("verbose");
const PATH: Named = short('p').long("path");

/// The combinator version: an `alt` of occurrence parsers, each folded into a local.
fn example(input: &mut Argv<'_>) -> Result<Example, Error> {
    let mut verbose = false;
    let mut path = None;
    args(alt((
        VERBOSE.switch().map(|()| verbose = true),
        PATH.argument_as::<PathBuf>().map(|p| path = Some(p)),
    )))
    .parse_next(input)?;
    Ok(Example { verbose, path })
}

/// The derive version of the same CLI.
#[derive(Args, Debug, PartialEq)]
struct Derived {
    /// Say what is going on while it goes on, one line for each step taken
    #[arg(short, long)]
    verbose: bool,
    /// Where to work: a directory that exists and that this program may write to
    #[arg(short, long)]
    path: Option<PathBuf>,
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = words(&args);
    match example.parse_next(&mut Argv::new(&words)) {
        Ok(parsed) => println!("combinator: {parsed:?}"),
        Err(e) => eprintln!("combinator: error: {e}"),
    }
    match Derived::parse_from(&words) {
        Ok(parsed) => println!("derive:     {parsed:?}"),
        // Help (`-h`) included, wrapped to `COLUMNS` or, with the
        // `terminal-size` feature, to the terminal.
        Err(e) => std::process::exit(report(&e, "example")),
    }
}
