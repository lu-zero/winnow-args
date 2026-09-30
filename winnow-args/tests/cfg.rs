//! `#[cfg]` on fields: a flag compiled out is not parsed, listed or built.
#![cfg(feature = "derive")]

use winnow::stream::BStr;
use winnow_args::{Args, Error, ErrorKind};

#[derive(Args, Debug)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    /// Present in every build.
    #[cfg(test)]
    #[arg(long)]
    kept: Option<String>,
    /// Never compiled.
    #[cfg(not(test))]
    #[arg(long)]
    gone: bool,
    /// One field per configuration, the same flag.
    #[cfg(test)]
    #[arg(long, default = "on")]
    mode: String,
    #[cfg(not(test))]
    #[arg(long, default = "off")]
    mode: String,
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words: Vec<&BStr> = line.iter().map(BStr::new).collect();
    Cli::parse_from(&words)
}

#[test]
fn only_the_configured_fields_exist() {
    let cli = parse(&["-v", "--kept", "x"]).unwrap();
    assert!(cli.verbose);
    assert_eq!(cli.kept.as_deref(), Some("x"));
    assert_eq!(cli.mode, "on");
    assert_eq!(
        parse(&["--gone"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
    assert_eq!(parse(&["--mode", "m"]).unwrap().mode, "m");
    assert!(!Cli::HELP.items.iter().any(|i| i.long == Some("gone")));
    assert_eq!(
        Cli::HELP
            .items
            .iter()
            .filter(|i| i.long == Some("mode"))
            .count(),
        1
    );
}
