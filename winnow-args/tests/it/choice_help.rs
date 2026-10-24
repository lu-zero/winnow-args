//! What each choice means, choices left out of help, help from a constant,
//! and a `--help` that prints the short page.

use winnow_args::help::{Item, render};
use winnow_args::{Args, ErrorKind, FromArg, ValueEnum};

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Speed {
    /// As fast as it goes.
    Fast,
    Slow,
    #[arg(hide)]
    Secret,
}

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Plain {
    Up,
    Down,
}

/// The help `--level` shows, made where help can be.
const LEVEL: &str = if cfg!(target_pointer_width = "64") {
    "how loud (wide)"
} else {
    "how loud (narrow)"
};

#[derive(Args, Debug, PartialEq)]
#[arg(name = "tool")]
struct Tool {
    /// How fast.
    #[arg(long)]
    speed: Option<Speed>,
    /// Which way.
    #[arg(long)]
    way: Option<Plain>,
    /// What to sort by.
    #[arg(long, choices("name" = "The file's name", "size", "time" = "When it changed"))]
    sort: Option<String>,
    /// Where it logs.
    #[arg(long, choices("stdout", "stderr"), hide_choices)]
    log: Option<String>,
    /// How loud.
    #[arg(long, help = LEVEL)]
    level: Option<u8>,
}

/// What help says each of an item's choices means.
fn helps(item: &Item) -> Vec<&'static str> {
    let help = item.choice_help.expect("a choice says something");
    (0..item.choices.len()).map(help).collect()
}

fn item(long: &str) -> &'static Item {
    let items = Tool::HELP.items.iter();
    items.into_iter().find(|i| i.long == Some(long)).unwrap()
}

#[test]
fn a_value_enum_says_what_each_visible_variant_means() {
    assert_eq!(Speed::CHOICES, ["fast", "slow"]);
    const { assert!(Speed::DESCRIBES_CHOICES) };
    assert_eq!(Speed::choice_help(0, ""), "As fast as it goes.");
    // The field's text, when it writes one, wins.
    assert_eq!(Speed::choice_help(0, "written"), "written");
    assert_eq!(Speed::choice_help(1, ""), "");
    assert_eq!(helps(item("speed")), ["As fast as it goes.", ""]);
    // An enum whose variants say nothing costs nothing.
    const { assert!(!Plain::DESCRIBES_CHOICES) };
    assert!(item("way").choice_help.is_none());
}

#[test]
fn choices_written_on_the_field_say_what_they_mean() {
    let sort = item("sort");
    assert_eq!(sort.choices, ["name", "size", "time"]);
    assert_eq!(helps(sort), ["The file's name", "", "When it changed"]);
    assert!(item("log").choice_help.is_none() && item("log").hide_choices);
}

#[test]
fn the_long_page_lists_each_choice_on_a_line() {
    let long = render(Tool::HELP, &["tool"], true);
    let list = "Possible values:\n          - name: The file's name\n          - size\n          \
                - time: When it changed";
    assert!(long.contains(list), "{long}");
    assert!(
        long.contains("- fast: As fast as it goes.\n          - slow\n"),
        "{long}"
    );
    // Choices that say nothing stay on one line.
    assert!(long.contains("[possible values: up, down]"), "{long}");
    // The short page keeps them on one line.
    let short = render(Tool::HELP, &["tool"], false);
    assert!(
        short.contains("[possible values: name, size, time]"),
        "{short}"
    );
    assert!(!short.contains("Possible values:"), "{short}");
}

/// A type whose choices' help only the running program knows, as a crate's
/// enum without winnow-args keeps it.
struct Mode(usize);

impl FromArg for Mode {
    const CHOICES: &'static [&'static str] = &["one", "two"];
    const DESCRIBES_CHOICES: bool = true;

    fn choice_help(index: usize, default: &'static str) -> &'static str {
        if index == 1 {
            "the second, said at run time"
        } else {
            default
        }
    }

    fn from_arg(value: &winnow_args::BStr) -> Result<Self, winnow_args::error::BoxError> {
        Ok(Mode(usize::from(&**value == b"two")))
    }
}

#[derive(Args, Debug)]
struct Modes {
    /// Which one.
    #[arg(long, choices("one" = "the first, as written", "two" = "the second, as written"))]
    mode: Option<Mode>,
    #[arg(long)]
    plain: Option<Mode>,
}

impl std::fmt::Debug for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[test]
fn the_type_answers_and_the_written_text_is_its_default() {
    let mode = |long| {
        Modes::HELP
            .items
            .iter()
            .find(|i| i.long == Some(long))
            .unwrap()
    };
    assert_eq!(
        helps(mode("mode")),
        ["the first, as written", "the second, said at run time"]
    );
    assert_eq!(helps(mode("plain")), ["", "the second, said at run time"]);
    let _ = Modes::parse_from(["--mode", "two"])
        .unwrap()
        .mode
        .map(|m| m.0);
}

#[test]
fn hidden_choices_are_accepted_but_not_shown() {
    for long in [false, true] {
        let page = render(Tool::HELP, &["tool"], long);
        assert!(!page.contains("stdout"), "{page}");
    }
    let tool = Tool::parse_from(["--log", "stderr"]).unwrap();
    assert_eq!(tool.log.as_deref(), Some("stderr"));
    let e = Tool::parse_from(["--log", "file"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidChoice);
}

#[test]
fn help_may_be_a_constant() {
    assert_eq!(item("level").help, LEVEL);
    let page = render(Tool::HELP, &["tool"], false);
    assert!(page.contains(LEVEL), "{page}");
}

/// Short.
///
/// Long.
#[derive(Args, Debug)]
#[arg(brief_help)]
struct Brief {
    /// A flag.
    ///
    /// What it does, at length.
    #[arg(short)]
    verbose: bool,
}

#[test]
fn brief_help_makes_help_the_short_page() {
    let e = Brief::parse_from(["--help"]).unwrap_err();
    let page = e.render_help("brief").unwrap();
    assert!(
        !page.contains("at length") && !page.contains("Long."),
        "{page}"
    );
    assert!(page.contains("-h, --help  Print help\n"), "{page}");
    assert!(Brief::HELP.brief_help);
    let e = Brief::parse_from(["-h"]).unwrap_err();
    assert_eq!(e.render_help("brief").unwrap(), page);
}

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Phase {
    /// Alpha text.
    Alpha,
    #[arg(hide)]
    Hidden,
    /// Gamma text.
    Gamma,
    /// Delta text.
    Delta,
}

#[derive(Args, Debug)]
struct Subset {
    /// Two of the modes, in another order.
    #[arg(long, choices("delta", "alpha"))]
    some: Option<Phase>,
    /// One, with the field's own words.
    #[arg(long, choices("gamma" = "mine", "delta"))]
    mine: Vec<Phase>,
}

#[test]
fn declared_choices_ask_the_type_about_each_by_name() {
    let item = |long| {
        Subset::HELP
            .items
            .iter()
            .find(|i| i.long == Some(long))
            .unwrap()
    };
    assert_eq!(helps(item("some")), ["Delta text.", "Alpha text."]);
    assert_eq!(helps(item("mine")), ["mine", "Delta text."]);
}

#[derive(winnow_args::Subcommand, Debug)]
enum Briefs {
    Short(Brief),
}

#[derive(Args, Debug)]
struct Above {
    #[arg(subcommand)]
    command: Briefs,
}

#[test]
fn help_for_a_brief_command_is_short_however_asked() {
    for line in [&["help", "short"][..], &["short", "--help"]] {
        let e = Above::parse_from(line).unwrap_err();
        let page = e.render_help("above").unwrap();
        assert!(!page.contains("at length"), "{line:?}: {page}");
    }
    // Not inherited: the command above has its long page.
    let e = Above::parse_from(["--help"]).unwrap_err();
    assert!(e.render_help("above").unwrap().contains("see a summary"));
}
