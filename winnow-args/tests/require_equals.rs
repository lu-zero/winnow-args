//! `require_equals`, following usage's corpus: only an attached value binds.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, long, positional, short};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short, long, require_equals)]
    inspect: Option<String>,
    /// aube's `--inspect-brk`: with a missing default, a bare flag still works.
    #[arg(long, require_equals, default_missing = "9229")]
    debug: Option<String>,
    #[arg(positional)]
    rest: Option<String>,
}

const INSPECT: Named = short('i').long("inspect").require_equals();
const DEBUG: Named = long("debug").require_equals();

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        INSPECT.argument_as().map(|i| cli.inspect = Some(i)),
        DEBUG.argument_or("9229").map(|d| cli.debug = Some(d)),
        positional("REST").map(|r| cli.rest = Some(r)),
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
fn only_an_attached_value_binds() {
    // long-value-require-equals-attached, -still-takes-empty-attached,
    // short-value-require-equals-attached, short-value-require-equals-equals-form
    for (line, value) in [
        (&["--inspect=9229"][..], "9229"),
        (&["--inspect="], ""),
        (&["-i9229"], "9229"),
        (&["-i=9229"], "9229"),
    ] {
        assert_eq!(
            parse(line).unwrap().inspect.as_deref(),
            Some(value),
            "{line:?}"
        );
    }
}

#[test]
fn the_next_word_is_refused() {
    // long-/short-value-require-equals-refuses-detached
    for line in [&["--inspect", "9229"][..], &["-i", "9229"]] {
        let e = parse(line).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::MissingValue, "{line:?}");
    }
}

#[test]
fn with_a_missing_default_the_next_word_is_a_positional() {
    // long-value-default-missing-with-require-equals
    let cli = parse(&["--debug", "80"]).unwrap();
    assert_eq!(
        (cli.debug.as_deref(), cli.rest.as_deref()),
        (Some("9229"), Some("80"))
    );
    assert_eq!(parse(&["--debug=1"]).unwrap().debug.as_deref(), Some("1"));
}
