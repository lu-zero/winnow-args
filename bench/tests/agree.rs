//! Every framework must parse every benchmarked line to the same fields, or the
//! numbers compare different work.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use clap::Parser as _;
use winnow::Parser as _;
use winnow_args::{Args as _, Argv};

type Fields = (usize, Option<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);

#[test]
fn frameworks_agree_on_every_benchmarked_line() {
    for line in include_str!("../argv.txt")
        .lines()
        .filter(|l| !l.is_empty())
    {
        let args: Vec<OsString> = line.split(' ').map(OsString::from).collect();
        let strs: Vec<&str> = line.split(' ').collect();
        let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
        let words = winnow_args::words(&args);
        let clap_argv = std::iter::once(OsString::from("example")).chain(args.iter().cloned());

        let usage = bench::usage::Cli::parse_from(&refs)
            .map(|c| (c.verbose.into(), c.path, c.include, c.files))
            .unwrap_or_else(|_| panic!("usage rejected {line:?}"));
        let all: [(&str, Fields); 5] = [
            (
                "wa",
                bench::wa_derive::Cli::parse_from(&words)
                    .map(|c| (c.verbose.into(), c.path, c.include, c.files))
                    .unwrap(),
            ),
            (
                "wa-disp",
                bench::wa_disp::cli
                    .parse_next(&mut Argv::new(&words))
                    .map(|c| (c.verbose.into(), c.path, c.include, c.files))
                    .unwrap(),
            ),
            (
                "wa-comb",
                bench::wa_comb::cli
                    .parse_next(&mut Argv::new(&words))
                    .map(|c| (c.verbose.into(), c.path, c.include, c.files))
                    .unwrap(),
            ),
            (
                "bpaf",
                bench::bpaf010::cli_p()
                    .run_inner(&strs[..])
                    .map(|c| (c.verbose, c.path, c.include, c.files))
                    .unwrap(),
            ),
            (
                "clap",
                bench::clap4::Cli::try_parse_from(clap_argv)
                    .map(|c| (c.verbose.into(), c.path, c.include, c.files))
                    .unwrap(),
            ),
        ];
        for (name, fields) in all {
            assert_eq!(fields, usage, "{name} disagrees with usage on {line:?}");
        }
    }
}
