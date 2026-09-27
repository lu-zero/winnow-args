//! winnow-args derive. Borrowing argv as words happens once, outside the loop,
//! as usage's harness borrows its `&[&OsStr]`.

use std::hint::black_box;

use winnow_args::Args as _;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let words = winnow_args::words(&args);
    bench::run(|| {
        bench::wa_derive::Cli::parse_from(black_box(&words))
            .is_ok_and(|cli| cli.verbose && cli.path.is_some())
    });
}
