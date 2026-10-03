//! Shell completion answered from the help data.

use std::ffi::OsString;

use winnow_args::complete::{Shell, complete};
use winnow_args::{Args, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum When {
    Auto,
    Always,
    Never,
}

#[derive(Args, Debug)]
#[arg(name = "tool")]
struct Tool {
    /// Talk more.
    #[arg(short, long)]
    verbose: bool,
    /// When to color.
    #[arg(long)]
    color: Option<When>,
    /// Where to write.
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
    /// Hidden.
    #[arg(long, hide)]
    secret: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build it.
    Build(Build),
    /// Run it.
    Run,
}

#[derive(Args, Debug)]
struct Build {
    /// Release mode.
    #[arg(long)]
    release: bool,
    #[arg(positional)]
    targets: Vec<When>,
}

fn values(words: &[&str]) -> Vec<String> {
    complete(Tool::HELP, words)
        .candidates
        .into_iter()
        .map(|c| c.value)
        .collect()
}

#[test]
fn flags_after_a_dash() {
    assert_eq!(
        values(&["--"]),
        ["--verbose", "--color", "--output", "--help"]
    );
    assert_eq!(values(&["--co"]), ["--color"]);
    // Every visible spelling, the hidden `--secret` left out.
    assert_eq!(
        values(&["-"]),
        [
            "--verbose",
            "-v",
            "--color",
            "--output",
            "-o",
            "-h",
            "--help"
        ]
    );
}

#[test]
fn a_flags_value() {
    assert_eq!(values(&["--color", ""]), ["auto", "always", "never"]);
    assert_eq!(values(&["--color", "a"]), ["auto", "always"]);
    assert_eq!(values(&["--color=n"]), ["--color=never"]);
    let files = complete(Tool::HELP, &["-o", ""]);
    assert!(files.files && files.candidates.is_empty());
    // Attached, the value is already there.
    assert_eq!(values(&["-ofile", "b"]), ["build"]);
}

#[test]
fn subcommands_and_their_own_flags() {
    assert_eq!(values(&[""]), ["build", "run"]);
    assert_eq!(values(&["-v", "b"]), ["build"]);
    assert_eq!(values(&["build", "--"]), ["--release", "--help"]);
    assert_eq!(values(&["build", "a"]), ["auto", "always"]);
    assert_eq!(values(&["build", "auto", "n"]), ["never"]);
    // Past `--`, no flags.
    assert_eq!(values(&["build", "--", "-"]), Vec::<String>::new());
}

#[test]
fn the_callback_answers_per_shell() {
    let args = |words: &[&str]| -> Vec<OsString> { words.iter().map(OsString::from).collect() };
    assert_eq!(Tool::completion_request(&args(&["build"])), None);
    // Descriptions are help text, left out without the `help-text` feature.
    let (fish, zsh) = if cfg!(feature = "help-text") {
        ("--color\tWhen to color.\n", "--color:When to color.\n")
    } else {
        ("--color\n", "--color\n")
    };
    assert_eq!(
        Tool::completion_request(&args(&[
            "__complete_word__",
            "--shell",
            "fish",
            "--line",
            "tool --co"
        ]))
        .as_deref(),
        Some(fish)
    );
    assert_eq!(
        Tool::completion_request(&args(&[
            "__complete_word__",
            "--shell",
            "zsh",
            "--line",
            "tool --co"
        ]))
        .as_deref(),
        Some(zsh)
    );
    // bash hands over the word after its last `=` too.
    assert_eq!(
        Tool::completion_request(&args(&[
            "__complete_word__",
            "--shell",
            "bash",
            "--line",
            "tool --color=al",
            "--bash-word",
            "al",
        ]))
        .as_deref(),
        Some("always\n")
    );
    assert_eq!(
        Tool::completion_request(&args(&[
            "__complete_word__",
            "--shell",
            "bash",
            "--line",
            "tool -o ",
            "--bash-word",
            "",
        ]))
        .as_deref(),
        Some("\u{1}files\n")
    );
}

#[test]
fn scripts_name_the_program() {
    for &shell in Shell::ALL {
        assert!(Tool::completion_script(shell).contains("tool __complete_word__"));
    }
}

/// Aliases, `require_equals`, global flags and a declared `--help`.
#[derive(Args, Debug)]
#[arg(name = "more")]
#[allow(dead_code, reason = "only the help data is completed")]
struct More {
    #[arg(long, alias = "out", value_name = "FILE")]
    output: Option<String>,
    #[arg(long, require_equals)]
    color: Option<When>,
    #[arg(long, global)]
    global_flag: bool,
    /// Own help.
    #[arg(long)]
    help: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

fn more(words: &[&str]) -> winnow_args::complete::Completions {
    complete(More::HELP, words)
}

#[test]
fn the_walk_reads_the_line_as_the_parser_does() {
    // An alias takes its value from the next word, like the name it stands for.
    assert!(more(&["--out", ""]).files);
    // `require_equals`: the next word is not the value.
    let next = more(&["--color", ""]);
    assert!(
        !next.candidates.iter().any(|c| c.value == "auto"),
        "{next:?}"
    );
    assert_eq!(more(&["--color=a"]).candidates.len(), 2);
    // A global flag is offered below its command.
    let below: Vec<_> = more(&["build", "--g"])
        .candidates
        .into_iter()
        .map(|c| c.value)
        .collect();
    assert_eq!(below, ["--global-flag"]);
    // A declared `--help` is listed once.
    let help: Vec<_> = more(&["--he"])
        .candidates
        .into_iter()
        .map(|c| c.value)
        .collect();
    assert_eq!(help, ["--help"]);
}

#[test]
fn words_given_unquoted_are_not_split_again() {
    let args: Vec<OsString> = [
        "__complete_word__",
        "--shell",
        "elvish",
        "--words",
        "tool",
        "a b",
        "--co",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let answer = Tool::completion_request(&args).unwrap();
    assert!(answer.starts_with("--color"), "{answer}");
}

#[test]
fn a_shell_is_a_flag_value() {
    #[derive(winnow_args::Args)]
    struct Cli {
        #[arg(long)]
        completions: Option<Shell>,
    }
    let shell = |line: &[&str]| Cli::parse_from(line).map(|c| c.completions);
    assert_eq!(shell(&["--completions", "zsh"]).unwrap(), Some(Shell::Zsh));
    let e = shell(&["--completions", "tcsh"]).unwrap_err();
    assert_eq!(e.kind(), winnow_args::ErrorKind::InvalidChoice);
}
