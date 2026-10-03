//! Warm wall clock per parse, in process: the minimum and median over many
//! short rounds, the estimator usage's `time-sweep` uses because noise from a
//! loaded machine only ever adds time.
//!
//! Prints `name min_ns median_ns` per framework, for `just perf` to tabulate.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

use clap::Parser as _;
use winnow::Parser as _;
use winnow_args::{Args as _, Argv};

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    let words = winnow_args::words(&args);
    let strs: Vec<&str> = args
        .iter()
        .map(|a| a.to_str().expect("UTF-8 argv"))
        .collect();
    let clap_argv: Vec<OsString> = std::iter::once(OsString::from("example"))
        .chain(args.iter().cloned())
        .collect();

    bench::sweep("usage", 2_000, || {
        black_box(bench::usage::Cli::parse_from(black_box(&refs))).ok();
    });
    bench::sweep("wa", 2_000, || {
        black_box(bench::wa_derive::Cli::parse_words(black_box(&words))).ok();
    });
    bench::sweep("wa-comb", 2_000, || {
        black_box(bench::wa_comb::cli.parse_next(&mut Argv::new(black_box(&words)))).ok();
    });
    bench::sweep("wa-disp", 2_000, || {
        black_box(bench::wa_disp::cli.parse_next(&mut Argv::new(black_box(&words)))).ok();
    });
    bench::sweep("bpaf", 100, || {
        black_box(bench::bpaf09::cli_p().run_inner(black_box(&strs[..]))).ok();
    });
    bench::sweep("clap", 100, || {
        black_box(bench::clap4::Cli::try_parse_from(black_box(&clap_argv))).ok();
    });
}
