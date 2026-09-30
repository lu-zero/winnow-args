//! `#[arg(plus_options)]`: bash's `+` options (`set +e`, `declare +x`),
//! lexed natively rather than rewritten before parsing.
#![cfg(feature = "derive")]

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
    #[arg(short = 'o', value_optional, default_missing = "")]
    enable: Vec<String>,
    /// `+o NAME`; a bare `+o` lists them as commands.
    #[arg(plus = 'o', value_optional, default_missing = "")]
    disable: Vec<String>,
    #[arg(positional, double_dash = "preserve", stop_flags)]
    args: Vec<String>,
    /// Filled in after parsing, as brush does for declaration builtins.
    #[arg(skip)]
    extra: Vec<u32>,
}

fn parse(line: &[&str]) -> Result<Set, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    Set::parse_from(&words)
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
    let p = Plain::parse_from(&words).unwrap();
    assert_eq!((p.e, p.rest), (false, strings(&["+e"])));
}
