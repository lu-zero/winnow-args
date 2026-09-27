//! usage; the same loop as `../usage/benches/gate/src/bin/parse-n.rs`.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    bench::run(|| {
        bench::usage::Cli::parse_from(black_box(&refs))
            .is_ok_and(|cli| cli.verbose > 0 && cli.path.is_some())
    });
}
