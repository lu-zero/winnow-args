//! `#[arg(flatten)]`: another struct's flags parsed as if declared here.

use winnow_args::{Args, Error, ErrorKind};

/// Options shared by several commands, as `complete` and `compgen` share theirs.
#[derive(Args, Debug, Default, PartialEq)]
struct Common {
    /// Actions.
    #[arg(short = 'A', value_name = "action")]
    actions: Vec<String>,
    /// Quiet.
    #[arg(short, long)]
    quiet: bool,
    /// A pattern.
    #[arg(long, default = "*")]
    pattern: String,
    /// Colors, on or off.
    #[arg(long, negate = "no-color")]
    color: bool,
    #[arg(flatten)]
    deep: Deep,
}

/// Flattened into `Common`, so into whoever flattens it.
#[derive(Args, Debug, Default, PartialEq)]
struct Deep {
    /// Depth.
    #[arg(short = 'd', long)]
    depth: Option<u32>,
}

#[derive(Args, Debug, PartialEq)]
struct Complete {
    /// Print.
    #[arg(short)]
    print: bool,
    #[arg(flatten)]
    common: Common,
    #[arg(positional)]
    names: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
#[arg(unknown_flags = "value")]
struct Lenient {
    #[arg(short)]
    verbose: bool,
    #[arg(flatten)]
    common: Common,
    #[arg(positional)]
    words: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
#[arg(long_only)]
struct LongOnly {
    #[arg(long)]
    shared: bool,
    #[arg(flatten)]
    common: Common,
}

#[derive(Args, Debug, PartialEq)]
struct Needs {
    #[arg(flatten)]
    must: Must,
}

#[derive(Args, Debug, PartialEq)]
struct Must {
    #[arg(long, required)]
    name: Option<String>,
}

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::try_parse_from(line)
}

#[test]
fn flattened_flags_parse_among_ours() {
    let c: Complete = parse(&["-p", "-A", "alias", "--quiet", "-Afile", "x", "-d3", "y"]).unwrap();
    assert!(c.print);
    assert_eq!(c.common.actions, ["alias", "file"]);
    assert!(c.common.quiet);
    assert_eq!(c.common.deep.depth, Some(3));
    assert_eq!(c.names, ["x", "y"]);
}

#[test]
fn bundles_mix_ours_and_theirs() {
    let c: Complete = parse(&["-pqAvariable"]).unwrap();
    assert!(c.print && c.common.quiet);
    assert_eq!(c.common.actions, ["variable"]);
}

#[test]
fn defaults_and_negations_are_finished() {
    let c: Complete = parse(&[]).unwrap();
    assert_eq!(
        c.common,
        Common {
            pattern: "*".into(),
            ..Common::default()
        }
    );
    let c: Complete = parse(&["--color", "--no-color", "--pattern=?"]).unwrap();
    assert!(!c.common.color);
    assert_eq!(c.common.pattern, "?");
}

#[test]
fn errors_come_from_the_flattened_struct() {
    let e = parse::<Complete>(&["-A"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingValue);
    let e = parse::<Complete>(&["--depth", "deep"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
    let e = parse::<Complete>(&["--nope"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnknownFlag);
    let e = parse::<Needs>(&[]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
    assert!(parse::<Needs>(&["--name", "x"]).is_ok());
}

#[test]
fn a_lenient_bundle_knows_their_letters() {
    // `-vq`: both known, one ours, one flattened; `-vz` has an unknown letter.
    let l: Lenient = parse(&["-vq", "-vz"]).unwrap();
    assert!(l.verbose && l.common.quiet);
    assert_eq!(l.words, ["-vz"]);
}

#[test]
fn long_only_knows_their_names() {
    let l: LongOnly = parse(&["-shared", "-quiet", "-depth", "2"]).unwrap();
    assert!(l.shared && l.common.quiet);
    assert_eq!(l.common.deep.depth, Some(2));
}

#[test]
fn help_lists_their_items_after_ours() {
    // Ours in declaration order, then the flattened struct's, its own
    // flattened items last.
    let names: Vec<String> = Complete::HELP
        .items
        .iter()
        .map(|i| match (i.short, i.long) {
            (_, Some(long)) => format!("--{long}"),
            (Some(short), None) => format!("-{short}"),
            (None, None) => i.value_name.unwrap_or("").to_owned(),
        })
        .collect();
    assert_eq!(
        names,
        [
            "-p",
            "NAMES",
            "-A",
            "--quiet",
            "--pattern",
            "--color",
            "--depth"
        ]
    );
    let text = parse::<Complete>(&["--help"])
        .unwrap_err()
        .render_help("complete")
        .unwrap();
    for flag in ["-p", "-A", "--quiet", "--pattern", "--color", "--depth"] {
        assert!(text.contains(flag), "{flag} in {text}");
    }
}
