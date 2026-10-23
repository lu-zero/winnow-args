//! Relations between arguments: conflicts, overrides, requires, groups, required.

use winnow_args::{Args, Error, ErrorKind, with_env};

fn parse<T: Args>(line: &[&str], env: &[(&str, &str)]) -> Result<T, Error> {
    let words = crate::words(line);
    with_env(env, || T::parse_words(&words))
}

fn err<T: Args + std::fmt::Debug>(line: &[&str], env: &[(&str, &str)]) -> Error {
    parse::<T>(line, env).expect_err(&format!("{line:?} should fail"))
}

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

/// `requires` of a group: one member, once the flag is supplied.
#[derive(Args, Debug)]
#[arg(group("loaded"))]
struct Fork {
    #[arg(short = 'c', long = "continue", group = "loaded")]
    continue_session: bool,
    #[arg(long, group = "loaded", env = "WA_RESUME")]
    resume: Option<String>,
    #[arg(long, requires = "loaded")]
    check: bool,
    #[arg(long, requires("loaded", message = "{0} needs one of {1}"))]
    fork: bool,
}

#[test]
fn a_flag_requires_one_member_of_a_group() {
    let e = err::<Fork>(&["--check"], &[]);
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (
            ErrorKind::MissingOneOf,
            Some("loaded"),
            Some("--continue, --resume")
        )
    );
    assert_eq!(
        e.to_string(),
        "`--check` requires one of `--continue`, `--resume`"
    );
    let e = err::<Fork>(&["--fork"], &[]);
    assert_eq!(e.kind(), ErrorKind::MissingOneOf);
    assert_eq!(e.to_string(), "--fork needs one of --continue, --resume");
    assert!(parse::<Fork>(&["--fork", "--continue"], &[]).is_ok());
    assert!(parse::<Fork>(&["--fork", "--resume", "s"], &[]).is_ok());
    // A member's value may come from the environment. Nothing is required
    // while the declaring flag is absent.
    assert!(parse::<Fork>(&["--fork"], &[("WA_RESUME", "s")]).is_ok());
    assert!(parse::<Fork>(&[], &[]).is_ok());
    // The group's own rule still holds: at most one member.
    assert_eq!(
        err::<Fork>(&["--fork", "--continue", "--resume", "s"], &[]).kind(),
        ErrorKind::Conflict
    );
}

#[derive(Args, Debug)]
#[arg(group("output"))]
struct Exported {
    #[arg(long, requires = "output")]
    export: bool,
    #[arg(long, group = "output", default = "text")]
    format: Option<String>,
}

#[test]
fn a_members_default_satisfies_the_requirement() {
    let exported = parse::<Exported>(&["--export"], &[]).unwrap();
    assert_eq!(exported.format.as_deref(), Some("text"));
}

#[derive(Args, Debug)]
#[arg(group("json"))]
struct FieldFirst {
    #[arg(long, group = "json")]
    json: bool,
    #[arg(long, group = "json")]
    toml: bool,
    #[arg(long, requires = "json")]
    strict: bool,
}

#[test]
fn a_field_is_named_ahead_of_a_group() {
    let e = err::<FieldFirst>(&["--strict"], &[]);
    assert_eq!(
        (e.kind(), e.to_string()),
        (
            ErrorKind::MissingRequired,
            "`--json` is required by `--strict`".to_owned()
        )
    );
    // `--toml` would satisfy the group. It does not satisfy the field.
    assert_eq!(
        err::<FieldFirst>(&["--strict", "--toml"], &[]).kind(),
        ErrorKind::MissingRequired
    );
    assert!(parse::<FieldFirst>(&["--strict", "--json"], &[]).is_ok());
}

#[derive(Args, Debug)]
#[arg(group("loaded"))]
struct FieldThenGroup {
    #[arg(long)]
    json: bool,
    #[arg(long, group = "loaded")]
    resume: bool,
    #[arg(long, requires("--json", "loaded"))]
    strict: bool,
}

#[test]
fn a_missing_field_is_reported_before_the_group() {
    let e = err::<FieldThenGroup>(&["--strict"], &[]);
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::MissingRequired, Some("--json"))
    );
    let e = err::<FieldThenGroup>(&["--strict", "--json"], &[]);
    assert_eq!(e.kind(), ErrorKind::MissingOneOf);
    assert_eq!(e.token(), Some("loaded"));
    assert!(parse::<FieldThenGroup>(&["--strict", "--json", "--resume"], &[]).is_ok());
}

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

/// `required_if` and `required_unless_all`.
mod conditional {
    use winnow_args::{Args, ErrorKind, ValueEnum, with_env};

    #[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
    enum Format {
        Text,
        Json,
    }

    #[derive(Args, Debug)]
    struct Export {
        #[arg(long, env = "EXPORT_FORMAT")]
        format: Option<Format>,
        #[arg(long)]
        upload: bool,
        /// Required for JSON, and for an upload.
        #[arg(long, required_if("--format", "json"), required_if("--upload"))]
        schema: Option<String>,
        #[arg(long, default = "out")]
        dir: String,
        #[arg(long)]
        name: Option<String>,
        /// Not needed when both of the others are given.
        #[arg(long, required_unless_all("--dir", "--name"))]
        target: Option<String>,
    }

    fn parse(line: &[&str]) -> Result<Export, winnow_args::Error> {
        Export::parse_from(line)
    }

    #[test]
    fn a_value_of_another_field_makes_it_required() {
        let error = parse(&["--format", "json", "--name", "n"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::MissingRequired);
        assert_eq!(
            error.to_string(),
            "`--schema` is required by `--format=json`"
        );
        assert!(parse(&["--format", "text", "--name", "n"]).is_ok());
        assert!(parse(&["--format=json", "--schema", "s", "--name", "n"]).is_ok());
    }

    #[test]
    fn another_field_given_makes_it_required() {
        let error = parse(&["--upload", "--name", "n"]).unwrap_err();
        assert_eq!(error.to_string(), "`--schema` is required by `--upload`");
    }

    #[test]
    fn the_environment_counts_as_given() {
        let error = with_env(&[("EXPORT_FORMAT", "json")], || parse(&["--name", "n"])).unwrap_err();
        assert_eq!(
            error.to_string(),
            "`--schema` is required by `--format=json`"
        );
    }

    #[test]
    fn required_unless_all_needs_every_one() {
        // `--dir` has a default, so with `--name` both have a value.
        assert!(parse(&["--name", "n"]).unwrap().target.is_none());
        let error = parse(&[]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the following required arguments were not provided: `--target`"
        );
        assert!(parse(&["--target", "t"]).is_ok());
    }
}

/// `requires_if` and `exclusive`.
mod requires_if_exclusive {
    use winnow_args::{Args, ErrorKind, ValueEnum};

    #[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
    enum Mode {
        Local,
        Remote,
    }

    #[derive(Args, Debug)]
    struct Sync {
        /// A remote sync needs a host.
        #[arg(long, requires_if("remote", "--host"))]
        mode: Option<Mode>,
        #[arg(long)]
        host: Option<String>,
        #[arg(long, default = "4")]
        jobs: u32,
        /// Only by itself.
        #[arg(long, exclusive)]
        list: bool,
    }

    fn parse(line: &[&str]) -> Result<Sync, winnow_args::Error> {
        Sync::parse_from(line)
    }

    #[test]
    fn a_value_requires_another_field() {
        let error = parse(&["--mode", "remote"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::MissingRequired);
        assert_eq!(error.to_string(), "`--host` is required by `--mode=remote`");
        assert!(parse(&["--mode", "local"]).is_ok());
        assert!(parse(&["--mode", "remote", "--host", "h"]).is_ok());
    }

    #[test]
    fn an_exclusive_flag_is_given_alone() {
        assert!(parse(&["--list"]).unwrap().list);
        let error = parse(&["--list", "--jobs", "2"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Conflict);
        assert_eq!(error.to_string(), "`--list` cannot be used with `--jobs`");
        // A default is not something given.
        assert_eq!(parse(&["--list"]).unwrap().jobs, 4);
    }
}

/// Messages of the program's own, as brush's `unset` gives bash's.
mod worded {
    use super::*;

    #[derive(Args, Debug)]
    #[arg(group("how", required, message = "{0} or {1}: say how, with {1}"))]
    struct Unset {
        #[arg(short = 'f', group = "how")]
        function: bool,
        #[arg(
            short = 'v',
            group = "how",
            conflicts(
                "-f",
                message = "cannot simultaneously unset a function and a variable"
            )
        )]
        variable: bool,
        #[arg(short = 'n', requires("-v", message = "{0} needs {1}"))]
        nameref: bool,
    }

    #[derive(Args, Debug)]
    #[arg(group("mode", message = "{0} and {1} are both modes"))]
    struct Modes {
        #[arg(long, group = "mode")]
        fast: bool,
        #[arg(long, group = "mode")]
        safe: bool,
    }

    #[test]
    fn a_rule_says_its_own_message_of_the_declared_names() {
        let e = err::<Unset>(&["-f", "-v"], &[]);
        assert_eq!(
            (e.kind(), e.token(), e.value()),
            (ErrorKind::Conflict, Some("-v"), Some("-f"))
        );
        assert_eq!(
            e.to_string(),
            "cannot simultaneously unset a function and a variable"
        );
        let e = err::<Unset>(&["-f", "-n"], &[]);
        assert_eq!(e.kind(), ErrorKind::MissingRequired);
        assert_eq!(e.to_string(), "-n needs -v");
        let e = err::<Unset>(&[], &[]);
        assert_eq!(e.kind(), ErrorKind::MissingOneOf);
        assert_eq!(e.to_string(), "how or -f, -v: say how, with -f, -v");
        let e = err::<Modes>(&["--safe", "--fast"], &[]);
        assert_eq!(e.kind(), ErrorKind::Conflict);
        assert_eq!(e.to_string(), "--fast and --safe are both modes");
    }
}
