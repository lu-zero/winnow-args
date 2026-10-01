//! `Occurrence` variants for a linker's whole command line: value options,
//! `skip` variants, unknown flags kept in order, and `Spanned` values.
#![cfg(feature = "derive")]

use winnow::stream::BStr;
use winnow_args::value::Spanned;
use winnow_args::{Args, Error, ErrorKind, Occurrence};

#[derive(Occurrence, Debug, PartialEq)]
#[arg(allow_hyphen_values)]
enum Item {
    #[arg(short = 'o', long = "output", two_dashes)]
    Output(String),
    #[arg(long)]
    PluginOpt(String),
    /// `--build-id` alone, or `--build-id=sha1`; never the next word.
    #[arg(long, require_equals, default_missing = "\u{0}")]
    BuildId(String),
    #[arg(short = 'z')]
    Z(Spanned<String>),
    /// Built from a `-z` keyword, never parsed.
    #[arg(skip)]
    ZNow,
    #[arg(unknown)]
    Unknown(Spanned<String>),
    #[arg(positional)]
    Input(Spanned<String>),
}

#[derive(Args, Debug)]
#[arg(long_only)]
struct Ld {
    #[arg(sequence, unknown)]
    opts: Vec<Item>,
}

fn parse(line: &[&str]) -> Result<Vec<Item>, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    Ld::parse_from(&words).map(|ld| ld.opts)
}

fn spanned(value: &str, offset: usize, attached: bool) -> Spanned<String> {
    Spanned {
        value: value.to_owned(),
        offset,
        attached,
    }
}

#[test]
fn values_may_look_like_flags() {
    assert_eq!(
        parse(&["-o", "-weird", "-plugin-opt", "-pass-through=-lgcc"]).unwrap(),
        [
            Item::Output("-weird".into()),
            Item::PluginOpt("-pass-through=-lgcc".into())
        ]
    );
}

#[test]
fn require_equals_with_a_default() {
    assert_eq!(
        parse(&["--build-id", "a.o", "--build-id=sha1"]).unwrap(),
        [
            Item::BuildId("\u{0}".into()),
            Item::Input(spanned("a.o", 11, false)),
            Item::BuildId("sha1".into()),
        ]
    );
}

#[test]
fn spanned_values_know_if_they_were_attached() {
    assert_eq!(
        parse(&["-zfoo", "-z", "bar"]).unwrap(),
        [
            Item::Z(spanned("foo", 0, true)),
            Item::Z(spanned("bar", 6, false))
        ]
    );
}

#[test]
fn unknown_flags_keep_their_place() {
    assert_eq!(
        parse(&["a.o", "--lto-O3", "-abcdefg", "b.o"]).unwrap(),
        [
            Item::Input(spanned("a.o", 0, false)),
            Item::Unknown(spanned("--lto-O3", 4, false)),
            Item::Unknown(spanned("-abcdefg", 13, false)),
            Item::Input(spanned("b.o", 22, false)),
        ]
    );
}

#[test]
fn skip_variants_are_not_spelled() {
    // `ZNow` is only ever built by the program.
    let built = Item::ZNow;
    assert_eq!(
        parse(&["--z-now"]).unwrap(),
        [Item::Unknown(spanned("--z-now", 0, false))]
    );
    assert_ne!(parse(&["--z-now"]).unwrap()[0], built);
    // An `Occurrence` without an unknown variant still rejects.
    #[derive(Occurrence, Debug)]
    enum Strict {
        #[arg(long)]
        Shared,
    }
    #[derive(Args, Debug)]
    struct S {
        #[arg(sequence, unknown)]
        opts: Vec<Strict>,
    }
    let words: Vec<&BStr> = ["--nope"].iter().map(BStr::new).collect();
    assert_eq!(
        S::parse_from(&words).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}
