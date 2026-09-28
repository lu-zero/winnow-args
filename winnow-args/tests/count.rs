//! Counting switches, run against the combinator parser and the derived one.
#![cfg(feature = "derive")]

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, short};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Debug, PartialEq, Default)]
struct Cli {
    verbose: u8,
    quiet: usize,
    path: Option<String>,
}

const VERBOSE: Named = short('v').long("verbose");
const QUIET: Named = short('q');
const PATH: Named = short('p');

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        VERBOSE
            .switch()
            .map(|()| cli.verbose = cli.verbose.saturating_add(1)),
        QUIET.switch().map(|()| cli.quiet += 1),
        PATH.argument_as().map(|p| cli.path = Some(p)),
    )))
    .parse_next(input)?;
    Ok(cli)
}

#[derive(Args, Debug, PartialEq)]
struct Derived {
    #[arg(short, long, count)]
    verbose: u8,
    #[arg(short, count)]
    quiet: usize,
    #[arg(short)]
    path: Option<String>,
}

impl From<Derived> for Cli {
    fn from(d: Derived) -> Self {
        let Derived {
            verbose,
            quiet,
            path,
        } = d;
        Cli {
            verbose,
            quiet,
            path,
        }
    }
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Derived::parse_from(&words).map(Cli::from);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

fn counts(line: &[&str]) -> (u8, usize) {
    let cli = parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"));
    (cli.verbose, cli.quiet)
}

#[test]
fn every_spelling_counts() {
    assert_eq!(counts(&[]), (0, 0));
    assert_eq!(counts(&["-vvv"]), (3, 0));
    assert_eq!(counts(&["-v", "--verbose", "-vv"]), (4, 0));
    assert_eq!(counts(&["-vqvq", "-q"]), (2, 3));
}

#[test]
fn a_value_taking_letter_ends_the_count() {
    let cli = parse(&["-vvpvv"]).unwrap();
    assert_eq!((cli.verbose, cli.path.as_deref()), (2, Some("vv")));
}

#[test]
fn saturates_instead_of_wrapping() {
    let line = vec!["-v"; 300];
    assert_eq!(counts(&line).0, u8::MAX);
}

#[test]
fn a_count_takes_no_value() {
    let e = parse(&["--verbose=2"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.value()),
        (ErrorKind::UnexpectedValue, Some("2"))
    );
}
