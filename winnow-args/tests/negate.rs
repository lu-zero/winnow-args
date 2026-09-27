//! `negate`: a second long spelling that sets a `bool` false, as usage's
//! `negate = "--no-color"`. The last spelling given wins; a default or the
//! environment fills only what neither spelling set.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{args, long};
use winnow_args::{Args, Argv, Error, ErrorKind, Subcommand, with_env};

#[derive(Args, Debug, PartialEq)]
struct Cli {
    /// Colorize output
    #[arg(long, negate = "--no-color", default = "true")]
    color: bool,
    /// `no-cache`, from the long name.
    #[arg(long, negate)]
    cache: bool,
    #[arg(long, negate, env = "NEGATE_PROGRESS", default = "true")]
    progress: bool,
    #[arg(long, negate, global)]
    keep: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run,
}

fn parse(line: &[&str], env: &[(&str, &str)]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    with_env(env, || Cli::parse_from(&words))
}

fn ok(line: &[&str]) -> Cli {
    parse(line, &[]).unwrap()
}

#[test]
fn the_last_spelling_wins() {
    for (line, color) in [
        (&[][..], true),
        (&["--no-color"], false),
        (&["--color"], true),
        (&["--color", "--no-color"], false),
        (&["--no-color", "--color"], true),
    ] {
        assert_eq!(ok(line).color, color, "{line:?}");
    }
    assert!(!ok(&[]).cache);
    assert!(ok(&["--cache"]).cache);
    assert!(!ok(&["--cache", "--no-cache"]).cache);
}

#[test]
fn a_negation_takes_no_value() {
    let e = parse(&["--no-color=yes"], &[]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnexpectedValue);
}

#[test]
fn the_environment_fills_only_what_was_not_given() {
    assert!(ok(&[]).progress);
    let env = [("NEGATE_PROGRESS", "0")];
    assert!(!parse(&[], &env).unwrap().progress);
    assert!(parse(&["--progress"], &env).unwrap().progress);
    let env = [("NEGATE_PROGRESS", "1")];
    assert!(!parse(&["--no-progress"], &env).unwrap().progress);
}

#[test]
fn a_global_negation_reaches_subcommands() {
    let cli = ok(&["--keep", "run", "--no-keep"]);
    assert_eq!((cli.keep, cli.command), (false, Some(Command::Run)));
}

#[test]
fn help_shows_both_spellings() {
    let e = parse(&["-h"], &[]).unwrap_err();
    let help = e.render_help("negate").unwrap();
    assert!(
        help.contains("--color / --no-color        Colorize output [default: true]"),
        "{help}"
    );
}

#[test]
fn the_combinator_agrees() {
    const COLOR: winnow_args::combinator::Named = long("color");
    const NO_COLOR: winnow_args::combinator::Named = long("no-color");
    fn color(input: &mut Argv<'_>) -> Result<bool, Error> {
        let mut color = None;
        args(alt((COLOR.negated_by(NO_COLOR).map(|c| color = Some(c)),))).parse_next(input)?;
        Ok(color.unwrap_or(true))
    }
    for line in [
        &[][..],
        &["--no-color"],
        &["--no-color", "--color"],
        &["--color", "--no-color"],
    ] {
        let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
        let a = color.parse_next(&mut Argv::new(&words)).unwrap();
        assert_eq!(a, ok(line).color, "{line:?}");
    }
}
