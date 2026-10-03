//! Warm time per parse in bpaf 0.10, as `time-sweep` prints it.

use std::hint::black_box;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strs: Vec<&str> = args.iter().map(String::as_str).collect();
    bench::sweep("bpaf010", 100, || {
        black_box(bench_bpaf010::cli_p().run_inner(black_box(&strs[..]))).ok();
    });
}
