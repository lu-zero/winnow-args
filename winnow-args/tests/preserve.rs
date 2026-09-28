//! `double_dash = "preserve"`: a `--` reaching the positional is one of its
//! values and stops nothing (usage's `double_dash_preserve_keeps_the_separator`).

use std::cell::RefCell;
use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, positional, short};

use winnow_args::token::{Word, separator_word};
use winnow_args::{Args, Argv, Error};

#[derive(Args, Debug, PartialEq, Default)]
struct Wrap {
    #[arg(short)]
    verbose: bool,
    #[arg(positional)]
    tool: Option<String>,
    #[arg(positional, double_dash = "preserve")]
    args: Vec<String>,
}

const VERBOSE: Named = short('v');

/// The combinators: `--` is a word only once `TOOL` is filled.
fn combinator(input: &mut Argv<'_>) -> Result<Wrap, Error> {
    let w = RefCell::new(Wrap::default());
    let has_tool = || w.borrow().tool.is_some();
    args(alt((
        VERBOSE.switch().map(|()| w.borrow_mut().verbose = true),
        positional("TOOL")
            .verify(|_: &String| !has_tool())
            .map(|t| w.borrow_mut().tool = Some(t)),
        positional("ARGS").map(|a| w.borrow_mut().args.push(a)),
        separator_word
            .verify(|_: &Word<'_>| has_tool())
            .try_map(|s| s.convert("ARGS"))
            .map(|a| w.borrow_mut().args.push(a)),
    )))
    .parse_next(input)?;
    Ok(w.into_inner())
}

fn parse(line: &[&str]) -> Wrap {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    let a = combinator.parse_next(&mut Argv::new(&words)).unwrap();
    let b = Wrap::parse_from(&words).unwrap();
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    b
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn the_separator_is_a_value() {
    let w = parse(&["npm", "a", "--", "b"]);
    assert_eq!(w.args, strings(&["a", "--", "b"]));
    assert_eq!(parse(&["npm", "--"]).args, strings(&["--"]));
}

#[test]
fn flags_go_on_after_it() {
    let w = parse(&["npm", "--", "-v"]);
    assert_eq!((w.verbose, w.args), (true, strings(&["--"])));
}

#[test]
fn before_the_preserving_positional_it_is_a_separator() {
    let w = parse(&["--", "-v", "x"]);
    assert!(!w.verbose);
    assert_eq!(w.tool.as_deref(), Some("-v"));
    assert_eq!(w.args, strings(&["x"]));
}
