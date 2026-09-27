//! winnow-args combinators; see `parse-n-wa`.

use std::hint::black_box;

use winnow::Parser as _;
use winnow_args::ArgvBuf;

fn main() {
    let n = bench::parse_n();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut seen = 0usize;
    for _ in 0..n {
        let buf = ArgvBuf::new(black_box(&args));
        if let Ok(cli) = bench::wa_comb::cli.parse_next(&mut buf.argv()) {
            seen += usize::from(cli.verbose && cli.path.is_some());
        }
    }
    println!("{seen}");
}
