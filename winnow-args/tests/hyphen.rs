//! `allow_hyphen_values`, following usage's corpus: the flag takes the next word
//! whatever it looks like.
#![cfg(feature = "derive")]

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, long, positional, short};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short = 'd', long)]
    working_dir: Option<String>,
    #[arg(short, long, allow_hyphen_values)]
    args: Option<String>,
    #[arg(long, allow_hyphen_values)]
    each: Vec<String>,
    #[arg(positional)]
    rest: Vec<String>,
}

const WORKING_DIR: Named = short('d').long("working-dir");
const ARGS: Named = short('a').long("args").allow_hyphen_values();
const EACH: Named = long("each").allow_hyphen_values();

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        WORKING_DIR.argument_as().map(|d| cli.working_dir = Some(d)),
        ARGS.argument_as().map(|a| cli.args = Some(a)),
        EACH.argument_as().map(|e| cli.each.push(e)),
        positional("REST").map(|r| cli.rest.push(r)),
    )))
    .parse_next(input)?;
    Ok(cli)
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Cli::parse_from(&words);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    b
}

#[test]
fn the_next_word_is_the_value_whatever_it_looks_like() {
    // long-value-allow-hyphen-takes-flaglike, short-value-allow-hyphen-takes-flaglike
    assert_eq!(
        parse(&["--args", "-destroy"]).unwrap().args.as_deref(),
        Some("-destroy")
    );
    assert_eq!(
        parse(&["-a", "-destroy"]).unwrap().args.as_deref(),
        Some("-destroy")
    );
    // long-repeatable-allow-hyphen-each-occurrence
    assert_eq!(
        parse(&["--each", "-val1", "--each", "-val2"]).unwrap().each,
        ["-val1", "-val2"]
    );
}

#[test]
fn it_takes_the_separator_as_its_value() {
    // long-value-allow-hyphen-takes-the-separator: `--` is the value, not a separator,
    // so later words are read normally. usage then hands `-x` to `rest` because its
    // unknown flags default to words; winnow-args is strict (see the checklist).
    let cli = parse(&["--args", "--", "a"]).unwrap();
    assert_eq!(
        (cli.args.as_deref(), cli.rest),
        (Some("--"), vec!["a".to_string()])
    );
    let e = parse(&["--args", "--", "-x"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-x")));
}

#[test]
fn only_where_declared() {
    let e = parse(&["-d", "-x"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::MissingValue, Some("-d")));
    // A value is still needed.
    assert_eq!(
        parse(&["--args"]).unwrap_err().kind(),
        ErrorKind::MissingValue
    );
}
