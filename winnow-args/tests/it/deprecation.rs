//! Deprecated flags, positionals and subcommands, which a parse records as
//! warnings, and environment variables read from more than one name.

use winnow_args::warn::{Warning, WarningKind};
use winnow_args::{Args, Subcommand, with_env};

#[derive(Args, Debug, Default)]
#[arg(version = "1.5.0")]
struct Cli {
    #[arg(short, long, deprecated = "use --new")]
    old: bool,
    #[arg(long)]
    new: bool,
    #[arg(long, deprecated, deprecated_remove_at = "2.0")]
    gone: Option<String>,
    // Not warned about before the release it names.
    #[arg(long, deprecated, deprecated_warn_at = "1.6")]
    later: bool,
    #[arg(long, deprecated, deprecated_warn_at = "1.5")]
    now: bool,
    #[arg(positional, deprecated = "name it with --new")]
    file: Option<String>,
}

fn names(warnings: &[Warning]) -> Vec<&str> {
    warnings.iter().map(|w| w.name.as_str()).collect()
}

#[test]
fn a_deprecated_flag_given_is_a_warning_once() {
    let _ = Cli::warnings();
    let cli = Cli::parse_from(["-o", "--old", "--new"]).unwrap();
    assert!(cli.old && cli.new);
    let warnings = Cli::warnings();
    assert_eq!(names(&warnings), ["--old"]);
    assert_eq!(warnings[0].kind, WarningKind::Argument);
    assert_eq!(
        warnings[0].render(),
        "warning: --old is deprecated: use --new\n"
    );
    // Taken: the next call has nothing.
    assert!(Cli::warnings().is_empty());
    // A flag not given warns about nothing.
    Cli::parse_from(["--new"]).unwrap();
    assert!(Cli::warnings().is_empty());
}

#[test]
fn removal_and_the_release_warnings_start_at() {
    let _ = Cli::warnings();
    Cli::parse_from(["--gone", "x", "--later", "--now", "a.txt"]).unwrap();
    let warnings = Cli::warnings();
    assert_eq!(names(&warnings), ["--gone", "--now", "<FILE>"]);
    assert_eq!(
        warnings[0].render(),
        "warning: --gone is deprecated, removed at 2.0\n"
    );
    assert_eq!(
        warnings[2].render(),
        "warning: <FILE> is deprecated: name it with --new\n"
    );
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Build,
    #[arg(alias = "mk", deprecated = "use build", deprecated_remove_at = "3.0")]
    Make,
    // A variant with fields: the options are the variant's, not its command's.
    #[arg(deprecated)]
    Old {
        #[arg(long)]
        fast: bool,
    },
}

#[derive(Args, Debug)]
struct Tool {
    #[arg(subcommand)]
    command: Command,
}

#[test]
fn a_deprecated_subcommand_is_a_warning_by_the_name_typed() {
    let _ = Tool::warnings();
    assert_eq!(Tool::parse_from(["mk"]).unwrap().command, Command::Make);
    let warnings = Tool::warnings();
    assert_eq!(warnings[0].kind, WarningKind::Command);
    assert_eq!(
        warnings[0].render(),
        "warning: the mk command is deprecated, removed at 3.0: use build\n"
    );
    Tool::parse_from(["build"]).unwrap();
    assert!(Tool::warnings().is_empty());
    let old = Tool::parse_from(["old", "--fast"]).unwrap().command;
    assert_eq!(old, Command::Old { fast: true });
    assert_eq!(names(&Tool::warnings()), ["old"]);
}

#[derive(Args, Debug)]
struct Env {
    #[arg(
        long,
        env = "TOOL_JOBS",
        env_fallback("JOBS", "MAKE_JOBS"),
        deprecated_env = "OLD_JOBS"
    )]
    jobs: Option<u32>,
    #[arg(long, env_fallback = "TOOL_COLOR")]
    color: bool,
}

#[test]
fn variables_are_read_in_order() {
    let jobs = |vars: &[(&str, &str)]| with_env(vars, || Env::parse_from([""; 0]).unwrap().jobs);
    assert_eq!(jobs(&[("MAKE_JOBS", "3"), ("JOBS", "2")]), Some(2));
    assert_eq!(
        jobs(&[("TOOL_JOBS", "1"), ("JOBS", "2"), ("OLD_JOBS", "9")]),
        Some(1)
    );
    let _ = Env::warnings();
    assert_eq!(jobs(&[("MAKE_JOBS", "3")]), Some(3));
    assert!(Env::warnings().is_empty());
    // The first fallback stands for `env` when there is none.
    let color = with_env(&[("TOOL_COLOR", "1")], || {
        Env::parse_from([""; 0]).unwrap().color
    });
    assert!(color);
    let item = Env::HELP
        .items
        .iter()
        .find(|i| i.long == Some("color"))
        .unwrap();
    assert_eq!(item.env, Some("TOOL_COLOR"));
}

#[test]
fn a_value_from_a_deprecated_name_names_the_one_to_use() {
    let _ = Env::warnings();
    let jobs = with_env(&[("OLD_JOBS", "9")], || {
        Env::parse_from([""; 0]).unwrap().jobs
    });
    assert_eq!(jobs, Some(9));
    let warnings = Env::warnings();
    assert_eq!(warnings[0].kind, WarningKind::Env);
    assert_eq!(warnings[0].replacement, Some("TOOL_JOBS"));
    assert_eq!(
        warnings[0].render(),
        "warning: the variable OLD_JOBS is deprecated: use TOOL_JOBS\n"
    );
    // The command line wins, and reads no variable.
    with_env(&[("OLD_JOBS", "9")], || {
        Env::parse_from(["--jobs", "4"]).unwrap()
    });
    assert!(Env::warnings().is_empty());
}

#[cfg(feature = "help-text")]
#[test]
fn help_marks_what_is_deprecated() {
    use winnow_args::help::render;
    let short = render(Cli::HELP, &["cli"], false);
    assert!(
        short.contains("-o, --old") && short.contains("[deprecated]"),
        "{short}"
    );
    let long = render(Cli::HELP, &["cli"], true);
    assert!(long.contains("Deprecated: use --new\n"), "{long}");
    assert!(long.contains("Deprecated. Removed at 2.0.\n"), "{long}");
    let tool = render(Tool::HELP, &["tool"], false);
    assert!(tool.contains("make, mk  (deprecated)"), "{tool}");
}
