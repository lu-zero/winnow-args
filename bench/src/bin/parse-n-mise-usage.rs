//! mise's shadow in usage: `../usage/benches/gate/src/bin/parse-n.rs`.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    bench::run(|| {
        shadow_mise::Cli::parse_from(black_box(&refs)).is_ok_and(|cli| cli.command.is_some())
    });
}
