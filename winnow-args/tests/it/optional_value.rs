//! `default_missing`: a flag whose value may be left out (usage's corpus vectors
//! `*-default-missing-*`), and `restart_token`.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow_args::combinator::{Named, args, long, positional, short};
use winnow_args::{Args, Argv, Error, ValueEnum, with_env};

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy)]
enum Clear {
    Clear,
    Reset,
}

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short, long, default_missing = "always")]
    color: Option<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, default_missing = "30s", default = "off")]
    poll: Option<String>,
    #[arg(long, default_missing = "clear")]
    clear: Option<Clear>,
    #[arg(positional)]
    rest: Option<String>,
}

const COLOR: Named = short('c').long("color");
const VERBOSE: Named = short('v').long("verbose");
const POLL: Named = long("poll");
const CLEAR: Named = long("clear");

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    args(alt((
        COLOR.argument_or("always").map(|c| cli.color = Some(c)),
        VERBOSE.switch().map(|()| cli.verbose = true),
        POLL.argument_or("30s").map(|p| cli.poll = Some(p)),
        CLEAR.argument_or("clear").map(|c| cli.clear = Some(c)),
        positional("REST").map(|r| cli.rest = Some(r)),
    )))
    .parse_next(input)?;
    cli.poll.get_or_insert_with(|| "off".into());
    Ok(cli)
}

fn ok(line: &[&str]) -> Cli {
    let words = crate::words(line);
    let a = combinator.parse_next(&mut Argv::new(&words)).unwrap();
    let b = with_env(&[], || Cli::parse_from(&words)).unwrap();
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

#[test]
fn a_bare_flag_takes_its_missing_default() {
    assert_eq!(ok(&["--color"]).color.as_deref(), Some("always"));
    assert_eq!(ok(&["-c"]).color.as_deref(), Some("always"));
    assert_eq!(ok(&["--clear"]).clear, Some(Clear::Clear));
}

#[test]
fn a_given_value_wins() {
    assert_eq!(ok(&["--color=never"]).color.as_deref(), Some("never"));
    assert_eq!(ok(&["-cnever"]).color.as_deref(), Some("never"));
    // A detached word is still the value, as in the corpus.
    let cli = ok(&["--color", "never"]);
    assert_eq!((cli.color.as_deref(), cli.rest), (Some("never"), None));
    assert_eq!(ok(&["--clear", "reset"]).clear, Some(Clear::Reset));
}

#[test]
fn a_following_flag_is_not_the_value() {
    for line in [&["--color", "--verbose"][..], &["-c", "-v"]] {
        let cli = ok(line);
        assert_eq!(
            (cli.color.as_deref(), cli.verbose),
            (Some("always"), true),
            "{line:?}"
        );
    }
}

#[test]
fn absent_is_not_bare() {
    let cli = ok(&[]);
    assert_eq!((cli.color, cli.poll.as_deref()), (None, Some("off")));
    assert_eq!(ok(&["--poll"]).poll.as_deref(), Some("30s"));
}

#[derive(Args, Debug, PartialEq)]
#[arg(restart_token = ":::")]
struct Run {
    #[arg(short, long)]
    force: bool,
    #[arg(long)]
    jobs: Option<u32>,
    #[arg(positional)]
    task: Option<String>,
    #[arg(positional)]
    args: Vec<String>,
}

fn run(line: &[&str]) -> Run {
    Run::try_parse_from(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

#[test]
fn the_restart_token_starts_the_positionals_over() {
    let r = run(&["lint", "a", ":::", "test", "b"]);
    assert_eq!(
        (r.task.as_deref(), r.args),
        (Some("test"), vec!["b".to_string()])
    );
    // Flags carry over between invocations.
    let r = run(&["--jobs", "2", "lint", ":::", "-f", "test"]);
    assert_eq!(
        (r.jobs, r.force, r.task.as_deref()),
        (Some(2), true, Some("test"))
    );
    // It also ends a `--`: flags are flags again.
    let r = run(&["lint", "--", "--x", ":::", "--force", "test"]);
    assert_eq!(
        (r.force, r.task.as_deref(), r.args.len()),
        (true, Some("test"), 0)
    );
}
