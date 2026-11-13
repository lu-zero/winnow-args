//! How a command's own arguments relate to its subcommands.

use winnow_args::{Args, ErrorKind, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, winnow_args::Error> {
    let words = crate::words(line);
    T::parse_words(&words)
}

#[derive(Subcommand, Debug, PartialEq)]
enum Sub {
    Inspect,
    Run {
        #[arg(long)]
        fast: bool,
    },
}

#[derive(Args, Debug)]
#[arg(subcommand_negates_reqs)]
struct Negates {
    #[arg(positional, required)]
    input: Option<String>,
    #[arg(long, required)]
    mode: Option<String>,
    #[arg(long, requires = "--mode")]
    extra: bool,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[test]
fn a_subcommand_leaves_the_requirements_of_its_parent() {
    let n = parse::<Negates>(&["inspect"]).unwrap();
    assert_eq!((n.input, n.mode, n.sub), (None, None, Some(Sub::Inspect)));
    // Without one they are required, as ever.
    let e = parse::<Negates>(&[]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingArgument);
    let ok = parse::<Negates>(&["--mode", "m", "file"]).unwrap();
    assert_eq!(ok.input.as_deref(), Some("file"));
    // What a given argument needs is still needed.
    let e = parse::<Negates>(&["--extra", "inspect"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
}

#[derive(Args, Debug)]
struct Plain {
    #[arg(long, required)]
    mode: Option<String>,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[test]
fn without_the_setting_a_subcommand_does_not_help() {
    let e = parse::<Plain>(&["inspect"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
}

#[derive(Args, Debug)]
#[arg(args_conflicts_with_subcommands)]
struct Conflicts {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, bool_value)]
    flag: bool,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, default = "d")]
    level: Option<String>,
    #[arg(long, global)]
    log: Option<String>,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[test]
fn a_parent_argument_excludes_a_subcommand() {
    let e = parse::<Conflicts>(&["--verbose", "inspect"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
    assert_eq!((e.token(), e.value()), (Some("inspect"), Some("--verbose")));
    // A value that says nothing about being given still counts.
    let e = parse::<Conflicts>(&["--flag=false", "inspect"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
    let e = parse::<Conflicts>(&["--name", "x", "run"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
}

#[test]
fn a_subcommand_alone_or_with_its_own_arguments_is_fine() {
    assert_eq!(
        parse::<Conflicts>(&["inspect"]).unwrap().sub,
        Some(Sub::Inspect)
    );
    assert_eq!(
        parse::<Conflicts>(&["run", "--fast"]).unwrap().sub,
        Some(Sub::Run { fast: true })
    );
    // A default is not an argument given, and a global flag may come first.
    let c = parse::<Conflicts>(&["--log", "x", "inspect"]).unwrap();
    assert_eq!(c.log.as_deref(), Some("x"));
    assert_eq!(c.level.as_deref(), Some("d"));
    // Without a subcommand the arguments are the command's own.
    assert!(
        parse::<Conflicts>(&["--verbose", "--name", "x"])
            .unwrap()
            .verbose
    );
}

#[derive(Subcommand, Debug, PartialEq)]
enum Verbs {
    Build,
}

#[derive(Args, Debug)]
#[arg(subcommand_precedence_over_arg)]
struct Precedence {
    #[arg(long, values = 1..)]
    files: Vec<String>,
    #[arg(positional)]
    rest: Vec<String>,
    #[arg(subcommand)]
    verb: Option<Verbs>,
}

#[derive(Args, Debug)]
struct NoPrecedence {
    #[arg(long, values = 1..)]
    files: Vec<String>,
    #[arg(positional)]
    rest: Vec<String>,
    #[arg(subcommand)]
    verb: Option<Verbs>,
}

#[test]
fn a_subcommand_name_ends_what_would_swallow_it() {
    let p = parse::<Precedence>(&["--files", "a", "b", "build"]).unwrap();
    assert_eq!(
        (p.files.as_slice(), p.verb),
        (
            ["a".to_owned(), "b".to_owned()].as_slice(),
            Some(Verbs::Build)
        )
    );
    let p = parse::<Precedence>(&["x", "y", "build"]).unwrap();
    assert_eq!(
        (p.rest.as_slice(), p.verb),
        (
            ["x".to_owned(), "y".to_owned()].as_slice(),
            Some(Verbs::Build)
        )
    );
    // A word that names none is a value, as ever.
    let p = parse::<Precedence>(&["--files", "a", "builder"]).unwrap();
    assert_eq!(p.files, ["a", "builder"]);
}

#[test]
fn without_the_setting_the_name_is_a_value() {
    let p = parse::<NoPrecedence>(&["--files", "a", "build"]).unwrap();
    assert_eq!(p.files, ["a", "build"]);
    assert_eq!(p.verb, None);
    let p = parse::<NoPrecedence>(&["x", "build"]).unwrap();
    assert_eq!(p.rest, ["x", "build"]);
}

#[derive(Args, Debug)]
#[arg(
    args_conflicts_with_subcommands,
    default_subcommand = "inspect",
    default_subcommand_on_empty
)]
struct DefaultOnEmpty {
    #[arg(long)]
    verbose: bool,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[derive(Args, Debug)]
#[arg(
    args_conflicts_with_subcommands,
    default_subcommand = "run",
    default_subcommand_flags
)]
struct DefaultForFlags {
    #[arg(long)]
    verbose: bool,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[test]
fn a_default_subcommand_taken_for_the_line_or_a_flag_conflicts_too() {
    let e = parse::<DefaultOnEmpty>(&["--verbose"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
    assert_eq!(
        parse::<DefaultOnEmpty>(&[]).unwrap().sub,
        Some(Sub::Inspect)
    );
    let e = parse::<DefaultForFlags>(&["--verbose", "--fast"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
    let ok = parse::<DefaultForFlags>(&["--fast"]).unwrap();
    assert_eq!(ok.sub, Some(Sub::Run { fast: true }));
}

#[derive(Args, Debug)]
#[arg(args_conflicts_with_subcommands)]
struct Remembered {
    #[arg(long, override_self = "first")]
    name: Option<String>,
    #[arg(subcommand)]
    sub: Option<Sub>,
}

#[test]
fn a_flag_kept_by_override_self_still_conflicts() {
    let e = parse::<Remembered>(&["--name", "a", "inspect"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Conflict);
    assert_eq!(
        parse::<Remembered>(&["inspect"]).unwrap().sub,
        Some(Sub::Inspect)
    );
}
