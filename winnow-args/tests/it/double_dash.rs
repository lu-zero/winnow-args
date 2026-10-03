//! `double_dash`: positionals that only take words after `--`, or that stop
//! flag parsing once they have a value.

use winnow_args::{Args, Error, ErrorKind};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::parse_from(line)
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

/// `mise exec [TOOL]... -- COMMAND...`
#[derive(Args, Debug)]
struct Exec {
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional)]
    tools: Vec<String>,
    #[arg(positional, required, double_dash = "required")]
    command: Vec<String>,
}

#[test]
fn words_after_the_separator_skip_to_the_trailing_argument() {
    let e: Exec = parse(&["node@20", "--", "node", "app.js", "-v"]).unwrap();
    assert_eq!(
        (e.tools, e.command, e.verbose),
        (
            strings(&["node@20"]),
            strings(&["node", "app.js", "-v"]),
            false
        )
    );
    // Even a greedy variadic before it does not take them.
    let e: Exec = parse(&["-v", "--", "ls"]).unwrap();
    assert_eq!(
        (e.tools.len(), e.command, e.verbose),
        (0, strings(&["ls"]), true)
    );
    let err = parse::<Exec>(&["node@20", "python"]).unwrap_err();
    assert_eq!(
        (err.kind(), err.token()),
        (ErrorKind::MissingArgument, Some("COMMAND"))
    );
}

#[derive(Args, Debug)]
struct Single {
    #[arg(positional)]
    task: Option<String>,
    #[arg(positional, double_dash = "required")]
    rest: Option<String>,
}

#[test]
fn a_word_before_the_separator_cannot_reach_it() {
    let s: Single = parse(&["t", "--", "a"]).unwrap();
    assert_eq!(
        (s.task.as_deref(), s.rest.as_deref()),
        (Some("t"), Some("a"))
    );
    let err = parse::<Single>(&["t", "u"]).unwrap_err();
    assert_eq!(
        (err.kind(), err.token(), err.offset()),
        (ErrorKind::RequiresDoubleDash, Some("REST"), 2)
    );
    assert_eq!(
        err.to_string(),
        "`REST` can only be set after a `--` separator"
    );
    let err = parse::<Single>(&["t", "--", "a", "b"]).unwrap_err();
    assert_eq!(
        (err.kind(), err.token()),
        (ErrorKind::UnexpectedArg, Some("b"))
    );
}

/// `mise asdf ARGS...`
#[derive(Args, Debug)]
struct Asdf {
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional, double_dash = "automatic")]
    args: Vec<String>,
}

#[test]
fn an_automatic_argument_stops_flags_once_it_has_a_value() {
    let a: Asdf = parse(&["-v", "install", "--version", "-v", "--", "x"]).unwrap();
    assert!(a.verbose);
    assert_eq!(a.args, strings(&["install", "--version", "-v", "--", "x"]));
    // Before its first value, flags are still flags.
    let err = parse::<Asdf>(&["--version"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::UnknownFlag);
}

/// mise's `run`: `[TASK] [ARGS]... [-- ARGS_LAST...]`, `TASK` automatic.
#[derive(Args, Debug)]
struct Run {
    #[arg(short, long)]
    force: bool,
    #[arg(positional, double_dash = "automatic")]
    task: Option<String>,
    #[arg(positional)]
    args: Vec<String>,
    #[arg(positional, double_dash = "required")]
    args_last: Vec<String>,
}

#[test]
fn automatic_and_required_share_a_struct() {
    // After `TASK`, flags stop: `-f` and a later `--` are ordinary `ARGS`.
    let r: Run = parse(&["lint", "-f", "--", "y"]).unwrap();
    assert_eq!(
        (r.task.as_deref(), r.args, r.args_last.len(), r.force),
        (Some("lint"), strings(&["-f", "--", "y"]), 0, false)
    );
    // A `--` typed before `TASK` has a value is a real separator.
    let r: Run = parse(&["-f", "--", "a", "b"]).unwrap();
    assert_eq!(
        (r.task, r.args_last, r.force),
        (None, strings(&["a", "b"]), true)
    );
}
