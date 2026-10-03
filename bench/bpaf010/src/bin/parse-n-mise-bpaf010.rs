//! mise's shadow in bpaf 0.10; `cli_p()` is built inside the loop.

use std::hint::black_box;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    bench::run(|| {
        bench_bpaf010::mise::cli_p()
            .run_inner(black_box(&refs[..]))
            .is_ok_and(|cli| cli.command.is_some())
    });
}
