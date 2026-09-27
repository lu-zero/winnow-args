//! clap 4. `try_parse_from` builds the command tree and parses; both are
//! per-process work. `argv[0]` is kept because clap expects it.

use std::hint::black_box;

use clap::Parser as _;

fn main() {
    let n = bench::parse_n();
    let args: Vec<_> = std::env::args_os().collect();
    let mut seen = 0usize;
    for _ in 0..n {
        if let Ok(cli) = bench::clap4::Cli::try_parse_from(black_box(&args)) {
            seen += usize::from(cli.verbose && cli.path.is_some());
        }
    }
    println!("{seen}");
}
