//! Without the `help-text` feature, help keeps its structure (commands, flags,
//! values, defaults, possible values, the version) and loses its prose.
#![cfg(all(feature = "derive", not(feature = "help-text")))]

use winnow::stream::BStr;
use winnow_args::{Args, ErrorKind, Subcommand, ValueEnum, with_env};

/// Dev tools, env vars, and tasks in one CLI
#[derive(Args, Debug)]
#[arg(name = "mise", version = "2026.9.0", after_help = "Examples: mise use")]
#[expect(dead_code, reason = "these tests assert on the help")]
struct Cli {
    /// Show extra output
    #[arg(short, long, count)]
    verbose: u8,
    /// Number of jobs
    #[arg(short, long, default = "4")]
    jobs: Option<u32>,
    #[arg(long)]
    color: Option<When>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(ValueEnum, Debug)]
enum When {
    Auto,
    Never,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Installs a tool
    Use,
}

fn help() -> String {
    let e = Cli::parse_from(&[BStr::new("-h")]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested);
    with_env(&[], || e.render_help("mise").unwrap())
}

#[test]
fn help_keeps_its_structure_and_drops_the_prose() {
    assert_eq!(
        help(),
        "\
Usage: mise [OPTIONS] [COMMAND]

Commands:
  use

Options:
  -v, --verbose...
  -j, --jobs <JOBS>    [default: 4]
      --color <COLOR>  [possible values: auto, never]
  -h, --help           Print help (see more with '--help')
  -V, --version        Print version
"
    );
}

#[test]
fn the_version_stays() {
    let e = Cli::parse_from(&[BStr::new("-V")]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::VersionRequested);
}
