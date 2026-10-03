//! Warm time per parse of mise's shadow in bpaf 0.10, as `time-sweep-mise` prints it.

use std::hint::black_box;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strs: Vec<&str> = args.iter().map(String::as_str).collect();
    bench::sweep("bpaf010", 20, || {
        black_box(bench_bpaf010::mise::cli_p().run_inner(black_box(&strs[..]))).ok();
    });
}
