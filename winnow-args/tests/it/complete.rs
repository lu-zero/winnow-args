//! Shell completion answered from the help data.

use std::ffi::OsString;
use std::path::PathBuf;

use winnow_args::complete::{Candidate, Hint, Shell, complete};
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
    output: Option<PathBuf>,
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
    assert!(files.hint == Hint::Files && files.candidates.is_empty());
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
    output: Option<PathBuf>,
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
    assert_eq!(more(&["--out", ""]).hint, Hint::Files);
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

    // A second short is listed, and one that takes a value holds the next word.
    assert!(forms(&["-"]).contains(&"-L".to_owned()));
    assert_eq!(walk(&["-L", ""]).hint, Hint::Files);
    // `+e` is a switch, so the next word is still a flag.
    assert_eq!(forms(&["+"]), ["+e"]);
    assert!(forms(&["+e", "--"]).contains(&"--verbose".to_owned()));
    // `values = 3`: three words, and after the first they are values even when flag-like.
    assert_eq!(walk(&["--platform", "a", "-11", ""]).hint, Hint::Files);
    assert!(forms(&["--platform", "a", "b", "c", "-"]).contains(&"-v".to_owned()));
    // `values = 1..` stops at a flag. A following flag is not an optional value.
    assert_eq!(walk(&["--include", "a", "b", ""]).hint, Hint::Files);
    assert!(forms(&["--include", "a", "--"]).contains(&"--verbose".to_owned()));
    assert_eq!(walk(&["--include", "a", "--"]).hint, Hint::Nothing);
    assert!(forms(&["--color", "--v"]).contains(&"--verbose".to_owned()));
    assert_eq!(walk(&["--color", ""]).hint, Hint::Files);
}

/// Second shorts, `+` options, fixed and open value runs, an optional value.
#[derive(Args, Debug)]
#[arg(name = "forms", plus_options, disable_help_flag)]
#[allow(dead_code, reason = "only the help data is completed")]
struct Forms {
    #[arg(short = 'l', short = 'L', value_name = "FILE")]
    list: Option<PathBuf>,
    #[arg(short = 'v', long)]
    verbose: bool,
    #[arg(short = 'e', plus = 'e')]
    errexit: Option<bool>,
    #[arg(long, values = 3, value_name = "V")]
    platform: Vec<PathBuf>,
    #[arg(long, values = 1.., value_name = "PATH")]
    include: Vec<PathBuf>,
    #[arg(long, default_missing = "always", hint = "file")]
    color: Option<String>,
}

fn forms(words: &[&str]) -> Vec<String> {
    complete(Forms::HELP, words)
        .candidates
        .into_iter()
        .map(|c| c.value)
        .collect()
}

fn walk(words: &[&str]) -> winnow_args::complete::Completions {
    complete(Forms::HELP, words)
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

/// The branches a checkout may name, whatever is typed of one.
fn branches(_typed: &str) -> Vec<Candidate> {
    ["main", "maint", "next"]
        .into_iter()
        .map(|value| Candidate {
            value: value.to_owned(),
            help: "a branch",
        })
        .collect()
}

#[derive(Args, Debug)]
#[arg(name = "hinted")]
struct Hinted {
    /// Where to work.
    #[arg(short = 'C', long, hint = "dir")]
    dir: Option<String>,
    /// What to run after.
    #[arg(long, hint = "command")]
    then: Option<String>,
    /// How many at once.
    #[arg(short, long, hint = "none")]
    jobs: Option<u32>,
    /// The branch to start from.
    #[arg(long, complete = branches)]
    from: Option<String>,
    /// Unhinted: each type says what it is.
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    retries: Vec<u8>,
    #[arg(long)]
    bind: Option<std::net::SocketAddr>,
    #[arg(long)]
    limit: Option<winnow_args::value::CInt<u64>>,
    /// A number is no directory, unless the field says so.
    #[arg(long, hint = "dir")]
    depth: Option<u32>,
    /// A path is no file, when the field says so.
    #[arg(long, hint = "none")]
    opaque: Option<PathBuf>,
    /// A type of the program says what it is.
    #[arg(long)]
    tree: Vec<Tree>,
    #[arg(positional, complete = branches)]
    branch: Option<String>,
    #[arg(positional, hint = "command")]
    run: Option<String>,
}

#[derive(Debug)]
struct Tree;

impl winnow_args::FromArg for Tree {
    const HINT: Hint = Hint::Dirs;

    fn from_arg(_: &winnow_args::BStr) -> Result<Self, winnow_args::error::BoxError> {
        Ok(Tree)
    }
}

#[test]
fn a_value_is_what_its_hint_or_its_function_says() {
    let at = |words: &[&str]| complete(Hinted::HELP, words);
    assert_eq!(at(&["-C", ""]).hint, Hint::Dirs);
    assert_eq!(at(&["--dir=sr"]).hint, Hint::Dirs);
    assert_eq!(at(&["--then", ""]).hint, Hint::Commands);
    // Nothing a shell knows, and no candidate either.
    assert_eq!(at(&["-j", ""]), Default::default());
    // With no hint stated, the type's: a path or a word is a file, and a
    // number or an address nothing a shell knows.
    assert_eq!(at(&["--config", ""]).hint, Hint::Files);
    assert_eq!(at(&["--name", ""]).hint, Hint::Files);
    assert_eq!(at(&["--retries", ""]).hint, Hint::Nothing);
    assert_eq!(at(&["--bind", ""]).hint, Hint::Nothing);
    assert_eq!(at(&["--limit", ""]).hint, Hint::Nothing);
    assert_eq!(at(&["--depth", ""]).hint, Hint::Dirs);
    assert_eq!(at(&["--opaque", ""]).hint, Hint::Nothing);
    assert_eq!(at(&["--tree", ""]).hint, Hint::Dirs);
    // A positional too.
    assert_eq!(at(&["main", ""]).hint, Hint::Commands);
    let values = |words: &[&str]| -> Vec<String> {
        at(words).candidates.into_iter().map(|c| c.value).collect()
    };
    // The function answers; what does not start as typed is left out.
    assert_eq!(values(&["--from", ""]), ["main", "maint", "next"]);
    assert_eq!(values(&["--from", "ma"]), ["main", "maint"]);
    assert_eq!(values(&["--from=n"]), ["--from=next"]);
    assert_eq!(values(&["mai"]), ["main", "maint"]);
    assert_eq!(at(&["--from", ""]).hint, Hint::Nothing);
    let args = |words: &[&str]| -> Vec<OsString> { words.iter().map(OsString::from).collect() };
    let asked = |shell: &str, line: &str| {
        Hinted::completion_request(&args(&[
            "__complete_word__",
            "--shell",
            shell,
            "--line",
            line,
        ]))
    };
    assert_eq!(asked("bash", "hinted -C ").as_deref(), Some("\u{1}dirs\n"));
    assert_eq!(
        asked("zsh", "hinted --then ").as_deref(),
        Some("\u{1}commands\t\n")
    );
    // A value attached to its flag: the shell is told what comes before it,
    // but for bash, which breaks the word at the `=` itself.
    assert_eq!(at(&["--dir=sr"]).before, "--dir=");
    assert_eq!(at(&["-C", "sr"]).before, "");
    for shell in ["zsh", "fish", "elvish", "powershell"] {
        assert_eq!(
            asked(shell, "hinted --dir=sr").as_deref(),
            Some("\u{1}dirs\t--dir=\n"),
            "{shell}"
        );
    }
    assert_eq!(
        asked("bash", "hinted --dir=sr").as_deref(),
        Some("\u{1}dirs\n")
    );
    assert_eq!(asked("fish", "hinted -j ").as_deref(), Some(""));
    assert_eq!(
        asked("fish", "hinted --from n").as_deref(),
        Some("next\ta branch\n")
    );
    // Every script reads each line it may be answered with.
    for &shell in Shell::ALL {
        let script = Hinted::completion_script(shell);
        for line in ["files", "dirs", "commands"] {
            assert!(script.contains(line), "{shell}: {line}");
        }
    }
}

#[derive(Args, Debug)]
struct Aliased {
    /// Resume a session.
    #[arg(long, visible_alias = "session", alias = "old")]
    resume: Option<When>,
}

#[test]
fn completion_offers_a_visible_alias_and_its_values() {
    let names = |words: &[&str]| -> Vec<String> {
        complete(Aliased::HELP, words)
            .candidates
            .into_iter()
            .map(|c| c.value)
            .collect()
    };
    let offered = names(&["--"]);
    assert!(offered.contains(&"--resume".to_owned()), "{offered:?}");
    assert!(offered.contains(&"--session".to_owned()), "{offered:?}");
    assert!(!offered.contains(&"--old".to_owned()), "{offered:?}");
    assert_eq!(names(&["--session", ""]), ["auto", "always", "never"]);
}
