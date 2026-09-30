//! Unknown and ignored linker options (ld checklist phase 6), wild's way:
//! silently ignored flags are aliases of one unread switch, unknown flags are
//! collected whole by an `#[arg(unknown)]` field, and the linker warns about
//! those it knows to ignore and reports the rest together.
#![cfg(feature = "derive")]

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind, Occurrence};

#[derive(Occurrence, Debug, PartialEq)]
enum Item {
    #[arg(long)]
    AsNeeded,
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
    #[arg(short = 's', long)]
    strip_all: bool,
    #[arg(short = 'S', long)]
    strip_debug: bool,
    /// wild's `SILENTLY_IGNORED_FLAGS`, one switch that is never read.
    #[arg(
        long = "start-group",
        alias("end-group", "nostdlib", "fatal-warnings", "sort-common", "stats"),
        short = '('
    )]
    #[expect(dead_code, reason = "accepted and ignored")]
    ignored: bool,
    #[arg(unknown)]
    unknown: Vec<String>,
    #[arg(sequence)]
    items: Vec<Item>,
}

const IGNORED_FLAGS: &[&str] = &["fix-cortex-a53-835769", "discard-all", "x"];

fn parse(line: &str) -> Result<Ld, Error> {
    let words: Vec<&BStr> = line.split_whitespace().map(BStr::new).collect();
    Ld::parse_from(&words)
}

#[test]
fn silently_ignored_flags_are_aliases() {
    let ld = parse("--start-group -lc -end-group -nostdlib --stats -( a.o").unwrap();
    assert!(ld.unknown.is_empty());
    assert_eq!(
        ld.items,
        [Item::Library("c".into()), Item::Input("a.o".into())]
    );
}

#[test]
fn unknown_flags_are_collected_whole_in_order() {
    let ld =
        parse("-shared --frobnicate a.o -fix-cortex-a53-835769 -x -Q -lm --wat=1 -o out").unwrap();
    assert!(ld.shared);
    assert_eq!(ld.output.as_deref(), Some("out"));
    assert_eq!(
        ld.unknown,
        [
            "--frobnicate",
            "-fix-cortex-a53-835769",
            "-x",
            "-Q",
            "--wat=1"
        ]
    );
    assert_eq!(
        ld.items,
        [Item::Input("a.o".into()), Item::Library("m".into())]
    );

    // The linker's split: warn about the known-ignored, fail on the rest.
    let (ignored, unrecognized): (Vec<_>, Vec<_>) = ld.unknown.iter().partition(|word| {
        let name = word.trim_start_matches('-');
        IGNORED_FLAGS.contains(&name.split('=').next().unwrap_or(name))
    });
    assert_eq!(ignored, ["-fix-cortex-a53-835769", "-x"]);
    assert_eq!(unrecognized, ["--frobnicate", "-Q", "--wat=1"]);
}

#[test]
fn a_bundle_with_an_unknown_letter_is_unknown_whole() {
    // `-sS` is two known switches; `-sQ` names `Q`, which nothing declares.
    let ld = parse("-sS -sQ").unwrap();
    assert!(ld.strip_all && ld.strip_debug);
    assert_eq!(ld.unknown, ["-sQ"]);
}

#[test]
fn single_dash_longs_are_not_bundles() {
    // `-shared` is a long option before it is `-s -h -a …`.
    let ld = parse("-shared -strip-all").unwrap();
    assert!(ld.shared && ld.strip_all);
    assert!(ld.unknown.is_empty());
}

#[test]
fn prefix_letters_are_known() {
    // `-lfoo` is the sequence's library, not an unknown bundle `-l -f -o -o`.
    let ld = parse("-lfoo").unwrap();
    assert!(ld.unknown.is_empty());
    assert_eq!(ld.items, [Item::Library("foo".into())]);
}

#[test]
fn known_flags_still_check_their_values() {
    let error = parse("-o").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingValue);
}

/// LTO plugin options, forwarded as text, and ld's version flags.
#[derive(Args, Debug)]
#[arg(long_only, disable_help_flag)]
struct Lto {
    #[arg(long, allow_hyphen_values)]
    plugin: Option<String>,
    #[arg(long, allow_hyphen_values)]
    plugin_opt: Vec<String>,
    #[arg(long = "lto-O", require_equals)]
    lto_o: Option<u8>,
    /// `-v` and `--version` print the version; `-V` adds the emulations.
    #[arg(short = 'v', long)]
    version: bool,
    #[arg(short = 'V')]
    verbose_version: bool,
    #[arg(long)]
    help: bool,
    #[arg(unknown)]
    unknown: Vec<String>,
}

fn lto(line: &str) -> Lto {
    let words: Vec<&BStr> = line.split_whitespace().map(BStr::new).collect();
    Lto::parse_from(&words).unwrap()
}

#[test]
fn plugin_options_are_text() {
    let parsed = lto(
        "-plugin /usr/lib/LLVMgold.so -plugin-opt=-pass-through=-lgcc \
         -plugin-opt -mcpu=native --plugin-opt=O2 --lto-O=3 --thinlto-jobs=4",
    );
    assert_eq!(parsed.plugin.as_deref(), Some("/usr/lib/LLVMgold.so"));
    assert_eq!(
        parsed.plugin_opt,
        ["-pass-through=-lgcc", "-mcpu=native", "O2"]
    );
    assert_eq!(parsed.lto_o, Some(3));
    // Options the linker forwards by prefix, as mold's `read_lto_option`.
    assert_eq!(parsed.unknown, ["--thinlto-jobs=4"]);
}

#[test]
fn version_flags_are_the_linkers() {
    let parsed = lto("-v");
    assert!(parsed.version && !parsed.verbose_version);
    assert!(lto("--version").version);
    assert!(lto("-V").verbose_version);
    assert!(lto("-help").help);
}
