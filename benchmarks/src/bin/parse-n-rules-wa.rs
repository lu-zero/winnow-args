//! The rules CLI in winnow-args.

use std::hint::black_box;

use winnow_args::Args as _;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    bench::run(|| {
        bench::rules::wa::Cli::parse_words(black_box(&words)).is_ok_and(|cli| cli.level > 0)
    });
}
