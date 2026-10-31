//! `#[arg(plus_options)]`: bash's `+` options (`set +e`, `declare +x`),
//! lexed natively rather than rewritten before parsing.

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind};

/// A cut-down `set`.
#[derive(Args, Debug, PartialEq, Default)]
#[arg(plus_options, disable_help_flag)]
struct Set {
    #[arg(short = 'e', plus = 'e')]
    errexit: Option<bool>,
    #[arg(short = 'u', plus = 'u')]
    nounset: Option<bool>,
    #[arg(short = 'x', plus = 'x')]
    xtrace: Option<bool>,
    /// `-o NAME`; a bare `-o` lists the options.
    #[arg(short = 'o', default_missing = "")]
    enable: Vec<String>,
    /// `+o NAME`; a bare `+o` lists them as commands.
    #[arg(plus = 'o', default_missing = "")]
    disable: Vec<String>,
    #[arg(positional, double_dash = "preserve", stop_flags)]
    args: Vec<String>,
    /// Filled in after parsing, as brush does for declaration builtins.
    #[arg(skip)]
    extra: Vec<u32>,
}

fn parse(line: &[&str]) -> Result<Set, Error> {
    Set::parse_from(line)
}

fn ok(line: &[&str]) -> Set {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn minus_sets_and_plus_clears() {
    let s = ok(&["-eu", "+x"]);
    assert_eq!(
        (s.errexit, s.nounset, s.xtrace),
        (Some(true), Some(true), Some(false))
    );
    let s = ok(&["+eu"]);
    assert_eq!(
        (s.errexit, s.nounset, s.xtrace),
        (Some(false), Some(false), None)
    );
    // The last spelling wins.
    assert_eq!(ok(&["-e", "+e"]).errexit, Some(false));
    assert_eq!(ok(&["+e", "-e"]).errexit, Some(true));
}

#[test]
fn value_letters_in_both_bundles() {
    let s = ok(&["-euxo", "pipefail", "+o", "noglob"]);
    assert_eq!(s.enable, strings(&["pipefail"]));
    assert_eq!(s.disable, strings(&["noglob"]));
    assert_eq!((s.errexit, s.xtrace), (Some(true), Some(true)));
    // Bare `-o`/`+o` list the options.
    assert_eq!(ok(&["-o"]).enable, strings(&[""]));
    assert_eq!(ok(&["+o"]).disable, strings(&[""]));
}

#[test]
fn operands_end_both_kinds_of_bundle() {
    let s = ok(&["-e", "--", "+x", "-u"]);
    assert_eq!((s.errexit, s.xtrace, s.nounset), (Some(true), None, None));
    assert_eq!(s.args, strings(&["--", "+x", "-u"]));
    let s = ok(&["a", "+e"]);
    assert_eq!((s.errexit, s.args), (None, strings(&["a", "+e"])));
    // A lone `+` is a word.
    assert_eq!(ok(&["+"]).args, strings(&["+"]));
    assert!(ok(&["-e"]).extra.is_empty());
}

#[test]
fn an_unknown_plus_letter_is_reported_with_its_plus() {
    let e = parse(&["+q"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("+q")));
}

/// Without `plus_options`, `+` words are words.
#[derive(Args, Debug)]
struct Plain {
    #[arg(short)]
    e: bool,
    #[arg(positional)]
    rest: Vec<String>,
}

#[test]
fn plus_words_are_words_elsewhere() {
    let words = [BStr::new("+e")];
    let p = Plain::parse_words(&words).unwrap();
    assert_eq!((p.e, p.rest), (false, strings(&["+e"])));
}

/// `declare`'s attributes, where every `-x` and `+x` counts.
#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct Declare {
    #[arg(short = 'x', plus = 'x')]
    export: Vec<bool>,
    #[arg(short = 'r', plus = 'r')]
    readonly: Option<bool>,
    #[arg(positional)]
    names: Vec<String>,
}

#[test]
fn a_bool_list_keeps_every_spelling_in_order() {
    let d = Declare::parse_from(["-x", "+x", "-xr", "+x", "v"]).unwrap();
    assert_eq!(d.export, [true, false, true, false]);
    assert_eq!((d.readonly, d.names), (Some(true), strings(&["v"])));
    let d = Declare::parse_from(["+rx", "v"]).unwrap();
    assert_eq!((d.export, d.readonly), (vec![false], Some(false)));
    assert!(Declare::parse_from(["v"]).unwrap().export.is_empty());
}

/// `set -o NAME` and `set +o NAME` in one field, in order, as bash applies
/// them.
#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct SetOrdered {
    #[arg(short = 'o', plus = 'o', default_missing = "")]
    options: Vec<(bool, String)>,
    #[arg(positional)]
    args: Vec<String>,
}

fn on(name: &str) -> (bool, String) {
    (true, name.into())
}

fn off(name: &str) -> (bool, String) {
    (false, name.into())
}

#[test]
fn set_o_and_plus_o_keep_their_order() {
    let set = SetOrdered::parse_from(["-o", "noglob", "+o", "noglob"]).unwrap();
    assert_eq!(set.options, [on("noglob"), off("noglob")]);
    let set = SetOrdered::parse_from(["-o", "noglob", "-o", "noclobber", "+o", "noglob"]).unwrap();
    assert_eq!(set.options, [on("noglob"), on("noclobber"), off("noglob")]);
    let set = SetOrdered::parse_from(["+onoglob", "-onoglob", "x"]).unwrap();
    assert_eq!(
        (&set.options[..], &set.args[..]),
        (&[off("noglob"), on("noglob")][..], &["x".to_owned()][..])
    );
    // A bare `-o` or `+o` lists the options.
    let set = SetOrdered::parse_from(["+o"]).unwrap();
    assert_eq!(set.options, [off("")]);
}

/// The single letters and `-o NAME` of `set`, all in one order: `-o errexit
/// +e` leaves it off, as in bash.
#[derive(winnow_args::Occurrence, Debug, PartialEq)]
enum SetItem {
    #[arg(short = 'e')]
    Errexit,
    #[arg(plus = 'e')]
    NoErrexit,
    #[arg(short = 'o', default_missing = "")]
    Enable(String),
    #[arg(plus = 'o', default_missing = "")]
    Disable(String),
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct SetSequence {
    #[arg(short = 'x', plus = 'x')]
    xtrace: Option<bool>,
    #[arg(sequence)]
    items: Vec<SetItem>,
}

#[test]
fn occurrence_variants_take_plus_spellings() {
    let set = SetSequence::parse_from(["-o", "errexit", "+e", "+x", "+o", "noglob", "-e"]).unwrap();
    assert_eq!(set.xtrace, Some(false));
    assert_eq!(
        set.items,
        [
            SetItem::Enable("errexit".into()),
            SetItem::NoErrexit,
            SetItem::Disable("noglob".into()),
            SetItem::Errexit,
        ]
    );
    // `+e` bundled with the struct's own `+x`.
    let set = SetSequence::parse_from(["+ex"]).unwrap();
    assert_eq!(
        (set.xtrace, &set.items[..]),
        (Some(false), &[SetItem::NoErrexit][..])
    );
}

#[cfg(feature = "help-text")]
#[test]
fn help_lists_plus_variants() {
    let help = winnow_args::help::render(SetSequence::HELP, &["set"], false);
    assert!(help.contains("+e"), "{help}");
    assert!(help.contains("+o [<VALUE>]"), "{help}");
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct Features {
    #[arg(short = 'F', plus = 'F', delimiter = ',', default = "std,alloc")]
    features: Vec<(bool, String)>,
}

#[test]
fn a_delimiter_gives_each_piece_its_sign() {
    let features = Features::parse_from(["-Fa,b", "+F", "b,c", "-Fd"]).unwrap();
    assert_eq!(
        features.features,
        [on("a"), on("b"), off("b"), off("c"), on("d")]
    );
    // The default's pieces are turned on.
    let features = Features::parse_from([""; 0]).unwrap();
    assert_eq!(features.features, [on("std"), on("alloc")]);
}

/// One variant an option: it holds the sign of `-x` (`true`) and `+x`
/// (`false`), as brush's `set` reads them.
#[derive(winnow_args::Occurrence, Debug, PartialEq)]
enum SetOption {
    /// Exit on nonzero command exit.
    #[arg(short = 'e', plus = 'e')]
    Errexit(bool),
    /// Disable filename globbing.
    #[arg(short = 'f', plus = 'f')]
    Noglob(bool),
    /// Set or unset the option NAME; alone, list them. As bash, only the
    /// next word is its value: `-oe` is a bare `-o`, then `-e`.
    #[arg(
        short = 'o',
        plus = 'o',
        value_name = "OPT",
        default_missing = "\u{0}",
        detached_only
    )]
    Named(bool, String),
    /// The same, its sign and value as one tuple.
    #[arg(short = 'O', plus = 'O')]
    Paired((bool, String)),
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct SetSigned {
    #[arg(sequence)]
    options: Vec<SetOption>,
    #[arg(positional, double_dash = "preserve", stop_flags)]
    args: Vec<String>,
}

fn items(line: &[&str]) -> Vec<SetOption> {
    SetSigned::parse_from(line).unwrap().options
}

fn named(on: bool, name: &str) -> SetOption {
    SetOption::Named(on, name.into())
}

#[test]
fn a_variant_holds_the_sign() {
    use SetOption::*;
    // bash's answers come from applying these in order.
    assert_eq!(
        items(&["-o", "noglob", "+o", "noglob"]),
        [named(true, "noglob"), named(false, "noglob")]
    );
    assert_eq!(
        items(&["-o", "noglob", "-o", "noclobber", "+o", "noglob"]),
        [
            named(true, "noglob"),
            named(true, "noclobber"),
            named(false, "noglob")
        ]
    );
    assert_eq!(
        items(&["+o", "noglob", "-o", "noglob"]),
        [named(false, "noglob"), named(true, "noglob")]
    );
    // Letters and names in one order: `-o noglob +f` leaves it off.
    assert_eq!(
        items(&["-o", "noglob", "+f"]),
        [named(true, "noglob"), Noglob(false)]
    );
    // A bundle whose last letter takes the next word.
    assert_eq!(
        items(&["-eo", "pipefail"]),
        [Errexit(true), named(true, "pipefail")]
    );
    assert_eq!(
        items(&["+eo", "pipefail"]),
        [Errexit(false), named(false, "pipefail")]
    );
    // A bare `-o` after a name: set it, then list.
    assert_eq!(
        items(&["-o", "noglob", "-o"]),
        [named(true, "noglob"), named(true, "\0")]
    );
    // What follows a bad name is still there, in order, for the program to stop at.
    assert_eq!(
        items(&["-o", "nosuch", "-f"]),
        [named(true, "nosuch"), Noglob(true)]
    );
    assert_eq!(items(&["+O", "x"]), [Paired((false, "x".into()))]);
}

#[cfg(feature = "help-text")]
#[test]
fn help_has_one_row_a_signed_variant() {
    let help = winnow_args::help::render(SetSigned::HELP, &["set"], false);
    assert!(help.contains("-e, +e"), "{help}");
    assert!(help.contains("Exit on nonzero command exit."), "{help}");
    assert!(help.contains("-o, +o [<OPT>]"), "{help}");
}

#[test]
fn a_detached_only_value_is_never_the_rest_of_the_bundle() {
    use SetOption::*;
    assert_eq!(items(&["-oe"]), [named(true, "\0"), Errexit(true)]);
    assert_eq!(items(&["+oe"]), [named(false, "\0"), Errexit(false)]);
    // Last in its bundle, or alone, it takes the next word.
    assert_eq!(
        items(&["-eo", "pipefail"]),
        [Errexit(true), named(true, "pipefail")]
    );
    assert_eq!(items(&["-o", "noglob"]), [named(true, "noglob")]);
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, disable_help_flag)]
struct Detached {
    #[arg(short = 'o', default_missing = "", detached_only)]
    option: Vec<String>,
    #[arg(short = 'e')]
    errexit: bool,
    #[arg(short = 'n', detached_only)]
    count: Option<u8>,
}

#[test]
fn a_detached_only_field() {
    let detached = Detached::parse_from(["-oe", "-o", "x"]).unwrap();
    assert_eq!(
        (&detached.option[..], detached.errexit),
        (&["".to_owned(), "x".to_owned()][..], true)
    );
    // With no `default_missing`, a bare one is a missing value.
    let error = Detached::parse_from(["-ne"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingValue);
    assert_eq!(Detached::parse_from(["-en", "3"]).unwrap().count, Some(3));
}
