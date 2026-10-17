//! What derived help looks like: a small mise-like CLI whose help covers
//! subcommands (one of them the default, one a variant with fields), aliases,
//! value names, defaults, environment variables, possible values, a heading
//! with a paragraph, a warning, examples, and long help. Its descriptions
//! use each mark help paints: `` `code` ``, `*emphasis*` and `**strong**`. An
//! example's command line is shown as typed, marks and all.
//!
//!     cargo run --example help -- -h            # short help
//!     cargo run --example help -- --help        # long help
//!     cargo run --example help -- help use      # a subcommand's help
//!     cargo run --example help -- use -h        # with the global flags it inherits
//!     COLUMNS=60 cargo run --example help -- -h # wrapped to 60 columns
//!     cargo run --example help --features terminal-size -- -h
//!     cargo run --example help --no-default-features --features derive -- -h
//!
//! The last one leaves the prose out (the `help-text` feature).
//!
//! Colors follow the terminal: `NO_COLOR=1` turns them off,
//! `CLICOLOR_FORCE=1` keeps them in a pipe, and on a 24-bit terminal
//! (`COLORTERM=truecolor`) the headings use the deeper teal of [`THEME`];
//! other terminals get the default theme's palettes.

use winnow_args::color::{Color, Paint, Palette, Theme};
use winnow_args::{Args, Subcommand, ValueEnum, report_with, words};

/// The default theme, with a deeper teal for headings where 24-bit color is
/// available.
const THEME: Theme = Theme {
    truecolor: Some(Palette {
        header: Paint::fg(Color::Rgb(38, 166, 154)).bold(),
        ..Palette::DEFAULT_256
    }),
    ..Theme::DEFAULT
};

/// Dev tools, env vars, and tasks in one CLI
///
/// Prepares the development environment before each command runs: installs
/// the tools a project asks for and puts them on PATH.
#[derive(Args, Debug)]
#[arg(
    name = "tool",
    version = "0.1.0",
    default_subcommand = "use",
    heading("Performance", help = "How *much* is done at once."),
    example("tool use -g node@20", header = "Install a tool for **every** project"),
    example(
        "tool use \"node@`cat .nvmrc`\"",
        header = "Install the version in `.nvmrc`",
        help = "The shell runs what is between the backticks: *not* a mark here."
    ),
    example = "tool ls --json"
)]
struct Cli {
    /// Show extra output (repeat for more)
    #[arg(short, long, count, global)]
    verbose: u8,
    /// Change to this directory before running the command
    #[arg(short = 'C', long, value_name = "DIR", env = "TOOL_CD")]
    cd: Option<String>,
    /// Number of jobs to run in parallel
    ///
    /// Defaults to four; more jobs install *faster* but print **interleaved**
    /// output.
    #[arg(short, long, default = "4", help_heading = "Performance")]
    jobs: Option<u32>,
    /// When to use color
    #[arg(long, default = "auto")]
    color: Option<When>,
    /// Colorize progress bars
    #[arg(long, negate, default = "true")]
    progress: bool,
    /// Install without checking signatures
    #[arg(
        long,
        help_heading = "Performance",
        warning = "Only for a mirror you trust."
    )]
    unverified: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(ValueEnum, Debug)]
enum When {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Install a tool and add it to the project
    #[arg(alias = "u")]
    Use(UseArgs),
    /// List installed tools
    #[arg(alias = "list")]
    Ls(LsArgs),
    /// Remove a tool
    #[arg(alias = "rm", example = "tool remove node@20")]
    Remove {
        /// The tool to remove, as `name@version`
        #[arg(positional, value_name = "TOOL@VERSION")]
        tool: String,
    },
}

/// Install a tool and add it to the project.
#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct UseArgs {
    /// Use the global config file instead of the project's
    #[arg(short, long)]
    global: bool,
    /// Tools to install, as `name@version`
    #[arg(positional, value_name = "TOOL@VERSION")]
    tools: Vec<String>,
}

/// List installed tools.
#[derive(Args, Debug)]
struct LsArgs {
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let cli = match Cli::parse_words(&words(&args)) {
        Ok(cli) => cli,
        // `-h`, `--help`, `help …` and `-V` end up here too.
        Err(e) => std::process::exit(report_with(&e, "tool", &THEME)),
    };
    println!(
        "verbose {}, cd {:?}, jobs {:?}, color {:?}, progress {}, unverified {}",
        cli.verbose, cli.cd, cli.jobs, cli.color, cli.progress, cli.unverified
    );
    match cli.command {
        Some(Command::Use(u)) => println!("use: global {}, tools {:?}", u.global, u.tools),
        Some(Command::Ls(l)) => println!("ls: json {}", l.json),
        Some(Command::Remove { tool }) => println!("remove: {tool}"),
        None => {}
    }
}
