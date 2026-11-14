//! `choices_strict = false`: choices that are suggestions.

use winnow_args::help::{Item, render};
use winnow_args::{Args, ErrorKind};

#[derive(Args, Debug, PartialEq)]
#[arg(name = "tool")]
struct Tool {
    /// Where to log.
    #[arg(long, choices("stdout", "stderr"), choices_strict = false)]
    log: Option<String>,
    /// Which way.
    #[arg(long, choices("up", "down"))]
    way: Option<String>,
    /// Targets.
    #[arg(positional, choices("a", "b"), choices_strict = false)]
    targets: Vec<String>,
}

fn parse(words: &[&str]) -> Result<Tool, winnow_args::Error> {
    Tool::parse_from(words.iter().copied())
}

fn item(long: &str) -> &'static Item {
    Tool::HELP
        .items
        .iter()
        .find(|i| i.long == Some(long))
        .unwrap()
}

#[test]
fn a_suggestion_does_not_limit_the_value() {
    let tool = parse(&["--log", "/var/log/x", "c"]).unwrap();
    assert_eq!(tool.log.as_deref(), Some("/var/log/x"));
    assert_eq!(tool.targets, ["c"]);
}

#[test]
fn a_list_without_it_still_limits() {
    let error = parse(&["--way", "left"]).unwrap_err();
    assert!(
        matches!(error.kind(), ErrorKind::InvalidChoice),
        "{error:?}"
    );
}

#[test]
fn suggestions_are_still_listed() {
    assert!(item("log").choices_open && !item("way").choices_open);
    assert_eq!(item("log").choices, ["stdout", "stderr"]);
    let help = render(Tool::HELP, &["tool"], false);
    assert!(
        help.contains("[suggested values: stdout, stderr]"),
        "{help}"
    );
    assert!(help.contains("[possible values: up, down]"), "{help}");
}

#[derive(winnow_args::ValueEnum, Clone)]
enum Speed {
    Fast,
}

#[derive(Args)]
struct Generic<T: winnow_args::FromArg + Send + Sync + 'static> {
    #[arg(long, choices("x"), choices_strict = false)]
    name: Option<String>,
    #[arg(long)]
    other: Option<T>,
}

#[test]
fn a_generic_struct_still_checks_a_type_without_the_parameter() {
    let _ = Generic::<String>::HELP;
    let _ = Speed::Fast;
}

#[test]
fn the_long_page_says_suggested() {
    let long = render(Tool::HELP, &["tool"], true);
    assert!(
        long.contains("[suggested values: stdout, stderr]"),
        "{long}"
    );
}

#[derive(Args)]
struct Out {
    #[arg(long, choices("-"), choices_strict = false, hint = "file")]
    out: Option<String>,
}

#[test]
fn completion_offers_the_hint_beside_the_suggestions() {
    use winnow_args::complete::{Hint, complete};
    let done = complete(Out::HELP, &["--out", ""]);
    assert_eq!(done.candidates.len(), 1);
    assert_eq!(done.hint, Hint::Files);
}
