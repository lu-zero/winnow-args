//! mise's shadow in winnow-args, translated from usage's.

use std::hint::black_box;

use winnow_args::Args as _;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    bench::run(|| {
        shadow_mise_wa::Cli::parse_from(black_box(&words)).is_ok_and(|cli| cli.command.is_some())
    });
}
