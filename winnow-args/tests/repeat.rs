//! Repeatable options, run against the combinator parser and the derived one.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, long, positional, short};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Debug, PartialEq, Default)]
struct Cli {
    include: Vec<String>,
    num: Vec<u32>,
    files: Vec<String>,
}

const INCLUDE: Named = short('I').long("include");
const NUM: Named = long("num");

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        INCLUDE.argument_as().map(|v| cli.include.push(v)),
        NUM.argument_as().map(|v| cli.num.push(v)),
        positional("FILES").map(|v| cli.files.push(v)),
    )))
    .parse_next(input)?;
    Ok(cli)
}

#[derive(Args, Debug, PartialEq)]
struct Derived {
    #[arg(short = 'I', long)]
    include: Vec<String>,
    num: Vec<u32>,
    #[arg(positional)]
    files: Vec<String>,
}

impl From<Derived> for Cli {
    fn from(d: Derived) -> Self {
        let Derived {
            include,
            num,
            files,
        } = d;
        Cli {
            include,
            num,
            files,
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

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn every_occurrence_in_order() {
    assert_eq!(ok(&[]), Cli::default());
    let cli = ok(&["-I", "a", "--include=b", "-Ic", "--include", "d", "-I=e"]);
    assert_eq!(cli.include, strings(&["a", "b", "c", "d", "e"]));
    assert_eq!(ok(&["--num", "3", "--num=1"]).num, vec![3, 1]);
}

#[test]
fn one_value_per_occurrence() {
    // A repeatable flag takes one word; the next word is a positional.
    let cli = ok(&["x", "-I", "a", "y", "-Ib", "z"]);
    assert_eq!(cli.include, strings(&["a", "b"]));
    assert_eq!(cli.files, strings(&["x", "y", "z"]));
}

#[test]
fn errors_name_the_occurrence() {
    let e = parse(&["-I", "a", "-I"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::MissingValue, Some("-I")));

    let e = parse(&["--num", "1", "--num", "x"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.value(), e.offset()),
        (ErrorKind::InvalidValue, Some("x"), 8)
    );
}
