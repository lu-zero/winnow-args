//! winnow-args derive. The flattening into an `ArgvBuf` is inside the loop: it
//! is part of what a parse costs, the way clap and bpaf copy argv too.

use std::hint::black_box;

use bench::wa_derive::Cli;
use winnow_args::{Args as _, ArgvBuf};

fn main() {
    let n = bench::parse_n();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut seen = 0usize;
    for _ in 0..n {
        let buf = ArgvBuf::new(black_box(&args));
        if let Ok(cli) = Cli::parse_argv(&mut buf.argv()) {
            seen += usize::from(cli.verbose && cli.path.is_some());
        }
    }
    println!("{seen}");
}
