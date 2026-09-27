//! clap 4. `try_parse_from` builds the command tree and parses; both are
//! per-process work. `argv[0]` is kept because clap expects it.

use std::hint::black_box;

use clap::Parser as _;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    bench::run(|| {
        bench::clap4::Cli::try_parse_from(black_box(&args))
            .is_ok_and(|cli| cli.verbose && cli.path.is_some())
    });
}
