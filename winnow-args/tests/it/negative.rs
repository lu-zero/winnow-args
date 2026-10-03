//! `allow_negative_numbers`, following usage's corpus vectors: a negative number
//! is a value only where a flag or positional opts in.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow_args::combinator::{Named, args, long, short};
use winnow_args::token::number;
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(long, allow_negative_numbers)]
    offset: Option<f64>,
    #[arg(short, long)]
    verbose: bool,
    /// fd's `-0`: a declared digit short outranks a number.
    #[arg(short = '0', long = "print0")]
    print0: bool,
    #[arg(positional, allow_negative_numbers)]
    delta: Option<f64>,
    #[arg(positional)]
    rest: Vec<String>,
}

const OFFSET: Named = long("offset").allow_negative_numbers();
const VERBOSE: Named = short('v').long("verbose");
const PRINT0: Named = short('0').long("print0");

/// The combinators take the positional number with `token::number`.
fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        OFFSET.argument_as().map(|o| cli.offset = Some(o)),
        VERBOSE.switch().map(|()| cli.verbose = true),
        PRINT0.switch().map(|()| cli.print0 = true),
        number
            .try_map(|w| w.convert("DELTA"))
            .map(|d| cli.delta = Some(d)),
    )))
    .parse_next(input)?;
    Ok(cli)
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    Cli::try_parse_from(line)
}

#[test]
fn a_flag_that_opts_in_takes_a_negative_number() {
    // long-value-accepts-negative-number
    assert_eq!(parse(&["--offset", "-1"]).unwrap().offset, Some(-1.0));
    // A flag-like word is still not its value.
    let e = parse(&["--offset", "--verbose"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingValue);
}

#[test]
fn a_positional_that_opts_in_takes_a_negative_number() {
    // arg-value-looks-like-negative-number, short-exponent-number-is-a-value,
    // short-fractional-exponent-is-a-value
    for (word, value) in [
        ("-1", -1.0),
        ("-2.5", -2.5),
        ("-1e5", -1e5),
        ("-1.5e-3", -1.5e-3),
    ] {
        assert_eq!(parse(&[word]).unwrap().delta, Some(value), "{word}");
    }
    // Only while that positional is the next to fill.
    let e = parse(&["-2", "-3"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-3")));
}

#[test]
fn not_every_dash_word_is_a_number() {
    // A declared digit short keeps its spelling.
    let cli = parse(&["-0"]).unwrap();
    assert_eq!((cli.print0, cli.delta), (true, None));
    for word in ["-inf", "-1x", "-1e", "-."] {
        assert_eq!(
            parse(&[word]).unwrap_err().kind(),
            ErrorKind::UnknownFlag,
            "{word}"
        );
    }
}

#[test]
fn the_combinators_agree() {
    for line in [&["--offset", "-1"][..], &["-2.5"], &["-v", "-1e5"], &["-0"]] {
        let words = crate::words(line);
        let a = combinator.parse_next(&mut Argv::new(&words)).unwrap();
        let b = parse(line).unwrap();
        assert_eq!(
            (a.offset, a.delta, a.verbose, a.print0),
            (b.offset, b.delta, b.verbose, b.print0),
            "{line:?}"
        );
    }
}
