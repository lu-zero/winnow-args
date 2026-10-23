//! What an unknown flag, subcommand or value was probably meant to be.

use winnow_args::help::Style;
use winnow_args::{Args, ErrorKind, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum When {
    Auto,
    Always,
    Never,
}

#[derive(Args, Debug)]
#[arg(name = "tool", version = "1")]
struct Tool {
    /// Over what is there.
    #[arg(short, long, negate)]
    force: bool,
    /// Talk more, in every command.
    #[arg(short, long, global)]
    verbose: bool,
    #[arg(long, hide)]
    secret: bool,
    #[arg(long, global, hide)]
    trace: bool,
    #[arg(long)]
    color: Option<When>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build it.
    #[arg(alias = "make")]
    Build(Build),
    /// The places it is kept.
    Remote(Remote),
    #[arg(hide, alias = "plumbing")]
    Internal,
}

#[derive(Args, Debug)]
struct Remote {
    #[arg(long, global)]
    quiet: bool,
    #[arg(subcommand)]
    command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
enum RemoteCommand {
    Add(Add),
    Prune,
}

#[derive(Args, Debug)]
struct Add {
    #[arg(long)]
    mirror: bool,
}

#[derive(Args, Debug)]
struct Build {
    #[arg(long)]
    release: bool,
    #[arg(long)]
    target: Option<String>,
}

fn similar(line: &[&str]) -> Option<String> {
    let error = Tool::parse_from(line).unwrap_err();
    error.similar().map(str::to_owned)
}

#[test]
fn an_unknown_flag_is_close_to_one_that_is_shown() {
    assert_eq!(similar(&["--fore"]).as_deref(), Some("--force"));
    assert_eq!(similar(&["--no-forc"]).as_deref(), Some("--no-force"));
    assert_eq!(similar(&["--versoin"]).as_deref(), Some("--version"));
    assert_eq!(similar(&["--hepl"]).as_deref(), Some("--help"));
    // With a value attached, the name alone is compared.
    assert_eq!(similar(&["--colour=auto"]).as_deref(), Some("--color"));
    // A hidden flag is not given away, and nothing is close to this one.
    assert_eq!(similar(&["--secre"]), None);
    assert_eq!(similar(&["--zzz"]), None);
    // A letter is not a misspelt word.
    assert_eq!(similar(&["-x"]), None);
}

#[test]
fn a_subcommand_s_flags_are_its_own_and_the_globals_above() {
    assert_eq!(
        similar(&["build", "--relase"]).as_deref(),
        Some("--release")
    );
    assert_eq!(
        similar(&["build", "--verbos"]).as_deref(),
        Some("--verbose")
    );
    // `--force` is the root's alone.
    assert_eq!(similar(&["build", "--fore"]), None);
    // Two commands down: its own, and the globals of each command above.
    let add = |flag| similar(&["remote", "add", flag]);
    assert_eq!(add("--miror").as_deref(), Some("--mirror"));
    assert_eq!(add("--quite").as_deref(), Some("--quiet"));
    assert_eq!(add("--verbos").as_deref(), Some("--verbose"));
    assert_eq!(add("--relase"), None);
    // A hidden global is not given away below either.
    assert_eq!(add("--trac"), None);
}

#[test]
fn an_unknown_word_is_close_to_a_subcommand() {
    assert_eq!(similar(&["biuld"]).as_deref(), Some("build"));
    assert_eq!(similar(&["mak"]).as_deref(), Some("make"));
    assert_eq!(similar(&["interna"]), None);
    // Nor is a hidden subcommand given away by its other name.
    assert_eq!(similar(&["plumbin"]), None);
    // Among those of the command it was given to.
    assert_eq!(similar(&["remote", "prun"]).as_deref(), Some("prune"));
    assert_eq!(similar(&["remote", "biuld"]), None);
}

#[test]
fn a_value_is_close_to_a_choice() {
    let error = Tool::parse_from(["--color", "alwyas"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidChoice);
    assert_eq!(error.similar(), Some("always"));
    assert_eq!(similar(&["--color", "blue"]), None);
    assert!(
        error
            .render(Style::PLAIN)
            .contains("tip: a similar value exists: `always`"),
        "{error:?}"
    );
}

#[test]
fn the_report_has_the_tip_and_the_message_does_not() {
    let error = Tool::parse_from(["--fore"]).unwrap_err();
    assert_eq!(error.to_string(), "unknown flag `--fore`");
    assert_eq!(
        error.render(Style::PLAIN),
        "error: unknown flag `--fore`\n\n  tip: a similar flag exists: `--force`\n\n\
         For more information, try '--help'."
    );
    let colored = error.render(Style::COLORED);
    assert!(
        colored.contains(
            "\u{1b}[32mtip:\u{1b}[0m a similar flag exists: `\u{1b}[32m--force\u{1b}[0m`"
        ),
        "{colored:?}"
    );
    let error = Tool::parse_from(["biuld"]).unwrap_err();
    assert!(
        error
            .render(Style::PLAIN)
            .contains("tip: a similar subcommand exists: `build`"),
        "{error:?}"
    );
    // Nothing close: the report is as it was.
    let error = Tool::parse_from(["--zzz"]).unwrap_err();
    assert_eq!(
        error.render(Style::PLAIN),
        "error: unknown flag `--zzz`\n\nFor more information, try '--help'."
    );
}

#[derive(Args, Debug)]
#[arg(name = "git")]
struct Git {
    #[arg(subcommand)]
    command: GitCommand,
}

#[derive(Subcommand, Debug)]
enum GitCommand {
    Fetch(Fetch),
}

#[derive(Args, Debug)]
struct Fetch {
    /// Where to fetch from
    #[arg(short, long, value_name = "URL")]
    remote: String,
    /// Talk more
    #[arg(short, long)]
    verbose: bool,
    /// What to fetch
    #[arg(positional)]
    refspec: String,
}

#[test]
fn every_missing_required_argument_is_listed_as_help_shows_it() {
    let error = Git::parse_from(["fetch", "-v"]).unwrap_err();
    assert_eq!(
        (error.kind(), error.token()),
        (ErrorKind::MissingRequired, Some("--remote"))
    );
    assert_eq!(
        error.to_string(),
        "the following required arguments were not provided: `--remote`, `<REFSPEC>`"
    );
    let report = winnow_args::with_env(&[("COLUMNS", "100")], || error.render(Style::PLAIN));
    let rows = if cfg!(feature = "help-text") {
        "  -r, --remote <URL>  Where to fetch from\n  <REFSPEC>           What to fetch\n"
    } else {
        // Without the prose, the spellings alone.
        "  -r, --remote <URL>\n  <REFSPEC>\n"
    };
    assert_eq!(
        report,
        format!(
            "error: the following required arguments were not provided:\n\n{rows}\n\
             For more information, try '--help'."
        )
    );
    let error = Git::parse_from(["fetch", "-r", "origin"]).unwrap_err();
    assert_eq!(
        (error.kind(), error.token()),
        (ErrorKind::MissingArgument, Some("REFSPEC"))
    );
}
