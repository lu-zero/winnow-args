//! Help and version: rendered from doc comments and attributes, requested with
//! `-h`, `--help`, `help <command>`, `-V`, or a bare `arg_required_else_help` call.

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, Subcommand, ValueEnum, report, with_env};

/// Dev tools, env vars, and tasks in one CLI
///
/// mise prepares your development environment before each command runs.
#[derive(Args, Debug)]
#[arg(
    name = "mise",
    version = "2026.9.0",
    after_help = "Examples:\n    $ mise use -g node@20"
)]
#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its help"
)]
struct Cli {
    /// Show extra output (repeat for more)
    #[arg(short, long, count, global)]
    verbose: u8,
    /// Change to this directory before executing the command
    #[arg(short = 'C', long, value_name = "DIR", env = "MISE_CD")]
    cd: Option<String>,
    /// Number of jobs to run in parallel
    ///
    /// Defaults to the number of CPUs.
    #[arg(short, long, default = "4", help_heading = "Performance")]
    jobs: Option<u32>,
    /// Internal
    #[arg(long, hide)]
    debug_internal: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its help"
)]
enum Command {
    /// Installs a tool and adds the version to mise.toml
    #[arg(alias = "u")]
    Use(UseArgs),
    /// Replaced by the `help` attribute in the listing.
    #[arg(
        name = "ls",
        alias = "list",
        alias_hidden = "l",
        help = "List installed tools"
    )]
    Ls,
}

#[derive(ValueEnum, Debug)]
enum When {
    Auto,
    Always,
    Never,
    /// Not for users.
    #[arg(hide)]
    Debug,
}

/// Installs a tool and adds the version to mise.toml.
#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its help"
)]
struct UseArgs {
    /// Use the global config file
    #[arg(short, long)]
    global: bool,
    /// Output format
    #[arg(long, choices("json", "toml"))]
    format: Option<String>,
    /// When to use color
    #[arg(long)]
    color: Option<When>,
    /// Tool(s) to add
    #[arg(positional, value_name = "TOOL@VERSION")]
    tools: Vec<String>,
}

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    T::parse_from(&words)
}

/// Help at the default width: `COLUMNS` unset whatever the test runner has.
fn help(line: &[&str]) -> String {
    help_at(line, &[])
}

fn help_at(line: &[&str], env: &[(&str, &str)]) -> String {
    let e = parse::<Cli>(line).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested, "{line:?}");
    with_env(env, || e.render_help("mise").unwrap())
}

#[test]
fn short_help_lists_what_is_visible() {
    assert_eq!(
        help(&["-h"]),
        "\
Dev tools, env vars, and tasks in one CLI

Usage: mise [OPTIONS] [COMMAND]

Commands:
  use, u    Installs a tool and adds the version to mise.toml
  ls, list  List installed tools

Options:
  -v, --verbose...  Show extra output (repeat for more)
  -C, --cd <DIR>    Change to this directory before executing the command [env: MISE_CD]
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version

Performance:
  -j, --jobs <JOBS>  Number of jobs to run in parallel [default: 4]

Examples:
    $ mise use -g node@20
"
    );
}

#[test]
fn long_help_has_the_whole_descriptions() {
    let long = help(&["--help"]);
    assert!(long.contains("mise prepares your development environment"));
    assert!(long.contains("          Defaults to the number of CPUs.\n          [default: 4]"));
    assert!(!long.contains("debug-internal"), "hidden flags stay hidden");
}

#[test]
fn a_subcommand_help_names_its_path() {
    let text = help(&["use", "-h"]);
    assert!(text.starts_with("Installs a tool and adds the version to mise.toml.\n\nUsage: mise use [OPTIONS] [TOOL@VERSION]...\n"));
    assert!(text.contains("--format <FORMAT>  Output format [possible values: json, toml]"));
    // A `ValueEnum`'s visible variants, without being declared again.
    assert!(
        text.contains("--color <COLOR>    When to use color [possible values: auto, always, never]"),
        "{text}"
    );
    // A bundle can ask for help too.
    assert!(help(&["-vh"]).starts_with("Dev tools"));
}

#[test]
fn the_help_word_follows_subcommand_names() {
    // Through an alias to the canonical name, and in the long form.
    assert!(help(&["help", "list"]).contains("Usage: mise ls [OPTIONS]"));
    assert!(help(&["help", "u"]).contains("Usage: mise use [OPTIONS]"));
    assert!(help(&["help"]).contains("mise prepares your development environment"));
}

#[test]
fn report_prints_and_chooses_the_exit_status() {
    assert_eq!(report(&parse::<Cli>(&["-h"]).unwrap_err(), "mise"), 0);
    let version = parse::<Cli>(&["-V"]).unwrap_err();
    assert_eq!(
        version.version_text("mise").as_deref(),
        Some("mise 2026.9.0")
    );
    assert_eq!(report(&version, "mise"), 0);
    // A bare `arg_required_else_help` call is help, but also a failure.
    let bare = parse::<Cli>(&["use"]).unwrap_err();
    assert!(bare.is_bare_help());
    assert_eq!(report(&bare, "mise"), 2);
    assert_eq!(report(&parse::<Cli>(&["--nope"]).unwrap_err(), "mise"), 2);
}

#[derive(Args, Debug)]
struct OwnsH {
    /// Host to connect to
    #[arg(short = 'h', long)]
    host: Option<String>,
}

#[derive(Args, Debug)]
#[arg(disable_help_flag)]
struct NoHelp {}

#[test]
fn a_declared_or_disabled_help_flag_is_not_supplied() {
    // `-h` is the CLI's own; `--help` is still supplied.
    assert_eq!(
        parse::<OwnsH>(&["-h", "x"]).unwrap().host.as_deref(),
        Some("x")
    );
    assert_eq!(
        parse::<OwnsH>(&["--help"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    assert_eq!(
        parse::<NoHelp>(&["--help"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[test]
fn descriptions_wrap_under_their_column() {
    assert_eq!(
        help_at(&["-h"], &[("COLUMNS", "60")]),
        "\
Dev tools, env vars, and tasks in one CLI

Usage: mise [OPTIONS] [COMMAND]

Commands:
  use, u    Installs a tool and adds the version to
            mise.toml
  ls, list  List installed tools

Options:
  -v, --verbose...  Show extra output (repeat for more)
  -C, --cd <DIR>    Change to this directory before
                    executing the command [env: MISE_CD]
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version

Performance:
  -j, --jobs <JOBS>  Number of jobs to run in parallel
                     [default: 4]

Examples:
    $ mise use -g node@20
"
    );
}

#[test]
fn a_narrow_page_puts_descriptions_under_wide_items() {
    // At 40 columns the column is at most 14 wide: `-C, --cd <DIR>` fits,
    // `-v, --verbose...` goes to its own line.
    let text = help_at(&["-h"], &[("COLUMNS", "40")]);
    assert!(
        text.contains("  -v, --verbose...\n                  Show extra output\n"),
        "{text}"
    );
    for line in text.lines() {
        assert!(
            line.chars().count() <= 40 || !line.contains(' '),
            "{line:?}"
        );
    }
}

#[test]
fn long_help_wraps_its_indented_blocks() {
    let text = help_at(&["--help"], &[("COLUMNS", "50")]);
    assert!(
        text.contains(
            "          Change to this directory before\n          executing the command\n"
        ),
        "{text}"
    );
}
