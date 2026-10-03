//! mise's shadow in winnow-args rendering the help its command line asks for
//! (`--help`, `use -h`, …): what the help steps in docs/PERF.md measure.

use std::hint::black_box;

use winnow_args::Args as _;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    bench::run(|| {
        shadow_mise_wa::Cli::parse_from(black_box(&words))
            .err()
            .and_then(|e| e.render_help("mise"))
            .is_some_and(|text| !black_box(text).is_empty())
    });
}
