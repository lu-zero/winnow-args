//! `time-sweep` for the rules CLI: warm min and median per parse.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

use winnow_args::Args as _;

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    // usage's `try_into` entry point and clap both take the program name first.
    let argv: Vec<OsString> = std::iter::once(OsString::from("export"))
        .chain(args.iter().cloned())
        .collect();
    let refs: Vec<&OsStr> = argv.iter().map(|a| a.as_os_str()).collect();

    bench::sweep("usage", 2_000, || {
        black_box(bench::rules::usage::Cli::try_parse_into_from(black_box(
            &refs,
        )))
        .ok();
    });
    bench::sweep("wa", 2_000, || {
        black_box(bench::rules::wa::Cli::parse_words(black_box(&words))).ok();
    });
    bench::sweep("clap", 100, || {
        black_box(bench::rules::clap4::parse(black_box(&argv)));
    });
}
