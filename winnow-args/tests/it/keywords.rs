//! `#[arg(keywords)]`: ld's `-z` keywords as a vocabulary of their own.
//! Vectors from mold's `-z` tests.

use winnow_args::value::CInt;
use winnow_args::{Args, Error, ErrorKind};

/// The `-z` keywords: each `-z WORD` is `--WORD` here.
#[derive(Args, Debug, Default, PartialEq)]
#[arg(unknown_flags = "value")]
struct Z {
    /// `-z now`, `-z lazy`.
    #[arg(long, negate = "lazy")]
    now: bool,
    /// `-z relro`, `-z norelro`.
    #[arg(long, negate = "norelro")]
    relro: bool,
    #[arg(long)]
    max_page_size: Option<CInt<u64>>,
    #[arg(long)]
    noexecstack: bool,
    /// Keywords the linker does not know: warned about, not fatal.
    #[arg(positional)]
    unknown: Vec<String>,
}

#[derive(Args, Debug)]
struct Ld {
    #[arg(short = 'z', value_name = "KEYWORD", keywords)]
    z: Z,
    #[arg(short)]
    output: Option<String>,
    #[arg(positional)]
    inputs: Vec<String>,
}

fn parse(line: &[&str]) -> Result<Ld, Error> {
    Ld::parse_from(line)
}

fn z(line: &[&str]) -> Z {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}")).z
}

#[test]
fn keywords_attached_or_not() {
    let z = z(&["-z", "now", "-zrelro", "-z", "noexecstack"]);
    assert!(z.now && z.relro && z.noexecstack);
}

#[test]
fn key_value_keywords() {
    assert_eq!(
        z(&["-z", "max-page-size=0x1000"]).max_page_size,
        Some(CInt(0x1000))
    );
}

#[test]
fn pairs_are_switches_and_the_last_wins() {
    assert!(!z(&["-z", "now", "-z", "lazy"]).now);
    assert!(z(&["-z", "lazy", "-z", "now"]).now);
    assert!(!z(&["-z", "relro", "-z", "norelro"]).relro);
}

#[test]
fn unknown_keywords_are_collected_to_warn_about() {
    let z = z(&["-z", "separate-code", "-z", "now"]);
    assert!(z.now);
    assert_eq!(z.unknown, ["--separate-code"]);
}

#[test]
fn a_bad_value_names_the_keyword() {
    let e = parse(&["-z", "max-page-size=big"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
    assert_eq!(e.token(), Some("-z"));
    assert_eq!(e.value(), Some("max-page-size"));
}

#[test]
fn no_keywords_is_the_default_vocabulary() {
    let ld = parse(&["-o", "a.out", "a.o"]).unwrap();
    assert_eq!(ld.z, Z::default());
    assert_eq!(ld.output.as_deref(), Some("a.out"));
    assert_eq!(ld.inputs, ["a.o"]);
}
