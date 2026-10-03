//! The mise shadows must agree before their timings mean anything: the benchmark
//! line binds the same fields in all four, and usage's and winnow-args' accept
//! and reject the same lines.

use std::ffi::{OsStr, OsString};

use clap::Parser as _;
use winnow_args::Args as _;

fn os(line: &str) -> Vec<OsString> {
    line.split_whitespace().map(OsString::from).collect()
}

#[test]
fn the_benchmark_line_binds_the_same_fields() {
    let args = os("use -g node@20");
    let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
    let strs: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
    let expect = (true, vec!["node@20".to_string()]);

    let Some(shadow_mise::Commands::Use(u)) =
        shadow_mise::Cli::parse_from(&refs).ok().unwrap().command
    else {
        panic!("usage did not reach `use`")
    };
    assert_eq!((u.global, u.tool_version), expect);

    let words = winnow_args::words(&args);
    let Some(shadow_mise_wa::Commands::Use(u)) =
        shadow_mise_wa::Cli::parse_from(&words).unwrap().command
    else {
        panic!("winnow-args did not reach `use`")
    };
    assert_eq!((u.global, u.tool_version), expect);

    let argv = std::iter::once(OsString::from("mise")).chain(args.iter().cloned());
    let Some(shadow_mise_clap::Commands::Use(u)) =
        shadow_mise_clap::Cli::try_parse_from(argv).unwrap().command
    else {
        panic!("clap did not reach `use`")
    };
    assert_eq!((u.global, u.tool_version), expect);

    let Some(shadow_mise_bpaf::Commands::Use(u)) = shadow_mise_bpaf::cli_p()
        .run_inner(&strs[..])
        .unwrap()
        .command
    else {
        panic!("bpaf did not reach `use`")
    };
    assert_eq!((u.global, u.tool_version), expect);
}

/// Lines across the CLI: routing, globals on either side, aliases, `--`,
/// `:::`, `help`, values, unknown flags, and some that must fail.
const LINES: &[&str] = &[
    "use -g node@20",
    "u -g node@20",
    "-C /tmp use node@20 python@3.12",
    "use node@20 -C /tmp",
    "install node@20",
    "ls --json",
    "list",
    "x node@20 -- node app.js",
    "exec --jobs 4 node@20 -- npm test",
    "run build",
    "run build ::: test",
    "tasks ls",
    "settings set color false",
    "tool-alias get node lts",
    "sync node --brew",
    "use",
    "use --jobs",
    "settings set",
    "tool-alias nope",
    // Unknown flags: usage's default makes them positional values.
    "use --wat node@20",
    "use -gx node@20",
    "run build --wat",
    "x node@20 -z",
    "ls --wat",
    "settings set color --wat",
];

#[test]
fn usage_and_winnow_args_accept_the_same_lines() {
    for line in LINES {
        let args = os(line);
        let refs: Vec<&OsStr> = args.iter().map(|a| a.as_os_str()).collect();
        let usage = shadow_mise::Cli::parse_from(&refs).is_ok();
        let wa = winnow_args::with_env(&[], || {
            shadow_mise_wa::Cli::parse_from(&winnow_args::words(&args))
        });
        assert_eq!(
            wa.is_ok(),
            usage,
            "{line:?}: usage {usage}, winnow-args {:?}",
            wa.err()
        );
    }
}
