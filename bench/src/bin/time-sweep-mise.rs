//! `time-sweep` for the mise shadows: warm min and median per parse.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

use clap::Parser as _;
use winnow_args::Args as _;

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    let words = winnow_args::words(&args);
    let strs: Vec<&str> = args
        .iter()
        .map(|a| a.to_str().expect("UTF-8 argv"))
        .collect();
    let clap_argv: Vec<OsString> = std::iter::once(OsString::from("mise"))
        .chain(args.iter().cloned())
        .collect();

    bench::sweep("usage", 2_000, || {
        black_box(shadow_mise::Cli::parse_from(black_box(&refs))).ok();
    });
    bench::sweep("wa", 2_000, || {
        black_box(shadow_mise_wa::Cli::parse_from(black_box(&words))).ok();
    });
    bench::sweep("clap", 20, || {
        black_box(shadow_mise_clap::Cli::try_parse_from(black_box(&clap_argv))).ok();
    });
    bench::sweep("bpaf", 20, || {
        black_box(shadow_mise_bpaf::cli_p().run_inner(black_box(&strs[..]))).ok();
    });
}
