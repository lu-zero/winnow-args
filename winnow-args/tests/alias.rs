//! Aliases for long flags and subcommands, run against the combinators and the derive.
#![cfg(feature = "derive")]

use winnow::combinator::{alt, dispatch, fail};
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, command, long, positional, short};
use winnow_args::token::{Kind, finish, kind};
use winnow_args::{Args, Argv, Error, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short, long, alias = "loud", global)]
    verbose: bool,
    #[arg(short = 'p', long = "tool", alias = "plugin")]
    tool: Option<String>,
    #[arg(alias("inc", "incl"))]
    include: Vec<String>,
    /// `--legacy` by default, plus its alias.
    #[arg(alias = "old-name")]
    legacy: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    #[arg(name = "ls", alias = "list")]
    Ls,
    #[arg(alias("rm", "remove"))]
    Unset(UnsetArgs),
}

#[derive(Args, Debug, PartialEq, Default)]
struct UnsetArgs {
    #[arg(positional)]
    key: String,
}

const VERBOSE: Named<2> = short('v').longs(["verbose", "loud"]);
const TOOL: Named<2> = short('p').longs(["tool", "plugin"]);
const INCLUDE: Named<3> = long("include").longs(["include", "inc", "incl"]);
const LEGACY: Named<2> = long("legacy").longs(["legacy", "old-name"]);

fn unset<'a, 'i>(verbose: &'a mut bool) -> impl Parser<Argv<'i>, UnsetArgs, Error> + 'a {
    move |input: &mut Argv<'i>| {
        let mut key = None;
        let v = &mut *verbose;
        args(alt((
            VERBOSE.switch().map(|()| *v = true),
            positional("KEY").map(|k| key = Some(k)),
        )))
        .parse_next(input)?;
        let key = key.ok_or_else(|| Error::missing_argument(input.offset(), "KEY"))?;
        Ok(UnsetArgs { key })
    }
}

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    let c = &mut cli;
    args(dispatch! {kind;
        Kind::Long | Kind::Short => alt((
            VERBOSE.switch().map(|()| c.verbose = true),
            TOOL.argument_as().map(|t| c.tool = Some(t)),
            INCLUDE.argument_as().map(|i| c.include.push(i)),
            LEGACY.switch().map(|()| c.legacy = true),
        )),
        Kind::Word => alt((
            command(["ls", "list"], finish).map(|()| Command::Ls),
            command(["unset", "rm", "remove"], unset(&mut c.verbose)).map(Command::Unset),
        ))
        .map(|found| c.command = Some(found)),
        Kind::Separator => fail,
    })
    .parse_next(input)?;
    Ok(cli)
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Cli::parse_from(&words);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

fn ok(line: &[&str]) -> Cli {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

#[test]
fn a_long_flag_answers_to_its_aliases() {
    for line in [
        &["--plugin", "x"][..],
        &["--tool=x"],
        &["-px"],
        &["--plugin=x"],
    ] {
        assert_eq!(ok(line).tool.as_deref(), Some("x"), "{line:?}");
    }
    assert_eq!(
        ok(&["--inc", "a", "--incl=b", "--include", "c"]).include,
        ["a", "b", "c"]
    );
    assert!(ok(&["--old-name"]).legacy);
    assert!(ok(&["--legacy"]).legacy);
    // Aliases match exactly.
    assert_eq!(
        parse(&["--plug", "x"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[test]
fn a_subcommand_answers_to_its_aliases() {
    assert_eq!(ok(&["ls"]).command, Some(Command::Ls));
    assert_eq!(ok(&["list"]).command, Some(Command::Ls));
    for name in ["unset", "rm", "remove"] {
        let cli = ok(&[name, "k", "--loud"]);
        assert_eq!(
            cli.command,
            Some(Command::Unset(UnsetArgs { key: "k".into() }))
        );
        assert!(cli.verbose, "global reached through its alias after {name}");
    }
}

#[derive(Args, Debug)]
struct TwoLongs {
    /// A second `long` is another spelling, as mise's shadow writes it.
    #[arg(long = "path", long = "file", short = 'p')]
    path: Option<String>,
}

#[test]
fn a_repeated_long_adds_a_spelling() {
    for line in [&["--path", "x"][..], &["--file=x"], &["-px"]] {
        let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
        assert_eq!(
            TwoLongs::parse_from(&words).unwrap().path.as_deref(),
            Some("x"),
            "{line:?}"
        );
    }
}
