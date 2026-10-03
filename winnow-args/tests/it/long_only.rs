//! `#[arg(long_only)]`: GNU's `getopt_long_only`, as `ld` reads its command
//! line. Vectors from mold (`src/cmdline.rs` tests, commit 890ec2da).

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind};

#[derive(Args, Debug, PartialEq, Default)]
#[arg(long_only)]
struct Ld {
    /// `-o` is output; `--output` only with two dashes (`-output` is `-o utput`).
    #[arg(short, long, two_dashes)]
    output: Option<String>,
    #[arg(short, long)]
    entry: Option<String>,
    #[arg(long)]
    eh_frame_hdr: bool,
    #[arg(long)]
    filter: Vec<String>,
    /// `-lfoo` is always a library, even where `lfoo` spells a long option.
    #[arg(short = 'l', long, prefix)]
    library: Vec<String>,
    #[arg(long)]
    lazy: bool,
    #[arg(long, alias = "Bshareable")]
    shared: bool,
    #[arg(long)]
    soname: Option<String>,
    #[arg(long = "Map")]
    map: Option<String>,
    /// Starts with `o`: two dashes only, `-omagic` is `-o magic`.
    #[arg(long, two_dashes)]
    omagic: bool,
    /// In ld's two-dashes-only list: `-execute-only` is `-e xecute-only`.
    #[arg(long, two_dashes)]
    execute_only: bool,
    #[arg(short)]
    s: bool,
    #[arg(short)]
    v: bool,
    #[arg(positional)]
    inputs: Vec<String>,
}

fn parse(line: &[&str]) -> Result<Ld, Error> {
    Ld::parse_from(line)
}

fn ok(line: &[&str]) -> Ld {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

#[test]
fn a_single_dash_long_wins_over_a_short_with_an_attached_value() {
    assert_eq!(ok(&["-entry=main"]).entry.as_deref(), Some("main"));
    assert_eq!(ok(&["-entry", "main"]).entry.as_deref(), Some("main"));
    assert!(ok(&["-eh-frame-hdr"]).eh_frame_hdr);
    assert_eq!(ok(&["-filter", "libf.so"]).filter, ["libf.so"]);
    assert!(ok(&["-shared"]).shared);
    assert!(ok(&["-Bshareable"]).shared);
    assert_eq!(ok(&["-soname=x"]).soname.as_deref(), Some("x"));
    assert_eq!(ok(&["-Map", "out.map"]).map.as_deref(), Some("out.map"));
    // No long name: a short with its attached value.
    assert_eq!(ok(&["-emain"]).entry.as_deref(), Some("main"));
    assert_eq!(ok(&["-e", "main"]).entry.as_deref(), Some("main"));
}

#[test]
fn two_dash_names_are_short_options_with_one() {
    assert_eq!(ok(&["-omagic"]).output.as_deref(), Some("magic"));
    assert!(ok(&["--omagic"]).omagic);
    assert_eq!(ok(&["-output", "x"]).output.as_deref(), Some("utput"));
    assert_eq!(ok(&["--output", "x"]).output.as_deref(), Some("x"));
    assert_eq!(ok(&["-execute-only"]).entry.as_deref(), Some("xecute-only"));
    assert!(ok(&["--execute-only"]).execute_only);
}

#[test]
fn a_prefix_letter_takes_its_word() {
    assert_eq!(ok(&["-lazy"]).library, ["azy"]);
    assert!(ok(&["--lazy"]).lazy);
    assert_eq!(ok(&["-lfoo", "-l", "bar"]).library, ["foo", "bar"]);
}

#[test]
fn short_bundles_and_inputs_still_work() {
    let ld = ok(&["a.o", "-sv", "-shared", "b.o", "--", "-c.o"]);
    assert!(ld.s && ld.v && ld.shared);
    assert_eq!(ld.inputs, ["a.o", "b.o", "-c.o"]);
    let e = parse(&["-xyz"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-x")));
    // The built-in `--help` takes one dash too.
    assert_eq!(
        parse(&["-help"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
}

/// Without `long_only`, one dash is always short letters.
#[derive(Args, Debug)]
struct Plain {
    #[arg(short, long)]
    entry: Option<String>,
}

#[test]
fn the_default_grammar_is_unchanged() {
    let words = [BStr::new("-entry=main")];
    assert_eq!(
        Plain::parse_words(&words).unwrap().entry.as_deref(),
        Some("ntry=main")
    );
}
