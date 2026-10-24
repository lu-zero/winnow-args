//! `#[arg(external)]`: a word that names no subcommand is one all the same,
//! and takes the rest of the line as typed.

use std::ffi::OsString;

use winnow_args::{Args, ErrorKind, Subcommand};

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Build,
    #[arg(external)]
    Other(Vec<String>),
}

#[derive(Args, Debug, PartialEq)]
struct Cargo {
    #[arg(short, long, global)]
    quiet: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Args, Debug, PartialEq)]
struct WithPositional {
    #[arg(positional)]
    target: Option<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn a_known_name_is_its_variant() {
    let cargo = Cargo::parse_from(["build"]).unwrap();
    assert_eq!(cargo.command, Some(Command::Build));
}

#[test]
fn any_other_word_takes_the_rest_as_typed() {
    let cargo = Cargo::parse_from(["foo", "--bar", "-q", "--", "x"]).unwrap();
    let words = strings(&["foo", "--bar", "-q", "--", "x"]);
    assert_eq!(
        (cargo.quiet, cargo.command),
        (false, Some(Command::Other(words)))
    );
    // A flag before the name is still the parent's.
    let cargo = Cargo::parse_from(["-q", "foo"]).unwrap();
    assert_eq!(
        (cargo.quiet, cargo.command),
        (true, Some(Command::Other(strings(&["foo"]))))
    );
    assert_eq!(Cargo::parse_from([""; 0]).unwrap().command, None);
}

#[test]
fn a_positional_takes_its_word_first() {
    let line = WithPositional::parse_from(["a", "foo", "b"]).unwrap();
    assert_eq!(line.target.as_deref(), Some("a"));
    assert_eq!(line.command, Some(Command::Other(strings(&["foo", "b"]))));
    // A known name is still routed before any positional.
    let line = WithPositional::parse_from(["build"]).unwrap();
    assert_eq!((line.target, line.command), (None, Some(Command::Build)));
}

#[test]
fn not_a_known_name_after_a_positional() {
    // Routing stops once a positional is filled: `build` is not routed then.
    let e = WithPositional::parse_from(["a", "build"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("build"))
    );
}

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value")]
struct Lenient {
    #[arg(short)]
    verbose: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[test]
fn a_lenient_unknown_flag_is_not_a_name() {
    for line in [&["--foo", "x"][..], &["-xyz"], &["-k"]] {
        let e = Lenient::parse_from(line).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::UnexpectedArg, "{line:?}");
    }
    let lenient = Lenient::parse_from(["-v", "foo", "--bar"]).unwrap();
    let words = strings(&["foo", "--bar"]);
    assert_eq!(lenient.command, Some(Command::Other(words)));
}

#[test]
fn not_after_the_separator() {
    let e = Cargo::parse_from(["--", "foo"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("foo"))
    );
}

#[test]
fn an_unknown_flag_is_not_a_name() {
    let e = Cargo::parse_from(["--foo"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnknownFlag);
}

#[test]
fn the_enum_as_the_whole_line() {
    let command = Command::parse_from(["foo", "-x"]).unwrap();
    assert_eq!(command, Command::Other(strings(&["foo", "-x"])));
    assert_eq!(Command::parse_from(["build"]).unwrap(), Command::Build);
}

#[derive(Subcommand, Debug, PartialEq)]
enum Raw {
    Build,
    #[arg(external)]
    Other(Vec<OsString>),
}

#[derive(Subcommand, Debug, PartialEq)]
enum Numbered {
    Build,
    #[arg(external)]
    Other(Vec<u32>),
}

#[test]
fn the_words_convert() {
    let raw = Raw::parse_from(["foo", "x"]).unwrap();
    assert_eq!(raw, Raw::Other(vec!["foo".into(), "x".into()]));
    assert_eq!(
        Numbered::parse_from(["1", "2"]).unwrap(),
        Numbered::Other(vec![1, 2])
    );
    let e = Numbered::parse_from(["1", "x"]).unwrap_err();
    assert_eq!((e.kind(), e.offset()), (ErrorKind::InvalidValue, 2));
    let e = Numbered::parse_from(["x"]).unwrap_err();
    assert_eq!((e.kind(), e.offset()), (ErrorKind::InvalidValue, 0));
}

#[derive(Subcommand, Debug, PartialEq)]
enum OnlyExternal {
    #[arg(external)]
    Other(Vec<String>),
}

#[test]
fn an_enum_of_it_alone() {
    let only = OnlyExternal::parse_from(["foo", "-x"]).unwrap();
    assert_eq!(only, OnlyExternal::Other(strings(&["foo", "-x"])));
}

#[test]
fn help_does_not_list_it() {
    let names: Vec<_> = Cargo::HELP.subcommands.iter().map(|s| s.name).collect();
    assert_eq!(names, ["build"]);
}

/// What a completion callback answers, for `line` typed up to the cursor.
fn complete<T: Args>(line: &str) -> String {
    let args: Vec<OsString> = ["__complete_word__", "--shell", "fish", "--line", line]
        .into_iter()
        .map(OsString::from)
        .collect();
    T::completion_request(&args).unwrap()
}

#[test]
fn completion_leaves_its_words_alone() {
    assert!(complete::<Cargo>("cargo ").contains("build"));
    assert!(complete::<Cargo>("cargo -").contains("--quiet"));
    assert_eq!(complete::<Cargo>("cargo foo "), "");
    assert_eq!(complete::<Cargo>("cargo foo -"), "");
    assert_eq!(complete::<Cargo>("cargo -q foo --bar "), "");
    // A positional takes the first word.
    assert!(complete::<WithPositional>("w a ").contains("build"));
    assert_eq!(complete::<WithPositional>("w a foo "), "");
}
