//! bpaf 0.10. `cli_p()` builds the combinator tree, which is per-process work,
//! so it is inside the loop — as in usage's `parse-n-bpaf`.

use std::hint::black_box;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    bench::run(|| {
        bench::bpaf010::cli_p()
            .run_inner(black_box(&refs[..]))
            .is_ok_and(|cli| cli.verbose && cli.path.is_some())
    });
}
