//! mise's shadow in bpaf; `cli_p()` is built inside the loop, as usage's gate does.

use std::hint::black_box;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    bench::run(|| {
        shadow_mise_bpaf::cli_p()
            .run_inner(black_box(&refs[..]))
            .is_ok_and(|cli| cli.command.is_some())
    });
}
