//! `default_subcommand` and `arg_required_else_help`. The first follows usage's
//! corpus, `09-default-subcommand.json`; each test names the vector it mirrors.

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    T::parse_from(&words)
}

#[derive(Args, Debug, PartialEq)]
#[arg(default_subcommand = "run")]
struct Ex {
    #[arg(long)]
    verbose: bool,
    #[arg(positional, value_name = "ROOT_TASK")]
    root_task: Option<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run(RunArgs),
    Install,
}

#[derive(Args, Debug, PartialEq)]
struct RunArgs {
    #[arg(positional)]
    task: Option<String>,
    #[arg(subcommand)]
    command: Option<RunCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum RunCommand {
    Lint,
}

fn run(task: Option<&str>, command: Option<RunCommand>) -> Option<Command> {
    Some(Command::Run(RunArgs {
        task: task.map(Into::into),
        command,
    }))
}

#[test]
fn default_route_and_default_beats_the_root_own_arg() {
    let ex: Ex = parse(&["build"]).unwrap();
    assert_eq!((ex.root_task, ex.command), (None, run(Some("build"), None)));
}

#[test]
fn default_not_used_when_named() {
    assert_eq!(
        parse::<Ex>(&["install"]).unwrap().command,
        Some(Command::Install)
    );
}

#[test]
fn default_word_reexamined() {
    // The word that selected the default is read again by it, as its subcommand.
    assert_eq!(
        parse::<Ex>(&["lint"]).unwrap().command,
        run(None, Some(RunCommand::Lint))
    );
}

#[test]
fn default_flag_before_the_word() {
    let ex: Ex = parse(&["--verbose", "build"]).unwrap();
    assert_eq!((ex.verbose, ex.command), (true, run(Some("build"), None)));
}

#[test]
fn default_after_separator() {
    let ex: Ex = parse(&["--", "build"]).unwrap();
    assert_eq!((ex.root_task.as_deref(), ex.command), (Some("build"), None));
}

#[test]
fn option_off_keeps_root() {
    // `17-default-subcommand-on-empty.json`: an empty line selects nothing.
    assert_eq!(parse::<Ex>(&[]).unwrap().command, None);
}

#[test]
fn default_not_taken_by_an_unknown_flag() {
    // usage's default is lenient, handing `--wat` to ROOT_TASK; winnow-args is
    // strict (see the checklist), so it is an unknown flag. Either way, the default
    // subcommand does not take it.
    assert_eq!(
        parse::<Ex>(&["--wat"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
    assert_eq!(
        parse::<Ex>(&["-x"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[derive(Args, Debug, PartialEq)]
#[arg(default_subcommand = "r")]
struct ByAlias {
    #[arg(subcommand)]
    command: Option<AliasCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum AliasCommand {
    #[arg(alias = "r")]
    Run(RunArgs),
}

#[test]
fn default_named_by_an_alias() {
    assert_eq!(
        parse::<ByAlias>(&["build"]).unwrap().command,
        Some(AliasCommand::Run(RunArgs {
            task: Some("build".into()),
            command: None
        }))
    );
}

#[derive(Args, Debug)]
#[arg(default_subcommand = "ls")]
struct RootOnly {
    #[arg(subcommand)]
    command: Option<RootOnlyCommand>,
}

#[derive(Subcommand, Debug)]
enum RootOnlyCommand {
    Ls,
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
struct ConfigArgs {
    #[arg(subcommand)]
    command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Ls(LsArgs),
}

#[derive(Args, Debug)]
struct LsArgs {
    #[arg(positional)]
    what: Option<String>,
}

#[test]
fn default_is_declared_for_the_root() {
    // `config` declares no default, so `zzz` is just unexpected there.
    let e = parse::<RootOnly>(&["config", "zzz"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("zzz"))
    );
    assert!(matches!(
        parse::<RootOnly>(&["config", "ls", "x"]).unwrap().command,
        Some(RootOnlyCommand::Config(ConfigArgs {
            command: Some(ConfigCommand::Ls(LsArgs { what: Some(_) }))
        }))
    ));
}

#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct Help {
    #[arg(short, long, global)]
    verbose: bool,
    #[arg(subcommand)]
    command: Option<HelpCommand>,
}

#[derive(Subcommand, Debug)]
enum HelpCommand {
    Run(RunHelp),
}

#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct RunHelp {
    #[arg(positional)]
    task: Option<String>,
}

#[test]
fn a_bare_invocation_asks_for_help() {
    assert_eq!(
        parse::<Help>(&[]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    // A global flag before the subcommand is not the subcommand's argument.
    assert_eq!(
        parse::<Help>(&["-v", "run"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    let help: Help = parse(&["-v", "run", "x"]).unwrap();
    assert!(help.verbose);
    assert!(matches!(
        help.command,
        Some(HelpCommand::Run(RunHelp { task: Some(_) }))
    ));
}
