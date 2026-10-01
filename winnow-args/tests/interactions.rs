//! Features in pairs: flatten, sequence, `+` options, globals and the built-in help flags.
#![cfg(feature = "derive")]

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, Occurrence, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    T::parse_from(&words)
}

fn kind<T: Args + std::fmt::Debug>(line: &[&str]) -> ErrorKind {
    parse::<T>(line).unwrap_err().kind()
}

#[derive(Args, Debug, Default, PartialEq)]
struct Common {
    #[arg(short)]
    verbose: bool,
    #[arg(short)]
    name: Option<String>,
}

#[derive(Args, Debug)]
#[arg(plus_options)]
struct Plus {
    #[arg(short = 'e', plus = 'e')]
    e: Option<bool>,
    #[arg(flatten)]
    common: Common,
}

#[test]
fn a_plus_word_never_binds_a_flattened_dash_flag() {
    assert_eq!(kind::<Plus>(&["+v"]), ErrorKind::UnknownFlag);
    assert_eq!(kind::<Plus>(&["+n", "x"]), ErrorKind::UnknownFlag);
    let ok: Plus = parse(&["+e", "-v"]).unwrap();
    assert_eq!((ok.e, ok.common.verbose), (Some(false), true));
}

/// Flattened and sequence flags named like the built-in help and version.
#[derive(Args, Debug, Default)]
struct Human {
    #[arg(short = 'h', long)]
    human: bool,
    #[arg(short = 'V')]
    verify: bool,
}

#[derive(Args, Debug)]
#[arg(version = "1")]
struct Df {
    #[arg(flatten)]
    human: Human,
}

#[derive(Occurrence, Debug, PartialEq)]
enum LdItem {
    #[arg(short = 'h')]
    Soname(String),
    #[arg(short = 's')]
    Strip,
    #[arg(short = 'S')]
    StripDebug,
    #[arg(bundle)]
    Grouped(String),
}

#[derive(Args, Debug)]
struct Ld {
    #[arg(short)]
    verbose: bool,
    #[arg(sequence)]
    items: Vec<LdItem>,
}

#[test]
fn built_in_help_and_version_yield_to_flattened_and_sequence_flags() {
    let df: Df = parse(&["-h", "-V"]).unwrap();
    assert!(df.human.human && df.human.verify);
    assert_eq!(kind::<Df>(&["--help"]), ErrorKind::HelpRequested);
    assert_eq!(kind::<Df>(&["--version"]), ErrorKind::VersionRequested);
    let ld: Ld = parse(&["-h", "libx.so"]).unwrap();
    assert_eq!(ld.items, [LdItem::Soname("libx.so".into())]);
    assert_eq!(kind::<Ld>(&["--help"]), ErrorKind::HelpRequested);
}

#[test]
fn a_bundle_is_reported_whoever_takes_its_first_letter() {
    let ld: Ld = parse(&["-vS", "-s", "-vsS"]).unwrap();
    assert!(ld.verbose);
    assert_eq!(
        ld.items,
        [
            LdItem::Grouped("-vS".into()),
            LdItem::StripDebug,
            LdItem::Strip,
            LdItem::Grouped("-vsS".into()),
            LdItem::Strip,
            LdItem::StripDebug,
        ]
    );
}

#[derive(Args, Debug, Default)]
struct Libs {
    #[arg(short = 'l', prefix)]
    libs: Vec<String>,
}

#[derive(Args, Debug)]
#[arg(long_only)]
struct Linker {
    #[arg(long)]
    lib64: bool,
    #[arg(flatten)]
    libs: Libs,
}

#[test]
fn a_flattened_prefix_letter_wins_over_a_single_dash_long_name() {
    let l: Linker = parse(&["-lib64", "--lib64"]).unwrap();
    assert!(l.lib64);
    assert_eq!(l.libs.libs, ["ib64"]);
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, unknown_flags = "value")]
struct Lenient {
    #[arg(short = 'e', plus = 'e')]
    e: Option<bool>,
    #[arg(short)]
    v: bool,
    #[arg(positional)]
    rest: Vec<String>,
}

#[test]
fn a_tristate_letter_takes_no_value_in_a_lenient_bundle() {
    // `x` is unknown: the whole word is a value, nothing bound.
    let l: Lenient = parse(&["-ex", "-evx"]).unwrap();
    assert_eq!((l.e, l.v), (None, false));
    assert_eq!(l.rest, ["-ex", "-evx"]);
}

#[derive(Args, Debug)]
#[arg(plus_options)]
struct Top {
    #[arg(short = 'e', plus = 'e', global)]
    e: Option<bool>,
    #[arg(short, global)]
    verbose: bool,
    #[arg(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Cmd {
    /// Show status.
    ///
    /// More about status.
    Status,
    Remote(Remote),
    Run(Run),
}

#[derive(Subcommand, Debug, PartialEq)]
enum Remote {
    List,
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options)]
struct Run {
    #[arg(short)]
    quick: bool,
}

#[test]
fn global_plus_flags_reach_a_subcommand() {
    assert_eq!(parse::<Top>(&["run", "+e"]).unwrap().e, Some(false));
    assert_eq!(parse::<Top>(&["run", "-e"]).unwrap().e, Some(true));
}

#[test]
fn globals_come_before_a_nested_subcommand_s_name() {
    let top: Top = parse(&["remote", "-v", "list"]).unwrap();
    assert!(top.verbose);
    assert_eq!(top.cmd, Cmd::Remote(Remote::List));
}

#[test]
fn a_unit_subcommand_has_help() {
    let e = parse::<Top>(&["status", "--help"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested);
    if cfg!(feature = "help-text") {
        assert!(e.render_help("top").unwrap().contains("More about status."));
    }
    assert_eq!(kind::<Top>(&["status", "-h"]), ErrorKind::HelpRequested);
}

/// A struct with a help flag of its own.
#[derive(Args, Debug)]
struct OwnHelp {
    /// My own help switch.
    #[arg(long)]
    help: bool,
}

#[test]
fn a_declared_help_flag_leaves_no_built_in_row() {
    assert!(parse::<OwnHelp>(&["--help"]).unwrap().help);
    let text = parse::<OwnHelp>(&["-h"])
        .unwrap_err()
        .render_help("x")
        .unwrap();
    assert_eq!(text.matches("      --help").count(), 1, "{text}");
}
