//! Relations between arguments: conflicts, overrides, requires, groups, required.

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, with_env};

fn parse<T: Args>(line: &[&str], env: &[(&str, &str)]) -> Result<T, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    with_env(env, || T::parse_from(&words))
}

fn err<T: Args + std::fmt::Debug>(line: &[&str], env: &[(&str, &str)]) -> Error {
    parse::<T>(line, env).expect_err(&format!("{line:?} should fail"))
}

#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its errors"
)]
#[derive(Args, Debug)]
struct Conflicts {
    #[arg(short, long, conflicts("--local", "--path"))]
    global: bool,
    #[arg(short, long)]
    local: bool,
    #[arg(long, env = "WA_PATH")]
    path: Option<String>,
    #[arg(long, default = "info", conflicts = "-g")]
    level: Option<String>,
}

#[test]
fn conflicts_count_what_was_supplied() {
    let e = err::<Conflicts>(&["-g", "-l"], &[]);
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::Conflict, Some("--global"), Some("--local"))
    );
    assert_eq!(e.to_string(), "`--global` cannot be used with `--local`");
    assert_eq!(
        err::<Conflicts>(&["--path=x", "-g"], &[]).kind(),
        ErrorKind::Conflict
    );
    // The environment supplies a value; a default does not.
    assert_eq!(
        err::<Conflicts>(&["-g"], &[("WA_PATH", "x")]).kind(),
        ErrorKind::Conflict
    );
    let ok = parse::<Conflicts>(&["-g"], &[]).unwrap();
    assert_eq!((ok.global, ok.level.as_deref()), (true, Some("info")));
    // Declared once, a conflict holds both ways; unrelated flags combine freely.
    assert_eq!(
        err::<Conflicts>(&["--level=x", "-g"], &[]).kind(),
        ErrorKind::Conflict
    );
    assert!(parse::<Conflicts>(&["-l", "--path", "x"], &[]).is_ok());
}

#[derive(Args, Debug, PartialEq)]
struct Overrides {
    #[arg(long, overrides = "--no-color")]
    color: bool,
    #[arg(long, env = "WA_NO_COLOR")]
    no_color: bool,
}

#[test]
fn the_last_of_two_overriding_flags_wins() {
    let o = parse::<Overrides>(&["--color", "--no-color"], &[]).unwrap();
    assert_eq!((o.color, o.no_color), (false, true));
    let o = parse::<Overrides>(&["--no-color", "--color"], &[]).unwrap();
    assert_eq!((o.color, o.no_color), (true, false));
    // A flag that lost is not refilled from the environment.
    let o = parse::<Overrides>(&["--color"], &[("WA_NO_COLOR", "1")]).unwrap();
    assert_eq!((o.color, o.no_color), (true, false));
    assert!(
        parse::<Overrides>(&[], &[("WA_NO_COLOR", "1")])
            .unwrap()
            .no_color
    );
}

#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its errors"
)]
#[derive(Args, Debug)]
struct Requires {
    #[arg(long, requires("--json", "--key"))]
    strict: bool,
    #[arg(long)]
    json: bool,
    #[arg(long, env = "WA_KEY")]
    key: Option<String>,
}

#[test]
fn requires_needs_every_target_to_have_a_value() {
    let e = err::<Requires>(&["--strict"], &[]);
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::MissingRequired, Some("--json"), Some("--strict"))
    );
    assert_eq!(e.to_string(), "`--json` is required by `--strict`");
    let e = err::<Requires>(&["--strict", "--json"], &[]);
    assert_eq!(e.token(), Some("--key"));
    // A target's value may come from the environment.
    assert!(parse::<Requires>(&["--strict", "--json"], &[("WA_KEY", "k")]).is_ok());
    // Nothing is required while the declaring flag is absent.
    assert!(parse::<Requires>(&[], &[]).is_ok());
}

#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its errors"
)]
#[derive(Args, Debug)]
#[arg(group("output"), group("source", required, multiple))]
struct Groups {
    #[arg(short = 'J', long, group = "output")]
    json: bool,
    #[arg(long, group = "output")]
    toml: bool,
    #[arg(long, group = "source")]
    brew: bool,
    #[arg(long, group = "source")]
    nvm: bool,
}

#[test]
fn groups_bound_how_many_members_are_given() {
    // A bare group allows at most one.
    let e = err::<Groups>(&["--brew", "-J", "--toml"], &[]);
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::Conflict, Some("--json"), Some("--toml"))
    );
    // `required, multiple` needs at least one.
    let e = err::<Groups>(&["--json"], &[]);
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (
            ErrorKind::MissingOneOf,
            Some("source"),
            Some("--brew, --nvm")
        )
    );
    assert!(parse::<Groups>(&["--brew", "--nvm", "--toml"], &[]).is_ok());
}

#[expect(
    dead_code,
    reason = "the fields declare the parser; these tests assert on its errors"
)]
#[derive(Args, Debug)]
struct Required {
    #[arg(short, long)]
    all: bool,
    #[arg(positional, required_unless = "--all")]
    tools: Vec<String>,
}

#[derive(Args, Debug)]
struct RequiredVec {
    #[arg(positional, required)]
    targets: Vec<String>,
}

#[test]
fn required_and_required_unless() {
    let e = err::<Required>(&[], &[]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingArgument, Some("TOOLS"))
    );
    assert!(parse::<Required>(&["--all"], &[]).is_ok());
    assert!(parse::<Required>(&["node"], &[]).is_ok());

    let e = err::<RequiredVec>(&[], &[]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingArgument, Some("TARGETS"))
    );
    assert_eq!(parse::<RequiredVec>(&["a"], &[]).unwrap().targets, ["a"]);
}

#[derive(Args, Debug)]
struct RequiredSwitch {
    /// `mise sync ruby --brew`: the switch must be given.
    #[arg(long, required)]
    brew: bool,
}

#[test]
fn a_required_switch_must_be_given() {
    let e = err::<RequiredSwitch>(&[], &[]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingRequired, Some("--brew"))
    );
    assert!(parse::<RequiredSwitch>(&["--brew"], &[]).unwrap().brew);
}
