//! The rules CLI: every framework accepts and refuses the same lines, and
//! accepts each benchmarked line to the same fields, or the numbers compare
//! different work.

use std::ffi::{OsStr, OsString};

use winnow_args::Args as _;

/// `(format, schema, upload, endpoint, level, dir, name, target, from, to)`.
type Fields = (
    Option<String>,
    Option<String>,
    bool,
    Option<String>,
    u32,
    Option<String>,
    Option<String>,
    Option<String>,
    u32,
    u32,
);

fn wa(line: &[&str]) -> Option<Fields> {
    let args: Vec<OsString> = line.iter().map(OsString::from).collect();
    let c = bench::rules::wa::Cli::parse_words(&winnow_args::words(&args)).ok()?;
    let format = c.format.map(|f| format!("{f:?}"));
    Some((
        format, c.schema, c.upload, c.endpoint, c.level, c.dir, c.name, c.target, c.from, c.to,
    ))
}

fn clap(line: &[&str]) -> Option<Fields> {
    let args: Vec<OsString> = std::iter::once("export")
        .chain(line.iter().copied())
        .map(OsString::from)
        .collect();
    let c = bench::rules::clap4::parse(&args)?;
    let format = c.format.map(|f| format!("{f:?}"));
    Some((
        format, c.schema, c.upload, c.endpoint, c.level, c.dir, c.name, c.target, c.from, c.to,
    ))
}

fn usage(line: &[&str]) -> Option<Fields> {
    let refs: Vec<&OsStr> = std::iter::once("export")
        .chain(line.iter().copied())
        .map(OsStr::new)
        .collect();
    let c = bench::rules::usage::Cli::try_parse_into_from(&refs).ok()?.0;
    let format = c.format.map(|f| format!("{f:?}"));
    Some((
        format, c.schema, c.upload, c.endpoint, c.level, c.dir, c.name, c.target, c.from, c.to,
    ))
}

#[test]
fn every_benchmarked_line_is_accepted_alike() {
    for line in include_str!("../rules-argv.txt").lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let ours = wa(&words);
        assert!(ours.is_some(), "winnow-args refused {line:?}");
        assert_eq!(clap(&words), ours, "clap on {line:?}");
        assert_eq!(usage(&words), ours, "usage on {line:?}");
    }
}

#[test]
fn every_framework_refuses_what_the_rules_refuse() {
    for line in [
        "--format json --target t",
        "--upload --target t",
        "--dir d",
        "--from 5 --to 1 --target t",
    ] {
        let words: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(wa(&words), None, "winnow-args accepted {line:?}");
        assert_eq!(clap(&words), None, "clap accepted {line:?}");
        assert_eq!(usage(&words), None, "usage accepted {line:?}");
    }
}
