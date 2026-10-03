//! mise's shadow in clap: `../usage/benches/gate/src/bin/parse-n-clap.rs`.

use std::hint::black_box;

use clap::Parser as _;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    bench::run(|| {
        shadow_mise_clap::Cli::try_parse_from(black_box(&args))
            .is_ok_and(|cli| cli.command.is_some())
    });
}
