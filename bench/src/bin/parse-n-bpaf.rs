//! bpaf 0.10. `cli_p()` builds the combinator tree, which is per-process work,
//! so it is inside the loop — as in usage's `parse-n-bpaf`.

use std::hint::black_box;

fn main() {
    let n = bench::parse_n();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut seen = 0usize;
    for _ in 0..n {
        let parsed = bench::bpaf010::cli_p().run_inner(black_box(&refs[..]));
        if let Ok(cli) = parsed {
            seen += usize::from(cli.verbose && cli.path.is_some());
        }
    }
    println!("{seen}");
}
