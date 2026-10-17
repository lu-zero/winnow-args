//! `default_fn` and `default_if`: defaults computed, or that depend on another flag.

use winnow_args::{Args, Error, ValueEnum, with_env};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
enum Format {
    Text,
    Json,
}

fn cores() -> usize {
    8
}

fn all() -> Vec<String> {
    vec!["src".into(), "tests".into()]
}

#[derive(Args, Debug, PartialEq)]
struct Cli {
    #[arg(long)]
    json: bool,
    #[arg(long)]
    format: Option<Format>,
    /// Given: `--json` was; valued: `--format` is `json`.
    #[arg(
        long,
        default_if("--json", "never"),
        default_if("--format", "json", "never"),
        default = "auto"
    )]
    color: String,
    #[arg(long, default_if("--format", "json", "true"))]
    quiet: bool,
    #[arg(short, long, env = "JOBS", default_fn = cores, default_note = "one per core")]
    jobs: usize,
    #[arg(long, default_fn = all)]
    dir: Vec<String>,
    #[arg(long, negate, default_fn = on)]
    cache: bool,
}

fn on() -> bool {
    true
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    with_env(&[], || Cli::parse_from(line))
}

#[test]
fn a_default_follows_another_flag() {
    assert_eq!(parse(&[]).unwrap().color, "auto");
    assert_eq!(parse(&["--json"]).unwrap().color, "never");
    assert_eq!(parse(&["--format", "json"]).unwrap().color, "never");
    assert_eq!(parse(&["--format", "text"]).unwrap().color, "auto");
    // The command line still wins.
    assert_eq!(
        parse(&["--json", "--color=always"]).unwrap().color,
        "always"
    );
}

#[test]
fn a_switch_can_follow_a_value() {
    assert!(parse(&["--format=json"]).unwrap().quiet);
    assert!(!parse(&["--format=text"]).unwrap().quiet);
    assert!(!parse(&[]).unwrap().quiet);
}

#[test]
fn a_function_gives_the_default_after_the_command_line_and_the_environment() {
    let cli = parse(&[]).unwrap();
    assert_eq!((cli.jobs, cli.cache), (8, true));
    assert_eq!(cli.dir, ["src", "tests"]);
    let cli = parse(&["-j2", "--dir", "x", "--no-cache"]).unwrap();
    assert_eq!((cli.jobs, cli.cache), (2, false));
    assert_eq!(cli.dir, ["x"]);
    let from_env = with_env(&[("JOBS", "3")], || Cli::parse_from::<_, &str>([]));
    assert_eq!(from_env.unwrap().jobs, 3);
}

#[test]
fn help_shows_the_note_and_does_not_require_the_flag() {
    let help = winnow_args::help::render(Cli::HELP, &["cli"], false);
    assert!(help.contains("[default: one per core]"), "{help}");
    assert!(help.contains("Usage: cli [OPTIONS]"), "{help}");
}

/// `default_from(field, …, function)`: a default made from other fields.
mod default_from {
    use winnow_args::{Args, ErrorKind, with_env};

    #[derive(Args, Debug)]
    struct Convert {
        #[arg(positional)]
        input: String,
        /// Where to write.
        #[arg(long, default_from(input, |input: &String| format!("{input}.out")))]
        output: String,
        #[arg(long)]
        level: Option<u8>,
        #[arg(long)]
        fast: bool,
        /// One from two fields, one of them optional.
        #[arg(
            long,
            default_from(level, fast, effort),
            default_note = "9 unless --fast"
        )]
        effort: u8,
        /// Made from a computed one.
        #[arg(long, env = "CONVERT_LOG", default_from(output, |output: &String| format!("{output}.log")))]
        log: Option<String>,
    }

    fn effort(level: &Option<u8>, fast: &bool) -> u8 {
        level.unwrap_or(if *fast { 1 } else { 9 })
    }

    #[test]
    fn it_is_made_from_the_fields_it_names() {
        let c = Convert::parse_from(["a.txt"]).unwrap();
        assert_eq!(c.output, "a.txt.out");
        assert_eq!(c.effort, 9);
        assert_eq!(c.log.as_deref(), Some("a.txt.out.log"));
        let c = Convert::parse_from(["a.txt", "--fast", "--output", "b"]).unwrap();
        assert_eq!(
            (c.output.as_str(), c.effort, c.log.as_deref()),
            ("b", 1, Some("b.log"))
        );
        assert_eq!(
            Convert::parse_from(["a", "--level", "4"]).unwrap().effort,
            4
        );
    }

    #[test]
    fn the_command_line_and_the_environment_come_first() {
        let c = Convert::parse_from(["a", "--effort", "2"]).unwrap();
        assert_eq!(c.effort, 2);
        let c = with_env(&[("CONVERT_LOG", "x.log")], || Convert::parse_from(["a"])).unwrap();
        assert_eq!(c.log.as_deref(), Some("x.log"));
    }

    #[test]
    fn a_missing_field_it_is_made_from_is_reported_as_missing() {
        let error = Convert::parse_from(["--fast"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::MissingArgument);
    }

    /// A closure needs no types: the derive states them.
    #[derive(Args, Debug)]
    #[arg(name = "cv")]
    struct Untyped {
        #[arg(positional)]
        input: std::path::PathBuf,
        #[arg(positional, default_from(input, |input| input.with_extension("p")))]
        second: std::path::PathBuf,
        #[arg(short, count)]
        verbose: u8,
        #[arg(long, default_from(verbose, input, |v, input| format!("{v}:{}", input.display())))]
        tag: String,
    }

    #[test]
    fn a_closure_needs_no_types() {
        let u = Untyped::parse_from(["a.txt", "-vv"]).unwrap();
        assert_eq!(u.second, std::path::Path::new("a.p"));
        assert_eq!(u.tag, "2:a.txt");
    }

    #[cfg(feature = "help-text")]
    #[test]
    fn a_field_with_a_computed_default_is_not_required() {
        let error = Untyped::parse_from(["-h"]).unwrap_err();
        let page = error.render_help("cv").unwrap();
        assert!(
            page.contains("Usage: cv [OPTIONS] <INPUT> [SECOND]"),
            "{page}"
        );
    }

    #[cfg(feature = "help-text")]
    #[test]
    fn help_says_what_it_is_made_from() {
        let error = Convert::parse_from(["--help"]).unwrap_err();
        let page =
            winnow_args::with_env(&[("COLUMNS", "100")], || error.render_help("convert")).unwrap();
        assert!(page.contains("[default: from `input`]"), "{page}");
        assert!(page.contains("[default: 9 unless --fast]"), "{page}");
    }
}
