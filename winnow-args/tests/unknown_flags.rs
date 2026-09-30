//! `unknown_flags = "value"`, usage's default: a flag-like word that names no
//! flag is offered to the positionals whole, and never selects a subcommand.
#![cfg(feature = "derive")]

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, positional, short};
use winnow_args::token::flag_word;
use winnow_args::{Args, Argv, Error, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value")]
struct Wrap {
    #[arg(short)]
    verbose: bool,
    #[arg(short, long)]
    jobs: Option<u32>,
    #[arg(positional)]
    args: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value")]
struct One {
    #[arg(positional)]
    file: Option<String>,
}

#[derive(Args, Debug, PartialEq)]
struct Strict {
    #[arg(positional)]
    args: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value", default_subcommand = "run")]
struct Root {
    #[arg(short, long, global)]
    quiet: bool,
    #[arg(positional)]
    task: Option<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run(RunArgs),
}

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value")]
struct RunArgs {
    #[arg(positional)]
    args: Vec<String>,
}

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    T::parse_from(&words)
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn an_unknown_long_is_a_value() {
    let w: Wrap = parse(&["--wat", "keep", "--wat=x"]).unwrap();
    assert_eq!(w.args, strings(&["--wat", "keep", "--wat=x"]));
    let w: Wrap = parse(&["--jobs", "2", "--wat"]).unwrap();
    assert_eq!((w.jobs, w.args), (Some(2), strings(&["--wat"])));
}

#[test]
fn a_bundle_is_checked_whole_before_any_letter_binds() {
    let w: Wrap = parse(&["-vx"]).unwrap();
    assert_eq!((w.verbose, w.args), (false, strings(&["-vx"])));
    // Letters after one that takes a value are that value.
    let w: Wrap = parse(&["-vj4"]).unwrap();
    assert_eq!((w.verbose, w.jobs), (true, Some(4)));
    assert_eq!(parse::<Wrap>(&["-1"]).unwrap().args, strings(&["-1"]));
    // `-h` is still help.
    let e = parse::<Wrap>(&["-vh"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested);
}

#[test]
fn with_nowhere_to_go_it_is_an_unexpected_argument() {
    let e = parse::<One>(&["a", "--wat"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("--wat"))
    );
    let e = parse::<One>(&["a", "-x"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("-x"))
    );
}

#[test]
fn strict_is_the_default() {
    let e = parse::<Strict>(&["--wat"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnknownFlag);
}

#[test]
fn an_unknown_flag_is_not_routed() {
    // usage's `an_unknown_flag_is_not_routed`: it binds where it was typed.
    for token in ["--wat", "-x"] {
        let r: Root = parse(&[token]).unwrap();
        assert_eq!(
            (r.task.as_deref(), r.command),
            (Some(token), None),
            "{token}"
        );
    }
}

#[test]
fn inherited_global_letters_are_known() {
    let r: Root = parse(&["run", "-q", "--wat"]).unwrap();
    assert!(r.quiet);
    assert_eq!(
        r.command,
        Some(Command::Run(RunArgs {
            args: strings(&["--wat"])
        }))
    );
    // `-q` is known through the parent, `x` is not: the whole word is a value.
    let r: Root = parse(&["run", "-qx"]).unwrap();
    assert!(!r.quiet);
    assert_eq!(
        r.command,
        Some(Command::Run(RunArgs {
            args: strings(&["-qx"])
        }))
    );
}

#[test]
fn the_combinator_takes_whole_words() {
    const VERBOSE: Named = short('v');
    const JOBS: Named = short('j').long("jobs");
    fn wrap(input: &mut Argv<'_>) -> Result<Wrap, Error> {
        let (mut verbose, mut jobs, mut rest) = (false, None, Vec::new());
        args(alt((
            VERBOSE.switch().map(|()| verbose = true),
            JOBS.argument_as().map(|j| jobs = Some(j)),
            // An unknown flag goes where a positional word would.
            alt((positional("ARGS"), flag_word.try_map(|w| w.convert("ARGS"))))
                .map(|a| rest.push(a)),
        )))
        .parse_next(input)?;
        Ok(Wrap {
            verbose,
            jobs,
            args: rest,
        })
    }
    for line in [
        &["--wat", "keep", "--wat=x"][..],
        &["--jobs", "2", "--wat"],
        &["-x", "-v"],
    ] {
        let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
        let a = wrap.parse_next(&mut Argv::new(&words)).unwrap();
        assert_eq!(a, parse::<Wrap>(line).unwrap(), "{line:?}");
    }
}

/// `long_only` and `unknown_flags = "value"` together: a known single-dash
/// long option is itself, an unknown one is a positional.
#[derive(Args, Debug, PartialEq)]
#[arg(long_only, unknown_flags = "value")]
struct LongOnlyLenient {
    #[arg(long)]
    shared: bool,
    #[arg(short = 's')]
    strip: bool,
    #[arg(positional)]
    args: Vec<String>,
}

#[test]
fn long_only_lenient() {
    let parsed = LongOnlyLenient::try_parse_from(["-shared", "-sQ", "-bogus", "x"]).unwrap();
    assert!(parsed.shared);
    assert!(!parsed.strip);
    assert_eq!(parsed.args, ["-sQ", "-bogus", "x"]);
}

/// An `unknown` field outside `long_only`: unknown flags apart from the
/// positionals, a known bundle still bound.
#[derive(Args, Debug, PartialEq)]
struct Collect {
    #[arg(short)]
    verbose: bool,
    #[arg(short, long)]
    jobs: Option<u32>,
    #[arg(unknown)]
    unknown: Vec<String>,
    #[arg(positional)]
    args: Vec<String>,
}

#[test]
fn unknown_field_collects_apart_from_positionals() {
    let parsed = Collect::try_parse_from([
        "-v", "--nope", "a", "-j2", "-vx", "--jobs=3", "-Z", "--", "-q",
    ])
    .unwrap();
    assert!(parsed.verbose);
    assert_eq!(parsed.jobs, Some(3));
    assert_eq!(parsed.unknown, ["--nope", "-vx", "-Z"]);
    assert_eq!(parsed.args, ["a", "-q"]);
}
