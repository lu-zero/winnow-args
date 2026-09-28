//! Positional arguments, run against the combinator parser and the derived
//! one: both must agree on every command line.
#![cfg(feature = "derive")]

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, positional, short};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Debug, PartialEq, Default)]
struct Cli {
    verbose: bool,
    path: Option<String>,
    input: String,
    output: Option<String>,
    rest: Vec<String>,
}

const VERBOSE: Named = short('v').long("verbose");
const PATH: Named = short('p').long("path");

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    let mut first = None;
    let mut position = 0;
    args(alt((
        VERBOSE.switch().map(|()| cli.verbose = true),
        PATH.argument_as().map(|p| cli.path = Some(p)),
        positional::<String>("WORD").map(|word| {
            match position {
                0 => first = Some(word),
                1 => cli.output = Some(word),
                _ => cli.rest.push(word),
            }
            position += 1;
        }),
    )))
    .parse_next(input)?;
    cli.input = first.ok_or_else(|| Error::missing_argument(input.offset(), "INPUT"))?;
    Ok(cli)
}

#[derive(Args, Debug, PartialEq)]
struct Derived {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    path: Option<String>,
    #[arg(positional)]
    input: String,
    #[arg(positional)]
    output: Option<String>,
    #[arg(positional)]
    rest: Vec<String>,
}

impl From<Derived> for Cli {
    fn from(d: Derived) -> Self {
        let Derived {
            verbose,
            path,
            input,
            output,
            rest,
        } = d;
        Cli {
            verbose,
            path,
            input,
            output,
            rest,
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

fn ok(line: &[&str]) -> Cli {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn err(line: &[&str]) -> Error {
    match parse(line) {
        Ok(cli) => panic!("{line:?} parsed as {cli:?}"),
        Err(e) => e,
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn fill_in_declaration_order() {
    let cli = ok(&["a"]);
    assert_eq!(
        (cli.input.as_str(), cli.output, cli.rest),
        ("a", None, vec![])
    );

    let cli = ok(&["a", "b", "c", "d"]);
    assert_eq!(cli.input, "a");
    assert_eq!(cli.output.as_deref(), Some("b"));
    assert_eq!(cli.rest, strings(&["c", "d"]));
}

#[test]
fn interleave_with_flags() {
    let cli = ok(&["a", "-v", "b", "--path=x", "c"]);
    assert!(cli.verbose);
    assert_eq!(cli.path.as_deref(), Some("x"));
    assert_eq!(
        (cli.input.as_str(), cli.output.as_deref()),
        ("a", Some("b"))
    );
    assert_eq!(cli.rest, strings(&["c"]));

    // A detached value belongs to its flag, not to the positionals.
    let cli = ok(&["--path", "a", "b"]);
    assert_eq!((cli.path.as_deref(), cli.input.as_str()), (Some("a"), "b"));
}

#[test]
fn words_that_look_like_flags() {
    assert_eq!(ok(&["-"]).input, "-");
    let cli = ok(&["a", "--", "--path", "-v"]);
    assert!(!cli.verbose);
    assert_eq!(cli.output.as_deref(), Some("--path"));
    assert_eq!(cli.rest, strings(&["-v"]));
    assert_eq!(ok(&["--", "--"]).input, "--");
}

#[test]
fn errors() {
    let e = err(&["-v"]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingArgument, Some("INPUT"))
    );
    assert_eq!(err(&["a", "--nope"]).kind(), ErrorKind::UnknownFlag);
}

#[derive(Args, Debug)]
struct One {
    #[arg(positional, value_name = "N")]
    n: u32,
}

#[test]
fn extra_and_invalid_words() {
    assert_eq!(One::parse_from(&[BStr::new("7")]).unwrap().n, 7);

    let words = [BStr::new("1"), BStr::new("2")];
    let e = One::parse_from(&words).unwrap_err();
    assert_eq!(
        (e.kind(), e.token(), e.offset()),
        (ErrorKind::UnexpectedArg, Some("2"), 2)
    );

    let words = [BStr::new("x")];
    let e = One::parse_from(&words).unwrap_err();
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::InvalidValue, Some("N"), Some("x"))
    );
}
