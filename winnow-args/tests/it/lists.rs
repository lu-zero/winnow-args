//! A flag's values held as `Option<Vec<T>>`, `Vec<Vec<T>>` or
//! `Vec<Option<T>>`, beside the plain `Vec<T>`.

use winnow_args::{Args, ErrorKind};

#[derive(Args, Debug, PartialEq)]
struct Cli {
    /// Every value, `None` where the flag is never given.
    #[arg(long, delimiter = ',')]
    features: Option<Vec<String>>,
    /// Each occurrence's values, apart.
    #[arg(long, values = 1..=3)]
    point: Vec<Vec<i32>>,
    /// Each occurrence's value, or `None` for a bare one.
    #[arg(short = 'D', long)]
    define: Vec<Option<String>>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional)]
    rest: Option<Vec<String>>,
}

fn parse(line: &[&str]) -> Cli {
    Cli::parse_from(line).unwrap()
}

#[test]
fn absent_is_none_and_given_is_some() {
    let cli = parse(&[]);
    assert_eq!((cli.features, cli.rest), (None, None));
    let cli = parse(&["--features", "a,b", "--features=c", "x"]);
    assert_eq!(cli.features.unwrap(), ["a", "b", "c"]);
    assert_eq!(cli.rest.unwrap(), ["x"]);
    // An empty value is still given.
    assert_eq!(parse(&["--features="]).features, Some(vec![String::new()]));
}

#[test]
fn each_occurrence_is_a_group() {
    let cli = parse(&["--point", "1", "2", "--point", "3", "-v", "--point=4"]);
    assert_eq!(cli.point, [vec![1, 2], vec![3], vec![4]]);
    assert!(cli.verbose);
    assert!(parse(&[]).point.is_empty());
}

#[test]
fn each_occurrence_has_a_value_or_none() {
    let cli = parse(&["-D", "-Dx", "--define", "y", "--define", "-v"]);
    assert_eq!(cli.define, [None, Some("x".into()), Some("y".into()), None]);
    assert!(cli.verbose);
}

#[test]
fn a_bad_value_in_a_group_is_an_error() {
    let e = Cli::parse_from(["--point", "1", "x"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
}

#[derive(Args, Debug, PartialEq)]
struct Filled {
    #[arg(long, delimiter = ',', env = "WA_TEST_LISTS_UNSET", default = "a,b")]
    tags: Option<Vec<String>>,
    #[arg(long, required_if("--tags", "b"))]
    level: Option<u8>,
}

#[test]
fn defaults_fill_a_list_and_rules_read_it() {
    let filled = Filled::parse_from([""; 0]).unwrap();
    assert_eq!(filled.tags.unwrap(), ["a", "b"]);
    let filled = Filled::parse_from(["--tags", "c"]).unwrap();
    assert_eq!(filled.tags.unwrap(), ["c"]);
    // A value inside the list is a condition, as in a `Vec<T>`.
    let e = Filled::parse_from(["--tags", "c,b"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
}

#[cfg(feature = "help-text")]
#[test]
fn help_marks_a_bare_occurrence() {
    let help = winnow_args::help::render(Cli::HELP, &["cli"], false);
    assert!(help.contains("--define [<DEFINE>]"), "{help}");
}

/// `Vec<u8>` is one value, its bytes, as it was before these shapes.
#[derive(Args, Debug, PartialEq)]
struct Bytes {
    #[arg(long)]
    raw: Option<Vec<u8>>,
    #[arg(long)]
    each: Vec<Vec<u8>>,
    #[arg(positional)]
    rest: Vec<Vec<u8>>,
}

#[test]
fn bytes_are_a_value() {
    let bytes = Bytes::parse_from(["--raw", "abc", "--each", "x", "--each=y", "z"]).unwrap();
    assert_eq!(bytes.raw.as_deref(), Some(&b"abc"[..]));
    assert_eq!(bytes.each, [b"x".to_vec(), b"y".to_vec()]);
    assert_eq!(bytes.rest, [b"z".to_vec()]);
}

fn some_tags() -> Vec<String> {
    vec!["x".into()]
}

#[derive(Args, Debug, PartialEq)]
struct Made {
    #[arg(long)]
    base: Option<String>,
    /// Given its `Vec`, as an `Option<T>` field's function is given `T`.
    #[arg(long, default_fn = some_tags)]
    tags: Option<Vec<String>>,
    #[arg(long, default_from(base, |base: &Option<String>| base.iter().cloned().collect()))]
    from_base: Option<Vec<String>>,
}

#[test]
fn defaults_make_the_inner_value() {
    let made = Made::parse_from(["--base", "b"]).unwrap();
    assert_eq!(made.tags.unwrap(), ["x"]);
    assert_eq!(made.from_base.unwrap(), ["b"]);
}

#[derive(Args, Debug, PartialEq)]
struct GroupDefaults {
    /// All the defaults are one occurrence's values.
    #[arg(long, values = 1..=3, default = "1", default = "2")]
    point: Vec<Vec<u16>>,
    #[arg(long, delimiter = ',', default = "3,4")]
    pair: Vec<Vec<u16>>,
}

#[test]
fn defaults_are_one_group() {
    let defaults = GroupDefaults::parse_from([""; 0]).unwrap();
    assert_eq!(defaults.point, [vec![1, 2]]);
    assert_eq!(defaults.pair, [vec![3, 4]]);
}
