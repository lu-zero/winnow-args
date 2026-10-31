//! `parse_with = f`: a value parsed by a function of its text, as clap's
//! `value_parser`.

use std::time::Duration;

use winnow_args::{Args, ErrorKind};

/// `30s`, `5m` or a number of seconds.
fn duration(text: &str) -> Result<Duration, String> {
    let (number, unit) = match text.strip_suffix('m') {
        Some(minutes) => (minutes, 60),
        None => (text.strip_suffix('s').unwrap_or(text), 1),
    };
    let n: u64 = number
        .parse()
        .map_err(|_| format!("`{text}` is not a duration"))?;
    Ok(Duration::from_secs(n * unit))
}

#[derive(Args, Debug, PartialEq)]
struct Cli {
    #[arg(long, parse_with = duration, default = "30s")]
    timeout: Duration,
    #[arg(long, parse_with = |s: &str| s.parse::<u8>().map(|n| n * 2))]
    double: Option<u8>,
    #[arg(long, delimiter = ',', parse_with = duration)]
    every: Vec<Duration>,
    #[arg(long, required_if("--timeout", "1m"))]
    reason: Option<String>,
    #[arg(positional, parse_with = str::parse::<i64>)]
    offset: Option<i64>,
}

fn parse(line: &[&str]) -> Result<Cli, winnow_args::Error> {
    Cli::parse_from(line)
}

#[test]
fn flags_positionals_and_defaults() {
    let cli = parse(&["--timeout", "5m", "--double=4", "--every=1s,2m", "7"]).unwrap();
    assert_eq!(cli.timeout, Duration::from_secs(300));
    assert_eq!(cli.double, Some(8));
    assert_eq!(
        cli.every,
        [Duration::from_secs(1), Duration::from_secs(120)]
    );
    assert_eq!(cli.offset, Some(7));
    assert_eq!(parse(&[]).unwrap().timeout, Duration::from_secs(30));
}

#[test]
fn its_error_is_the_cause() {
    let e = parse(&["--timeout", "soon"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
    assert!(e.to_string().contains("`soon` is not a duration"), "{e}");
    let e = parse(&["x"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
}

#[test]
fn a_condition_parses_its_literal_too() {
    // `1m` and `60s` are the same duration.
    let e = parse(&["--timeout", "60s"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingRequired);
    assert!(parse(&["--timeout", "60s", "--reason", "slow"]).is_ok());
}

#[cfg(unix)]
#[test]
fn a_value_not_utf8_is_refused() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let args = [OsString::from("--timeout"), OsString::from_vec(vec![0xff])];
    let e = Cli::parse_from(args).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidValue);
}
