//! Wall clock per parse, in process: the minimum over many short rounds, the
//! estimator usage's `time-sweep` uses because noise from a loaded machine only
//! ever adds time.

use std::ffi::OsString;
use std::hint::black_box;
use std::time::Instant;

use clap::Parser as _;
use winnow::Parser as _;
use winnow_args::{Args as _, ArgvBuf};

const ROUNDS: usize = 2_000;

fn sweep(iters: usize, mut f: impl FnMut()) -> (f64, f64) {
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
    (per_call[0], per_call[per_call.len() / 2])
}

fn report(label: &str, (min, median): (f64, f64)) {
    println!("{label:<36}{min:>9.0} {median:>9.0}  ns");
}

fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let words = if words.is_empty() {
        vec!["-v".into(), "--path".into(), "/tmp/x".into()]
    } else {
        words
    };
    let os: Vec<OsString> = words.iter().map(OsString::from).collect();
    let clap_argv: Vec<OsString> = std::iter::once(OsString::from("example"))
        .chain(os.iter().cloned())
        .collect();
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();

    println!("argv: {words:?}");
    println!("{:<36}{:>9} {:>9}", "", "min", "median");
    let usage_argv: Vec<&std::ffi::OsStr> = os.iter().map(|a| a.as_os_str()).collect();
    report(
        "usage derive",
        sweep(2_000, || {
            black_box(bench::usage::Cli::parse_from(black_box(&usage_argv))).ok();
        }),
    );
    report(
        "winnow-args derive",
        sweep(2_000, || {
            let buf = ArgvBuf::new(black_box(&os));
            black_box(bench::wa_derive::Cli::parse_argv(&mut buf.argv())).ok();
        }),
    );
    report(
        "winnow-args combinators",
        sweep(2_000, || {
            let buf = ArgvBuf::new(black_box(&os));
            black_box(bench::wa_comb::cli.parse_next(&mut buf.argv())).ok();
        }),
    );
    report(
        "bpaf 0.10: build + parse",
        sweep(100, || {
            let parsed = bench::bpaf010::cli_p().run_inner(black_box(&refs[..]));
            black_box(parsed).ok();
        }),
    );
    report(
        "clap 4: build + parse",
        sweep(100, || {
            black_box(bench::clap4::Cli::try_parse_from(black_box(&clap_argv))).ok();
        }),
    );
}
