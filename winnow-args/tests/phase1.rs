//! Phase-1 checklist behaviour, run against the combinator parser and the
//! derived one: both must agree on every command line.
#![cfg(feature = "derive")]

use std::path::PathBuf;

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, long, short};
use winnow_args::{Args, Argv, Error, ErrorKind, words};

#[derive(Debug, PartialEq, Default)]
struct Cli {
    verbose: bool,
    quiet: bool,
    path: Option<PathBuf>,
    jobs: u32,
    set: Option<String>,
    accent: bool,
}

const VERBOSE: Named = short('v').long("verbose");
const QUIET: Named = short('q');
const PATH: Named = short('p').long("path");
const JOBS: Named = short('j').long("jobs");
const SET: Named = long("set");
const ACCENT: Named = short('é');

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    let mut jobs = None;
    args(alt((
        VERBOSE.switch().map(|()| cli.verbose = true),
        QUIET.switch().map(|()| cli.quiet = true),
        PATH.argument_as().map(|p| cli.path = Some(p)),
        JOBS.argument_as().map(|j| jobs = Some(j)),
        SET.argument_as().map(|s| cli.set = Some(s)),
        ACCENT.switch().map(|()| cli.accent = true),
    )))
    .parse_next(input)?;
    cli.jobs = jobs.ok_or_else(|| JOBS.missing(input))?;
    Ok(cli)
}

#[derive(Args, Debug, PartialEq)]
struct Derived {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short)]
    quiet: bool,
    #[arg(short, long)]
    path: Option<PathBuf>,
    #[arg(short, long)]
    jobs: u32,
    set: Option<String>,
    #[arg(short = 'é')]
    accent: bool,
}

impl From<Derived> for Cli {
    fn from(d: Derived) -> Self {
        let Derived {
            verbose,
            quiet,
            path,
            jobs,
            set,
            accent,
        } = d;
        Cli {
            verbose,
            quiet,
            path,
            jobs,
            set,
            accent,
        }
    }
}

/// Parse with both, check they agree, return the shared answer.
fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Derived::parse_from(&words).map(Cli::from);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

fn ok(line: &[&str]) -> Cli {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn err(line: &[&str]) -> Error {
    match parse(line) {
        Ok(cli) => panic!("{line:?} parsed as {cli:?}"),
        Err(e) => e,
    }
}

fn path(p: &str) -> Option<PathBuf> {
    Some(PathBuf::from(p))
}

#[test]
fn long_forms() {
    assert_eq!(ok(&["-j1", "--path=/a"]).path, path("/a"));
    assert_eq!(ok(&["-j1", "--path", "/a"]).path, path("/a"));
    assert_eq!(ok(&["-j1", "--path="]).path, path(""));
    assert_eq!(ok(&["-j1", "--set=a=b"]).set.as_deref(), Some("a=b"));
    assert!(ok(&["-j1", "--verbose"]).verbose);
}

#[test]
fn short_forms() {
    assert_eq!(ok(&["-j1", "-p", "/a"]).path, path("/a"));
    assert_eq!(ok(&["-j1", "-p/a"]).path, path("/a"));
    assert_eq!(ok(&["-j1", "-p=/a"]).path, path("/a"));
    assert_eq!(ok(&["-j1", "-p==/a"]).path, path("=/a"));
    assert_eq!(ok(&["-j", "7"]).jobs, 7);
    assert!(ok(&["-j1", "-é"]).accent);
}

#[test]
fn bundles() {
    let cli = ok(&["-vqj3"]);
    assert!(cli.verbose && cli.quiet);
    assert_eq!(cli.jobs, 3);

    let cli = ok(&["-vp", "/a", "-j2"]);
    assert!(cli.verbose);
    assert_eq!(cli.path, path("/a"));

    // The value-taking letter ends the bundle: `q` is part of the value.
    let cli = ok(&["-j1", "-vpq"]);
    assert!(!cli.quiet);
    assert_eq!(cli.path, path("q"));

    assert!(ok(&["-j1", "-vé"]).accent);
}

#[test]
fn order_and_repetition() {
    assert_eq!(
        ok(&["--path=/a", "-v", "-j1"]),
        ok(&["-j1", "-v", "--path=/a"])
    );
    // A repeated switch stays set; a repeated option keeps the last value.
    assert!(ok(&["-vv", "-j1", "--verbose"]).verbose);
    assert_eq!(ok(&["-j1", "-p/a", "--path=/b"]).path, path("/b"));
    assert_eq!(ok(&["-j1", "-j2"]).jobs, 2);
}

#[test]
fn values_that_look_like_flags() {
    // A lone `-` is a value.
    assert_eq!(ok(&["-j1", "--path", "-"]).path, path("-"));
    assert_eq!(ok(&["-j1", "-p", "-"]).path, path("-"));
    // A dash-prefixed value can be attached.
    assert_eq!(ok(&["-j1", "--path=--verbose"]).path, path("--verbose"));
    assert_eq!(ok(&["-j1", "-p-v"]).path, path("-v"));
}

#[test]
fn words_are_never_resplit() {
    assert_eq!(ok(&["-j1", "--path", "/a b"]).path, path("/a b"));
    assert_eq!(ok(&["-j1", "-p", " "]).path, path(" "));
    assert_eq!(ok(&["-j1", "--set=a\0b"]).set.as_deref(), Some("a\0b"));
    assert_eq!(ok(&["-j1", "--set", ""]).set.as_deref(), Some(""));
}

#[test]
fn separator() {
    assert_eq!(ok(&["-j1", "--"]), ok(&["-j1"]));
    let e = err(&["-j1", "--", "-v"]);
    assert_eq!(e.kind(), ErrorKind::UnexpectedArg);
    assert_eq!(e.token(), Some("-v"));
    // Only the first `--` is a separator; the second is a word.
    let e = err(&["-j1", "--", "--"]);
    assert_eq!(e.kind(), ErrorKind::UnexpectedArg);
    assert_eq!(e.token(), Some("--"));
}

#[test]
fn errors() {
    let e = err(&["-j1", "--nope"]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnknownFlag, Some("--nope"))
    );

    // No abbreviations.
    assert_eq!(err(&["-j1", "--verb"]).kind(), ErrorKind::UnknownFlag);

    let e = err(&["-j1", "-vx"]);
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-x")));
    assert_eq!(e.offset(), 6, "points at the letter inside the bundle");

    let e = err(&["-j1", "stray"]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("stray"))
    );

    let e = err(&["-j1", "--path"]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingValue, Some("--path"))
    );
    let e = err(&["-j1", "--path", "--verbose"]);
    assert_eq!(e.kind(), ErrorKind::MissingValue);
    let e = err(&["-j1", "--path", "--"]);
    assert_eq!(e.kind(), ErrorKind::MissingValue);
    let e = err(&["-j1", "-p"]);
    assert_eq!((e.kind(), e.token()), (ErrorKind::MissingValue, Some("-p")));

    let e = err(&["-j1", "--verbose=yes"]);
    assert_eq!(
        (e.kind(), e.value()),
        (ErrorKind::UnexpectedValue, Some("yes"))
    );

    let e = err(&["-v"]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingRequired, Some("--jobs"))
    );

    let e = err(&["-jmany"]);
    assert_eq!(
        (e.kind(), e.value()),
        (ErrorKind::InvalidValue, Some("many"))
    );
    assert_eq!(
        e.to_string(),
        "invalid value `many` for `-j`: invalid digit found in string"
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_values_survive() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    let raw = OsStr::from_bytes(b"/tmp/\xff");
    let cli = Derived::parse_from(&words(&[OsStr::new("-j1"), OsStr::new("-p"), raw])).unwrap();
    assert_eq!(cli.path.as_deref().map(|p| p.as_os_str()), Some(raw));

    // Flags still match when a value is not UTF-8; only converting it to text fails.
    let e =
        Derived::parse_from(&words(&[OsStr::new("-j1"), OsStr::new("--set"), raw])).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
}

#[test]
fn composes_as_a_winnow_parser() {
    // `Derived::parse_argv` is a plain `Parser<Argv, Derived, Error>`.
    let words = [BStr::new("-j4")];
    let jobs = Derived::parse_argv
        .map(|d| d.jobs)
        .parse_next(&mut Argv::new(&words))
        .unwrap();
    assert_eq!(jobs, 4);
}
