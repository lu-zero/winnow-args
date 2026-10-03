//! The help attributes of a struct and its subcommands, and what switches
//! the built-in flags and the `help` word off.

use winnow_args::{Args, Error, ErrorKind, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::try_parse_from(line)
}

fn help<T: Args + std::fmt::Debug>(line: &[&str]) -> String {
    let error = parse::<T>(line).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::HelpRequested, "{line:?}");
    error.render_help("tool").unwrap()
}

/// From the doc comment, unused.
#[derive(Args, Debug)]
#[arg(
    version = "1.2",
    about = "Short about.",
    long_about = "Long about.",
    after_help = "After, short.",
    after_long_help = "After, long."
)]
struct Texts {
    /// Short help.
    #[arg(long, long_help = "Long help.")]
    flag: bool,
    #[arg(long, values = 2, value_name = "V")]
    pair: Vec<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    #[arg(alias_hidden = "l")]
    List,
}

#[test]
fn short_and_long_help_take_their_own_texts() {
    let (short, long) = (help::<Texts>(&["-h"]), help::<Texts>(&["--help"]));
    assert!(short.starts_with("Short about.\n"), "{short}");
    assert!(long.starts_with("Long about.\n"), "{long}");
    assert!(
        short.contains("Short help.") && !short.contains("Long help."),
        "{short}"
    );
    assert!(
        long.contains("Long help.") && !long.contains("Short help."),
        "{long}"
    );
    assert!(short.ends_with("After, short.\n") && long.ends_with("After, long.\n"));
    assert!(!long.contains("From the doc comment"), "{long}");
}

#[test]
fn a_hidden_alias_names_the_command_but_is_not_listed() {
    assert_eq!(parse::<Texts>(&["l"]).unwrap().command, Some(Command::List));
    assert!(help::<Texts>(&["help", "l"]).starts_with("Usage: tool list"));
    let commands: Vec<String> = help::<Texts>(&["-h"])
        .lines()
        .skip_while(|line| *line != "Commands:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| line.trim().to_owned())
        .collect();
    assert_eq!(commands, ["list"]);
}

#[test]
fn values_takes_that_many_words_each_time() {
    let texts: Texts = parse(&["--pair", "a", "-b", "--pair", "c", "d"]).unwrap();
    assert_eq!(texts.pair, ["a", "-b", "c", "d"]);
    assert_eq!(
        parse::<Texts>(&["--pair", "a"]).unwrap_err().kind(),
        ErrorKind::MissingValue
    );
}

#[derive(Args, Debug)]
#[arg(version = "1.2", disable_version_flag, disable_help_subcommand)]
struct Bare {
    #[arg(subcommand)]
    command: Option<Command>,
}

#[test]
fn disabled_built_ins_are_ordinary_input() {
    assert_eq!(
        parse::<Texts>(&["-V"]).unwrap_err().kind(),
        ErrorKind::VersionRequested
    );
    for flag in ["-V", "--version"] {
        assert_eq!(
            parse::<Bare>(&[flag]).unwrap_err().kind(),
            ErrorKind::UnknownFlag
        );
    }
    for line in [&["help"][..], &["help", "list"]] {
        assert_eq!(
            parse::<Bare>(line).unwrap_err().kind(),
            ErrorKind::UnexpectedArg
        );
    }
    // The commands themselves are still listed.
    assert!(help::<Bare>(&["-h"]).contains("Commands:\n  list\n"));
}
