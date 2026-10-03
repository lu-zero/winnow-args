//! `#[derive(Occurrence)]` and `#[arg(sequence)]`: flags whose position is
//! their meaning, as `ld`'s.

use winnow_args::{Args, Error, ErrorKind, Occurrence};

/// What an input's handling depends on, and the inputs, in order.
#[derive(Occurrence, Debug, PartialEq)]
enum Item {
    #[arg(long)]
    AsNeeded,
    #[arg(long)]
    NoAsNeeded,
    #[arg(long)]
    WholeArchive,
    #[arg(long)]
    NoWholeArchive,
    #[arg(long = "Bstatic", alias("static", "dn", "non_shared"))]
    Bstatic,
    #[arg(long = "Bdynamic", alias("dy", "call_shared"))]
    Bdynamic,
    #[arg(long)]
    PushState,
    #[arg(long)]
    PopState,
    #[arg(short = '(', long)]
    StartGroup,
    #[arg(short = ')', long)]
    EndGroup,
    #[arg(short = 'l', long = "library", prefix)]
    Library(String),
    #[arg(positional, value_name = "FILE")]
    Input(String),
}

#[derive(Args, Debug)]
#[arg(long_only)]
struct Ld {
    #[arg(short = 'o', long, two_dashes)]
    output: Option<String>,
    #[arg(long)]
    shared: bool,
    #[arg(long)]
    lazy: bool,
    #[arg(sequence)]
    items: Vec<Item>,
}

fn parse(line: &[&str]) -> Result<Ld, Error> {
    Ld::parse_from(line)
}

fn ok(line: &[&str]) -> Ld {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn input(s: &str) -> Item {
    Item::Input(s.to_owned())
}

fn lib(s: &str) -> Item {
    Item::Library(s.to_owned())
}

#[test]
fn state_and_inputs_keep_their_order() {
    let ld = ok(&[
        "a.o",
        "--as-needed",
        "-lm",
        "b.o",
        "--no-as-needed",
        "-(",
        "c.a",
        "d.a",
        "-)",
        "-o",
        "out",
        "-Bstatic",
        "-lfoo",
        "--push-state",
        "--whole-archive",
        "e.a",
        "--pop-state",
    ]);
    assert_eq!(ld.output.as_deref(), Some("out"));
    assert!(!ld.shared);
    assert_eq!(
        ld.items,
        [
            input("a.o"),
            Item::AsNeeded,
            lib("m"),
            input("b.o"),
            Item::NoAsNeeded,
            Item::StartGroup,
            input("c.a"),
            input("d.a"),
            Item::EndGroup,
            Item::Bstatic,
            lib("foo"),
            Item::PushState,
            Item::WholeArchive,
            input("e.a"),
            Item::PopState,
        ]
    );
}

#[test]
fn single_dash_longs_reach_the_sequence() {
    let ld = ok(&["-as-needed", "-start-group", "-static", "-shared", "x.o"]);
    assert!(ld.shared);
    assert_eq!(
        ld.items,
        [
            Item::AsNeeded,
            Item::StartGroup,
            Item::Bstatic,
            input("x.o")
        ]
    );
}

#[test]
fn the_sequence_prefix_letter_takes_its_word() {
    // `-lazy` is the library `azy`, although the struct has `--lazy`.
    assert_eq!(ok(&["-lazy"]).items, [lib("azy")]);
    assert!(ok(&["--lazy"]).lazy);
    assert_eq!(ok(&["--library=z", "-l", "y"]).items, [lib("z"), lib("y")]);
}

#[test]
fn unknown_flags_are_still_unknown() {
    let e = parse(&["--wat"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnknownFlag, Some("--wat"))
    );
    let e = parse(&["--as-needed=1"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnexpectedValue);
}
