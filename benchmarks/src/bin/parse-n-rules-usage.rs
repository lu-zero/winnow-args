//! The rules CLI in usage, through its `try_into` entry point, which takes the
//! program name first.

use std::ffi::{OsStr, OsString};
use std::hint::black_box;

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    bench::run(|| {
        bench::rules::usage::Cli::try_parse_into_from(black_box(&refs)).is_ok_and(|c| c.0.level > 0)
    });
}
