//! `Occurrence` variants for a linker's whole command line: value options,
//! `skip` variants, unknown flags kept in order, and `Spanned` values.

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
    Ld::try_parse_from(line).map(|ld| ld.opts)
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
    assert_eq!(built, Item::ZNow);
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

/// Grouped short flags are reported, for a linker that deprecates them.
#[derive(Occurrence, Debug, PartialEq)]
enum Grouping {
    #[arg(short = 's')]
    StripAll,
    #[arg(short = 'S')]
    StripDebug,
    #[arg(short = 'o')]
    Output(String),
    #[arg(long)]
    Shared,
    #[arg(bundle)]
    Grouped(String),
}

#[derive(Args, Debug)]
#[arg(long_only)]
struct Grouper {
    #[arg(sequence)]
    items: Vec<Grouping>,
}

#[test]
fn a_bundle_is_an_item_before_its_letters() {
    let words: Vec<&BStr> = ["-s", "-sS", "-shared", "-ofile", "-so", "x"]
        .iter()
        .map(BStr::new)
        .collect();
    assert_eq!(
        Grouper::parse_from(&words).unwrap().items,
        [
            Grouping::StripAll,
            Grouping::Grouped("-sS".into()),
            Grouping::StripAll,
            Grouping::StripDebug,
            Grouping::Shared,
            Grouping::Output("file".into()),
            Grouping::Grouped("-so".into()),
            Grouping::StripAll,
            Grouping::Output("x".into()),
        ]
    );
}
