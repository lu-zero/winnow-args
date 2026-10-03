//! `keep_equals`: a short option's attached `=` is part of the value, as GNU
//! ld reads `-L=dir` (a directory under the sysroot).

use winnow::stream::BStr;
use winnow_args::{Args, Occurrence};

#[derive(Args, Debug)]
struct Plain {
    #[arg(short = 'L')]
    dirs: Vec<String>,
}

#[derive(Args, Debug)]
struct Kept {
    #[arg(short = 'L', keep_equals)]
    dirs: Vec<String>,
}

#[derive(Occurrence, Debug, PartialEq)]
#[arg(keep_equals)]
enum Item {
    #[arg(short = 'L', long = "library-path")]
    Dir(String),
}

#[derive(Args, Debug)]
#[arg(long_only)]
struct Ld {
    #[arg(sequence)]
    items: Vec<Item>,
}

fn words<'a>(line: &'a [&'a str]) -> Vec<&'a BStr> {
    line.iter().map(BStr::new).collect()
}

#[test]
fn an_attached_equals_is_dropped_by_default_and_kept_on_request() {
    let line = ["-L=foo", "-Lbar", "-L", "=baz"];
    assert_eq!(
        Plain::parse_words(&words(&line)).unwrap().dirs,
        ["foo", "bar", "=baz"]
    );
    assert_eq!(
        Kept::parse_words(&words(&line)).unwrap().dirs,
        ["=foo", "bar", "=baz"]
    );
}

#[test]
fn a_long_option_s_equals_still_separates() {
    let line = ["-L=foo", "--library-path=bar", "-library-path=baz"];
    assert_eq!(
        Ld::parse_words(&words(&line)).unwrap().items,
        [
            Item::Dir("=foo".into()),
            Item::Dir("bar".into()),
            Item::Dir("baz".into())
        ]
    );
}
