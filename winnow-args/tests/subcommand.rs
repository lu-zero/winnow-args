//! Subcommands, run against the combinator parser and the derived one: the
//! combinators build the derived types by hand, so the results compare directly.

use winnow::combinator::{alt, cond, dispatch, fail};
use winnow::prelude::*;
use winnow::stream::BStr;
use winnow_args::combinator::{Named, args, command, positional, short};
use winnow_args::token::{Kind, finish, kind};
use winnow_args::{Args, Argv, Error, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional)]
    files: Vec<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Use(UseArgs),
    #[arg(name = "ls")]
    List,
    DryRun(Box<DryRunArgs>),
}

#[derive(Args, Debug, PartialEq, Default)]
struct UseArgs {
    #[arg(short, long)]
    global: bool,
    #[arg(positional)]
    tools: Vec<String>,
}

#[derive(Args, Debug, PartialEq, Default)]
struct DryRunArgs {
    #[arg(short)]
    n: Option<u32>,
}

const VERBOSE: Named = short('v').long("verbose");
const GLOBAL: Named = short('g').long("global");
const N: Named = short('n');

fn use_args(input: &mut Argv<'_>) -> Result<UseArgs, Error> {
    let mut u = UseArgs::default();
    let c = &mut u;
    args(dispatch! {kind;
        Kind::Long | Kind::Short => GLOBAL.switch().map(|()| c.global = true),
        Kind::Word => positional("TOOLS").map(|t| c.tools.push(t)),
        Kind::Separator => fail,
    })
    .parse_next(input)?;
    Ok(u)
}

fn dry_run_args(input: &mut Argv<'_>) -> Result<DryRunArgs, Error> {
    let mut d = DryRunArgs::default();
    args(N.argument_as().map(|n| d.n = Some(n))).parse_next(input)?;
    Ok(d)
}

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    let c = &mut cli;
    args(dispatch! {kind;
        Kind::Long | Kind::Short => VERBOSE.switch().map(|()| c.verbose = true),
        Kind::Word => alt((
            // A word only names a subcommand before any positional is filled.
            cond(c.files.is_empty(), alt((
                command("use", use_args).map(Command::Use),
                command("ls", finish).map(|()| Command::List),
                command("dry-run", dry_run_args).map(|d| Command::DryRun(Box::new(d))),
            )))
            .verify_map(|found| found)
            .map(|found| c.command = Some(found)),
            positional("FILES").map(|f| c.files.push(f)),
        )),
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

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn a_word_selects_the_subcommand() {
    assert_eq!(ok(&[]).command, None);
    let cli = ok(&["-v", "use", "-g", "node@20", "python"]);
    assert!(cli.verbose);
    assert_eq!(
        cli.command,
        Some(Command::Use(UseArgs {
            global: true,
            tools: strings(&["node@20", "python"])
        }))
    );
    assert_eq!(ok(&["ls"]).command, Some(Command::List));
    assert_eq!(
        ok(&["dry-run", "-n", "3"]).command,
        Some(Command::DryRun(Box::new(DryRunArgs { n: Some(3) })))
    );
}

#[test]
fn only_before_a_positional_and_before_the_separator() {
    let cli = ok(&["a", "use"]);
    assert_eq!((cli.files, cli.command), (strings(&["a", "use"]), None));
    let cli = ok(&["--", "use"]);
    assert_eq!((cli.files, cli.command), (strings(&["use"]), None));
    assert_eq!(ok(&["nope"]).files, strings(&["nope"]));
}

#[test]
fn the_rest_of_the_line_belongs_to_the_subcommand() {
    // No global flags yet: the parent's `-v` is unknown to `use`.
    let e = parse(&["use", "-v"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-v")));
    let e = parse(&["ls", "x"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnexpectedArg, Some("x")));
}

#[derive(Args, Debug)]
struct Required {
    #[arg(subcommand)]
    command: Command,
}

#[test]
fn required_subcommand_and_enum_as_the_whole_line() {
    let e = Required::parse_from(&[]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingSubcommand);
    let words = [BStr::new("ls")];
    assert_eq!(Required::parse_from(&words).unwrap().command, Command::List);

    assert_eq!(Command::parse_from(&words).unwrap(), Command::List);
    assert_eq!(
        Command::parse_from(&[]).unwrap_err().kind(),
        ErrorKind::MissingSubcommand
    );
    let words = [BStr::new("-x")];
    assert_eq!(
        Command::parse_from(&words).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[derive(Args, Debug, PartialEq)]
struct Outer {
    #[arg(subcommand)]
    command: Option<OuterCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum OuterCommand {
    Tool(ToolArgs),
}

#[derive(Args, Debug, PartialEq)]
struct ToolArgs {
    #[arg(short)]
    quiet: bool,
    #[arg(subcommand)]
    command: Command,
}

#[test]
fn subcommands_nest() {
    let words: Vec<&BStr> = ["tool", "-q", "use", "-g", "x"]
        .iter()
        .map(BStr::new)
        .collect();
    assert_eq!(
        Outer::parse_from(&words).unwrap().command,
        Some(OuterCommand::Tool(ToolArgs {
            quiet: true,
            command: Command::Use(UseArgs {
                global: true,
                tools: strings(&["x"])
            }),
        }))
    );
    let words = [BStr::new("tool")];
    assert_eq!(
        Outer::parse_from(&words).unwrap_err().kind(),
        ErrorKind::MissingSubcommand
    );
}
