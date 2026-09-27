//! winnow-args combinators; see `parse-n-wa`.

use std::hint::black_box;

use winnow::Parser as _;
use winnow_args::Argv;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    bench::run(|| {
        bench::wa_comb::cli
            .parse_next(&mut Argv::new(black_box(&words)))
            .is_ok_and(|cli| cli.verbose > 0 && cli.path.is_some())
    });
}
