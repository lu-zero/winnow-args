//! `values = a..` and `a..=b`: a flag that takes a variable number of words,
//! and `value_terminator`.

use winnow_args::{Args, Error, ErrorKind};

#[derive(Args, Debug, Default, PartialEq)]
struct Cli {
    /// Any number, at least one.
    #[arg(short = 'I', long, values = 1..)]
    include: Vec<String>,
    /// Two or three.
    #[arg(long, values = 2..=3)]
    point: Vec<i32>,
    /// Up to the `;`.
    #[arg(long, values = 1.., value_terminator = ";", allow_hyphen_values)]
    exec: Vec<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(positional)]
    files: Vec<String>,
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    Cli::parse_from(line)
}

#[test]
fn an_open_range_takes_words_until_a_flag() {
    let cli = parse(&["--include", "a", "b", "c", "-v", "f"]).unwrap();
    assert_eq!(cli.include, ["a", "b", "c"]);
    assert!(cli.verbose);
    assert_eq!(cli.files, ["f"]);
    // Each occurrence adds its own run; `--` ends one.
    let cli = parse(&["-I", "a", "-Ib", "c", "--", "d"]).unwrap();
    assert_eq!(cli.include, ["a", "b", "c"]);
    assert_eq!(cli.files, ["d"]);
    // The attached value is the first of the run.
    assert_eq!(parse(&["--include=a", "b"]).unwrap().include, ["a", "b"]);
}

#[test]
fn a_closed_range_stops_at_its_most_and_fails_below_its_least() {
    let cli = parse(&["--point", "1", "2", "3", "4"]).unwrap();
    assert_eq!(cli.point, [1, 2, 3]);
    assert_eq!(cli.files, ["4"]);
    assert_eq!(parse(&["--point", "1", "2", "-v"]).unwrap().point, [1, 2]);
    for line in [&["--point", "1"][..], &["--point", "1", "-v"]] {
        let e = parse(line).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::MissingValue, "{line:?}");
    }
}

#[test]
fn a_terminator_ends_the_run_and_is_dropped() {
    let cli = parse(&["--exec", "rm", "-f", "{}", ";", "f", "-v"]).unwrap();
    assert_eq!(cli.exec, ["rm", "-f", "{}"]);
    assert_eq!(cli.files, ["f"]);
    assert!(cli.verbose);
    // Without it, hyphen values run to the end of the line.
    assert_eq!(parse(&["--exec", "ls", "-v"]).unwrap().exec, ["ls", "-v"]);
}

#[test]
fn help_shows_the_run() {
    let help = winnow_args::help::render(Cli::HELP, &["cli"], false);
    assert!(help.contains("-I, --include <INCLUDE>..."), "{help}");
    assert!(help.contains("--point <POINT> <POINT>..."), "{help}");
}
