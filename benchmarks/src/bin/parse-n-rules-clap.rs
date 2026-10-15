//! The rules CLI in clap 4. `argv[0]` is kept because clap expects it.

use std::hint::black_box;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    bench::run(|| bench::rules::clap4::parse(black_box(&args)).is_some_and(|cli| cli.level > 0));
}
