//! Choices: value enums (both styles) and `choices(...)` on string fields (derive).
#![cfg(feature = "derive")]

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, long};
use winnow_args::{Args, Argv, Error, ErrorKind, ValueEnum};

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Color {
    Auto,
    Always,
    Never,
}

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Part {
    Plugins,
    #[arg(name = "pkgs", alias = "packages")]
    Packages,
    MiseShellActivate,
}

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(long)]
    color: Option<Color>,
    #[arg(long, delimiter = ',')]
    skip: Vec<Part>,
}

const COLOR: Named = long("color");
const SKIP: Named = long("skip");

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        COLOR.argument_as().map(|c| cli.color = Some(c)),
        SKIP.arguments_as(b',')
            .map(|p: Vec<Part>| cli.skip.extend(p)),
    )))
    .parse_next(input)?;
    Ok(cli)
}

fn words<'a>(line: &'a [&'a str]) -> Vec<&'a BStr> {
    line.iter().map(BStr::new).collect()
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words = words(line);
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Cli::parse_from(&words);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

#[test]
fn a_value_enum_matches_its_names() {
    assert_eq!(
        parse(&["--color", "always"]).unwrap().color,
        Some(Color::Always)
    );
    assert_eq!(parse(&["--color=never"]).unwrap().color, Some(Color::Never));
    let cli = parse(&[
        "--skip",
        "plugins,pkgs,packages",
        "--skip=mise-shell-activate",
    ])
    .unwrap();
    assert_eq!(
        cli.skip,
        [
            Part::Plugins,
            Part::Packages,
            Part::Packages,
            Part::MiseShellActivate
        ]
    );
}

#[test]
fn anything_else_is_an_invalid_choice() {
    let e = parse(&["--color", "blue"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.value()),
        (ErrorKind::InvalidChoice, Some("blue"))
    );
    assert_eq!(
        e.to_string(),
        "invalid value `blue` for `--color`: expected one of auto, always, never"
    );
    // Names match exactly, and the listing leaves aliases out.
    let e = parse(&["--skip", "plugins,Plugins"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.value()),
        (ErrorKind::InvalidChoice, Some("Plugins"))
    );
    assert!(
        e.to_string()
            .ends_with("expected one of plugins, pkgs, mise-shell-activate")
    );
}

#[derive(Args, Debug, PartialEq)]
struct Strings {
    #[arg(short, long, choices("bash", "zsh", "fish"))]
    shell: Option<String>,
    #[arg(positional, choices("up", "down"))]
    direction: Option<String>,
}

#[test]
fn string_choices_are_checked_before_conversion() {
    let line = words(&["-s", "zsh", "up"]);
    let parsed = Strings::parse_from(&line).unwrap();
    assert_eq!(
        (parsed.shell.as_deref(), parsed.direction.as_deref()),
        (Some("zsh"), Some("up"))
    );

    let line = words(&["--shell=tcsh"]);
    let e = Strings::parse_from(&line).unwrap_err();
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::InvalidChoice, Some("--shell"), Some("tcsh"))
    );

    let line = words(&["-s", "bash", "left"]);
    let e = Strings::parse_from(&line).unwrap_err();
    assert_eq!(
        (e.kind(), e.token(), e.offset()),
        (ErrorKind::InvalidChoice, Some("DIRECTION"), 8)
    );
}

/// `#[arg(rename_all = "lowercase")]`: bash's completion actions are
/// `arrayvar`, `bashdefault`, not kebab-case.
#[derive(winnow_args::ValueEnum, Debug, PartialEq)]
#[arg(rename_all = "lowercase")]
enum Action {
    ArrayVar,
    BashDefault,
    #[arg(name = "file")]
    FileName,
}

#[test]
fn rename_all_spells_every_variant() {
    use winnow_args::FromArg;
    let parse = |s: &str| Action::from_arg(BStr::new(s)).map_err(|e| e.to_string());
    assert_eq!(parse("arrayvar"), Ok(Action::ArrayVar));
    assert_eq!(parse("bashdefault"), Ok(Action::BashDefault));
    assert_eq!(parse("file"), Ok(Action::FileName));
    assert!(parse("array-var").is_err());
    assert_eq!(Action::CHOICES, ["arrayvar", "bashdefault", "file"]);
}
