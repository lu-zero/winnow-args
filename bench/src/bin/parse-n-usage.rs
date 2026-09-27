//! usage; the same loop as `../usage/benches/gate/src/bin/parse-n.rs`.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

use bench::usage::Cli;

fn main() {
    let n = bench::parse_n();
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    let mut seen = 0usize;
    for _ in 0..n {
        if let Ok(cli) = Cli::parse_from(black_box(&refs)) {
            seen += usize::from(cli.verbose && cli.path.is_some());
        }
    }
    println!("{seen}");
}
