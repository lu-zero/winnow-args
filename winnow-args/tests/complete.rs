//! Shell completion answered from the help data.
#![cfg(feature = "derive")]

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
#[expect(dead_code, reason = "only the help data is completed")]
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
#[expect(dead_code, reason = "only the help data is completed")]
enum Command {
    /// Build it.
    Build(Build),
    /// Run it.
    Run,
}

#[derive(Args, Debug)]
#[expect(dead_code, reason = "only the help data is completed")]
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
    assert_eq!(values(&["-"]).len(), 7, "{:?}", values(&["-"]));
    assert!(!values(&["--"]).contains(&"--secret".to_owned()));
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
