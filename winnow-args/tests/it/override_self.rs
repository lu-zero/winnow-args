//! A flag given twice: the last one, the first one, or an error.

use winnow_args::{Args, ErrorKind};

#[derive(Args, Debug, PartialEq)]
struct Defaults {
    #[arg(long)]
    name: Option<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, count)]
    quiet: u8,
    #[arg(long)]
    with: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
struct Mixed {
    #[arg(long, override_self = "first")]
    config: Option<String>,
    #[arg(long, override_self = "error")]
    output: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(short, long, override_self = "error")]
    yes: bool,
}

#[derive(Args, Debug, PartialEq)]
#[arg(override_self = "error")]
struct Strict {
    #[arg(long)]
    name: Option<String>,
    #[arg(long, override_self = "first")]
    config: Option<String>,
    #[arg(short, long, count)]
    quiet: u8,
    #[arg(long)]
    with: Vec<String>,
}

fn parse<T: Args>(line: &[&str]) -> Result<T, winnow_args::Error> {
    let words = crate::words(line);
    T::parse_words(&words)
}

#[test]
fn the_last_one_wins_unless_told() {
    let d = parse::<Defaults>(&[
        "--name", "a", "--name", "b", "-vv", "-qq", "--with", "x", "--with", "y",
    ])
    .unwrap();
    assert_eq!(d.name.as_deref(), Some("b"));
    assert!(d.verbose);
    assert_eq!(d.quiet, 2);
    assert_eq!(d.with, ["x", "y"]);
}

#[test]
fn the_first_one_is_kept() {
    let m = parse::<Mixed>(&[
        "--config", "a", "--config", "b", "--name", "x", "--name", "y",
    ])
    .unwrap();
    assert_eq!(m.config.as_deref(), Some("a"));
    assert_eq!(m.name.as_deref(), Some("y"));
}

#[test]
fn a_later_value_is_still_checked_when_the_first_is_kept() {
    #[derive(Args, Debug)]
    struct Port {
        #[arg(long, override_self = "first")]
        port: Option<u16>,
    }
    assert_eq!(
        parse::<Port>(&["--port", "1", "--port", "2"]).unwrap().port,
        Some(1)
    );
    let error = parse::<Port>(&["--port", "1", "--port", "many"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
}

#[test]
fn a_second_one_is_an_error_where_asked() {
    let error = parse::<Mixed>(&["--output", "a", "--output=b"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::RepeatedFlag);
    assert!(error.to_string().contains("--output"), "{error}");
    let ok = parse::<Mixed>(&["--output", "a"]).unwrap();
    assert_eq!(ok.output.as_deref(), Some("a"));
    // A switch too, spelled either way.
    let error = parse::<Mixed>(&["-y", "--yes"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::RepeatedFlag);
    assert!(parse::<Mixed>(&["-y"]).unwrap().yes);
}

#[test]
fn the_command_sets_the_default_and_a_field_overrides_it() {
    let error = parse::<Strict>(&["--name", "a", "--name", "b"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::RepeatedFlag);
    let s = parse::<Strict>(&[
        "--config", "a", "--config", "b", "-qq", "--with", "x", "--with", "y",
    ])
    .unwrap();
    assert_eq!(s.config.as_deref(), Some("a"));
    // A count and a list keep collecting.
    assert_eq!(s.quiet, 2);
    assert_eq!(s.with, ["x", "y"]);
}

#[test]
fn only_the_command_line_counts_as_a_first_occurrence() {
    #[derive(Args, Debug)]
    struct Env {
        #[arg(long, env = "OVERRIDE_SELF_TEST", override_self = "error")]
        name: Option<String>,
    }
    let words = crate::words(&["--name", "a"]);
    let name = winnow_args::with_env(&[("OVERRIDE_SELF_TEST", "env")], || {
        Env::parse_words(&words).unwrap().name
    });
    assert_eq!(name.as_deref(), Some("a"));
}

#[derive(Args, Debug, PartialEq)]
struct Odd {
    #[arg(long, bool_value, override_self = "first")]
    first_bool: bool,
    #[arg(long, bool_value, override_self = "error")]
    strict_bool: bool,
    #[arg(long, require_equals, override_self = "first")]
    first_eq: Option<String>,
    #[arg(long, require_equals, override_self = "error")]
    strict_eq: Option<String>,
    #[arg(long, negate, override_self = "first")]
    first_color: bool,
    #[arg(long, negate, override_self = "error")]
    strict_color: bool,
}

#[test]
fn a_value_that_says_nothing_about_being_given_still_counts() {
    let odd = |line: &[&str]| parse::<Odd>(line);
    // `--x=false` leaves `false` in the slot, as a flag never given does.
    assert!(
        !odd(&["--first-bool=false", "--first-bool=true"])
            .unwrap()
            .first_bool
    );
    assert_eq!(
        odd(&["--strict-bool=false", "--strict-bool=true"])
            .unwrap_err()
            .kind(),
        ErrorKind::RepeatedFlag
    );
    // A bare `require_equals` flag leaves `None`.
    assert_eq!(odd(&["--first-eq", "--first-eq=b"]).unwrap().first_eq, None);
    assert_eq!(
        odd(&["--strict-eq", "--strict-eq=b"]).unwrap_err().kind(),
        ErrorKind::RepeatedFlag
    );
}

#[test]
fn a_flag_and_its_negation_are_one_flag() {
    let odd = |line: &[&str]| parse::<Odd>(line);
    assert!(
        odd(&["--first-color", "--no-first-color"])
            .unwrap()
            .first_color
    );
    assert!(
        !odd(&["--no-first-color", "--first-color"])
            .unwrap()
            .first_color
    );
    for line in [
        ["--strict-color", "--no-strict-color"],
        ["--no-strict-color", "--no-strict-color"],
    ] {
        assert_eq!(odd(&line).unwrap_err().kind(), ErrorKind::RepeatedFlag);
    }
}

#[test]
fn a_second_one_fails_before_its_value_is_read() {
    let error = parse::<Mixed>(&["--output", "a", "--output"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::RepeatedFlag);
}

#[test]
fn a_default_is_not_an_occurrence() {
    #[derive(Args, Debug)]
    struct Defaulted {
        #[arg(long, default = "d", override_self = "error")]
        name: Option<String>,
    }
    assert_eq!(
        parse::<Defaulted>(&["--name", "a"])
            .unwrap()
            .name
            .as_deref(),
        Some("a")
    );
    assert_eq!(parse::<Defaulted>(&[]).unwrap().name.as_deref(), Some("d"));
}

#[test]
fn a_flattened_struct_has_its_own_setting() {
    #[derive(Args, Debug, Default)]
    struct Inner {
        #[arg(long)]
        name: Option<String>,
    }
    #[derive(Args, Debug)]
    #[arg(override_self = "error")]
    struct Outer {
        #[arg(long)]
        mine: Option<String>,
        #[arg(flatten)]
        inner: Inner,
    }
    let o = parse::<Outer>(&["--name", "a", "--name", "b"]).unwrap();
    assert_eq!(o.inner.name.as_deref(), Some("b"));
    let error = parse::<Outer>(&["--mine", "a", "--mine", "b"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::RepeatedFlag);
}
