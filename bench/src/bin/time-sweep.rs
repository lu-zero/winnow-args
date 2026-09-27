//! Warm wall clock per parse, in process: the minimum and median over many
//! short rounds, the estimator usage's `time-sweep` uses because noise from a
//! loaded machine only ever adds time.
//!
//! Prints `name min_ns median_ns` per framework, for `tasks/perf.sh` to tabulate.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;
use std::time::Instant;

use clap::Parser as _;
use winnow::Parser as _;
use winnow_args::{Args as _, Argv};

const ROUNDS: usize = 2_000;

fn sweep(name: &str, iters: usize, mut f: impl FnMut()) {
    for _ in 0..iters.max(200) {
        f();
    }
    let mut per_call: Vec<f64> = (0..ROUNDS)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..iters {
                f();
            }
            start.elapsed().as_secs_f64() * 1e9 / iters as f64
        })
        .collect();
    per_call.sort_by(f64::total_cmp);
    println!("{name} {:.0} {:.0}", per_call[0], per_call[ROUNDS / 2]);
}

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

    sweep("usage", 2_000, || {
        black_box(bench::usage::Cli::parse_from(black_box(&refs))).ok();
    });
    sweep("wa", 2_000, || {
        black_box(bench::wa_derive::Cli::parse_from(black_box(&words))).ok();
    });
    sweep("wa-comb", 2_000, || {
        black_box(bench::wa_comb::cli.parse_next(&mut Argv::new(black_box(&words)))).ok();
    });
    sweep("bpaf", 100, || {
        black_box(bench::bpaf010::cli_p().run_inner(black_box(&strs[..]))).ok();
    });
    sweep("clap", 100, || {
        black_box(bench::clap4::Cli::try_parse_from(black_box(&clap_argv))).ok();
    });
}
