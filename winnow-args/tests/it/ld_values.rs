//! ld's values: C-syntax numbers, `key=value`,
//! several words per occurrence, `-l:file`.

use winnow_args::value::{CInt, KeyValue};
use winnow_args::{Args, Error, ErrorKind};

#[derive(Args, Debug, Default)]
#[arg(long_only)]
struct Ld {
    #[arg(long)]
    image_base: Option<CInt<u64>>,
    #[arg(long)]
    section_start: Vec<KeyValue<String, CInt<u64>>>,
    #[arg(long)]
    defsym: Vec<KeyValue<String, String>>,
    #[arg(short = 'z')]
    z: Vec<String>,
    #[arg(long = "platform_version", values = 3)]
    platform_version: Vec<String>,
    #[arg(short = 'l', long, prefix)]
    library: Vec<String>,
    #[arg(positional)]
    inputs: Vec<String>,
}

fn parse(line: &[&str]) -> Result<Ld, Error> {
    Ld::try_parse_from(line)
}

#[test]
fn numbers_in_c_syntax() {
    let ld = parse(&["--image-base=0x400000"]).unwrap();
    assert_eq!(ld.image_base, Some(CInt(0x40_0000)));
    assert_eq!(
        parse(&["-image-base", "010"]).unwrap().image_base,
        Some(CInt(8))
    );
    let e = parse(&["--image-base=0xZZ"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
}

#[test]
fn key_value_splits_at_the_first_equals() {
    let ld = parse(&["--section-start=.text=0x1000", "--defsym=sym=a=b"]).unwrap();
    assert_eq!(ld.section_start[0].key, ".text");
    assert_eq!(ld.section_start[0].value, CInt(0x1000));
    assert_eq!(
        (ld.defsym[0].key.as_str(), ld.defsym[0].value.as_str()),
        ("sym", "a=b")
    );
    assert_eq!(
        parse(&["--defsym", "x"]).unwrap_err().kind(),
        ErrorKind::InvalidValue
    );
}

#[test]
fn z_keywords_attached_or_not() {
    let ld = parse(&["-z", "now", "-zrelro", "-z", "max-page-size=4096"]).unwrap();
    assert_eq!(ld.z, ["now", "relro", "max-page-size=4096"]);
}

#[test]
fn several_words_per_occurrence() {
    let ld = parse(&["-platform_version", "macos", "11.0", "12.0", "a.o"]).unwrap();
    assert_eq!(ld.platform_version, ["macos", "11.0", "12.0"]);
    assert_eq!(ld.inputs, ["a.o"]);
    let e = parse(&["-platform_version", "macos", "11.0"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingValue);
}

#[test]
fn a_colon_library_is_a_file_name() {
    assert_eq!(
        parse(&["-l:libfoo.a", "-lm"]).unwrap().library,
        [":libfoo.a", "m"]
    );
}
