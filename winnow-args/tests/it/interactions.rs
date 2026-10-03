//! Features in pairs: flatten, sequence, `+` options, globals and the built-in help flags.

use winnow_args::{Args, Error, ErrorKind, Occurrence, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::parse_from(line)
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

#[derive(Occurrence, Debug, PartialEq)]
enum Step {
    #[arg(short = 'n')]
    DryRun,
}

/// Lenient, with a sequence: the built-in `-h` is tried after the enum's
/// flags, and is still a known letter of a bundle.
#[derive(Args, Debug)]
#[arg(unknown_flags = "value")]
struct LenientSequence {
    #[arg(short)]
    verbose: bool,
    #[arg(sequence)]
    items: Vec<Step>,
    #[arg(positional)]
    files: Vec<String>,
}

#[test]
fn a_late_built_in_letter_is_known_in_a_bundle() {
    assert_eq!(kind::<LenientSequence>(&["-vh"]), ErrorKind::HelpRequested);
    assert_eq!(kind::<LenientSequence>(&["-hv"]), ErrorKind::HelpRequested);
    let ok: LenientSequence = parse(&["-nv"]).unwrap();
    assert!(ok.verbose);
    assert_eq!(ok.items, [Step::DryRun]);
    // A letter nobody knows still makes the word a value.
    assert_eq!(parse::<LenientSequence>(&["-vQ"]).unwrap().files, ["-vQ"]);
}

/// Lenient, flattening: no flattened `-h`, so the built-in one answers.
#[derive(Args, Debug)]
#[arg(unknown_flags = "value")]
struct LenientFlatten {
    #[arg(flatten)]
    common: Common,
    #[arg(positional)]
    files: Vec<String>,
}

#[test]
fn a_built_in_letter_after_a_flattened_one_asks_for_help() {
    assert_eq!(kind::<LenientFlatten>(&["-vh"]), ErrorKind::HelpRequested);
    assert_eq!(kind::<LenientFlatten>(&["-hv"]), ErrorKind::HelpRequested);
}

#[derive(Args, Debug, PartialEq)]
struct Formats {
    #[arg(long, overrides("--yaml"))]
    json: Option<String>,
    #[arg(long, required)]
    yaml: Option<String>,
}

#[test]
fn the_flag_that_overrode_a_required_one_satisfies_it() {
    let f: Formats = parse(&["--yaml=b", "--json=a"]).unwrap();
    assert_eq!((f.json.as_deref(), f.yaml), (Some("a"), None));
    assert_eq!(kind::<Formats>(&[]), ErrorKind::MissingRequired);
}

/// A rule may name a flag by any of its letters.
#[derive(Args, Debug)]
struct Spelled {
    #[arg(long, requires("-L"))]
    both: bool,
    #[arg(short = 'l', short = 'L', long)]
    list: bool,
}

#[test]
fn a_rule_resolves_a_second_short() {
    assert_eq!(kind::<Spelled>(&["--both"]), ErrorKind::MissingRequired);
    assert!(parse::<Spelled>(&["--both", "-L"]).unwrap().list);
}

#[derive(Args, Debug)]
struct Aliased {
    #[arg(long = "own", alias = "shared")]
    own: bool,
    #[arg(short = 'y', short = 'x')]
    two: bool,
}

#[derive(Args, Debug)]
struct Plainly {
    #[arg(long)]
    shared: bool,
}

#[derive(Args, Debug)]
struct SecondShort {
    #[arg(short = 'x')]
    x: bool,
}

#[test]
fn the_clash_check_sees_aliases_and_second_shorts() {
    use winnow_args::help::items_clash;
    assert!(items_clash(&[Aliased::HELP.items, Plainly::HELP.items]));
    assert!(items_clash(&[SecondShort::HELP.items, Aliased::HELP.items]));
    assert!(!items_clash(&[
        Plainly::HELP.items,
        SecondShort::HELP.items
    ]));
}

#[test]
fn any_width_renders() {
    for width in [1, 5, usize::MAX - 3, usize::MAX] {
        let page = winnow_args::help::render_width(Formats::HELP, &["x"], false, width);
        assert!(page.contains("--json"), "{width}");
    }
}
