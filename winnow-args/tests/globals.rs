//! Global flags: declared on a parent, accepted after its subcommand word at any depth.
#![cfg(feature = "derive")]

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq, Default)]
struct Root {
    #[arg(short, long, count, global)]
    verbose: u8,
    #[arg(short = 'C', long, global)]
    cd: Option<String>,
    #[arg(short)]
    local: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run(RunArgs),
    Ls,
    Tool(ToolArgs),
}

#[derive(Args, Debug, PartialEq)]
struct RunArgs {
    /// Redeclares the root's `-C`: below `run`, this one binds.
    #[arg(short = 'C')]
    cores: Option<u32>,
    #[arg(positional)]
    task: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
struct ToolArgs {
    #[arg(subcommand)]
    command: Option<ToolCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum ToolCommand {
    Install(InstallArgs),
}

#[derive(Args, Debug, PartialEq)]
struct InstallArgs {
    #[arg(short)]
    force: bool,
}

fn parse(line: &[&str]) -> Result<Root, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    Root::parse_from(&words)
}

fn ok(line: &[&str]) -> Root {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn run(cores: Option<u32>, task: &[&str]) -> Option<Command> {
    Some(Command::Run(RunArgs {
        cores,
        task: task.iter().map(|t| t.to_string()).collect(),
    }))
}

#[test]
fn a_global_binds_on_either_side_of_the_subcommand() {
    let root = ok(&["-v", "run", "-v", "x", "--verbose"]);
    assert_eq!((root.verbose, root.command), (3, run(None, &["x"])));
    assert_eq!(ok(&["run", "--cd=/a"]).cd.as_deref(), Some("/a"));
    assert_eq!(ok(&["ls", "-vv"]).verbose, 2);
}

#[test]
fn a_redeclared_name_binds_to_the_subcommand() {
    let root = ok(&["run", "-C", "3"]);
    assert_eq!((root.cd, root.command), (None, run(Some(3), &[])));
    // The long name is not redeclared, so it still reaches the root.
    assert_eq!(ok(&["run", "--cd", "/a"]).cd.as_deref(), Some("/a"));
}

#[test]
fn globals_reach_any_depth_even_inside_a_bundle() {
    let root = ok(&["tool", "install", "-vfC", "/d"]);
    assert_eq!((root.verbose, root.cd.as_deref()), (1, Some("/d")));
    assert_eq!(
        root.command,
        Some(Command::Tool(ToolArgs {
            command: Some(ToolCommand::Install(InstallArgs { force: true })),
        }))
    );
}

#[test]
fn only_globals_are_inherited() {
    let e = parse(&["run", "-l"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-l")));
    assert!(ok(&["-l", "run"]).local);
    let e = parse(&["ls", "x"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnexpectedArg, Some("x")));
    // After `--` a global's spelling is just a word.
    let root = ok(&["run", "--", "-v"]);
    assert_eq!((root.verbose, root.command), (0, run(None, &["-v"])));
}
