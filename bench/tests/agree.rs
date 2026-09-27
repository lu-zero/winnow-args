//! Every framework must parse every benchmarked line to the same fields, or the
//! numbers compare different work.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use clap::Parser as _;
use winnow::Parser as _;
use winnow_args::{Args as _, Argv};

#[derive(Debug, PartialEq)]
struct Fields {
    verbose: usize,
    path: Option<PathBuf>,
    include: Vec<PathBuf>,
    files: Vec<PathBuf>,
    cmd: Vec<String>,
    /// `--color`, by its variant name.
    color: Option<String>,
    jobs: Option<u32>,
    /// `--quiet`, `--json`, `--toml`, `--strict`.
    switches: [bool; 4],
    write: Option<String>,
    offset: Option<i32>,
    args: Option<String>,
    inspect: Option<String>,
    cache: bool,
    /// `use`: `--global` and the tools.
    command: Option<(bool, Vec<String>)>,
}

/// `--jobs` as `Option<u32>` whether a framework holds it as `u32` or `Option<u32>`.
trait FlattenJobs {
    fn flatten_jobs(self) -> Option<u32>;
}
impl FlattenJobs for Option<u32> {
    fn flatten_jobs(self) -> Option<u32> {
        self
    }
}
impl FlattenJobs for Option<Option<u32>> {
    fn flatten_jobs(self) -> Option<u32> {
        self.flatten()
    }
}

macro_rules! fields {
    ($cli:expr, $use:path) => {{
        let c = $cli;
        Fields {
            cache: c.cache(),
            verbose: c.verbose as usize,
            path: c.path,
            include: c.include,
            files: c.files,
            cmd: c.cmd,
            color: c.color.map(|w| format!("{w:?}")),
            jobs: Some(c.jobs).flatten_jobs(),
            switches: [c.quiet, c.json, c.toml, c.strict],
            write: c.write,
            offset: c.offset,
            args: c.args,
            inspect: c.inspect,
            command: c.command.map(|$use(u)| (u.global, u.tools)),
        }
    }};
}

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
            .map(|c| fields!(c, bench::usage::Commands::Use))
            .unwrap_or_else(|_| panic!("usage rejected {line:?}"));
        let all = [
            (
                "wa",
                bench::wa_derive::Cli::parse_from(&words)
                    .map(|c| fields!(c, bench::wa_derive::Commands::Use)),
            ),
            (
                "wa-disp",
                bench::wa_disp::cli
                    .parse_next(&mut Argv::new(&words))
                    .map(|c| fields!(c, bench::wa_comb::Commands::Use)),
            ),
            (
                "wa-comb",
                bench::wa_comb::cli
                    .parse_next(&mut Argv::new(&words))
                    .map(|c| fields!(c, bench::wa_comb::Commands::Use)),
            ),
        ];
        for (name, fields) in all {
            assert_eq!(
                fields.as_ref().ok(),
                Some(&usage),
                "{name} disagrees with usage on {line:?}"
            );
        }
        // See `bench::bpaf010::Cli::cmd`: bpaf cannot route words after `--`
        // away from a greedy positional before it.
        // Nor take a flag-like word as a value (`bench::bpaf010::Cli::args`).
        let bpaf_can = !strs.contains(&"--")
            && !strs
                .windows(2)
                .any(|w| w[0] == "--args" && w[1].starts_with('-'));
        if bpaf_can {
            let bpaf = bench::bpaf010::cli_p()
                .run_inner(&strs[..])
                .map(|c| fields!(c, bench::bpaf010::Commands::Use))
                .unwrap_or_else(|e| panic!("bpaf rejected {line:?}: {e:?}"));
            assert_eq!(bpaf, usage, "bpaf disagrees with usage on {line:?}");
        }
        let clap = bench::clap4::Cli::try_parse_from(clap_argv)
            .map(|c| fields!(c, bench::clap4::Commands::Use))
            .unwrap_or_else(|e| panic!("clap rejected {line:?}: {e}"));
        assert_eq!(clap, usage, "clap disagrees with usage on {line:?}");
    }
}
