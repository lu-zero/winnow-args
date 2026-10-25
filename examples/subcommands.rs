//! Subcommands and completion, as a `pm` package manager has them: a default
//! subcommand that its own flags select and an empty line runs, a variant
//! with fields, global flags from a flattened struct, and values the program
//! completes itself.
//!
//!     cargo run --example subcommands -- --help          # the default's page too
//!     cargo run --example subcommands -- -pu firefox     # `install -pu firefox`
//!     cargo run --example subcommands                    # `install`
//!     cargo run --example subcommands -- -v rm -n vim
//!     cargo run --example subcommands -- rm -h           # with its global flags
//!
//! Completion in bash, with the binary on `PATH` as `pm`:
//!
//!     source <(pm --bash-completion)

use winnow_args::complete::{Candidate, Shell};
use winnow_args::{Args, Subcommand};

/// A package manager.
#[derive(Args, Debug)]
#[arg(
    name = "pm",
    version = "1.0.0",
    default_subcommand = "install",
    default_subcommand_flags,
    default_subcommand_help,
    default_subcommand_on_empty,
    example("pm -pu firefox", header = "Update firefox, pretending")
)]
struct Pm {
    #[arg(flatten)]
    common: Common,
    #[arg(subcommand)]
    command: Command,
}

/// Flags every command takes.
#[derive(Args, Debug)]
struct Common {
    /// Print each step.
    #[arg(short, long, global)]
    verbose: bool,
    /// Install under this directory.
    #[arg(long, global, hint = "dir")]
    root: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Install packages.
    #[arg(alias = "i")]
    Install(Install),
    /// Remove packages.
    #[arg(alias = "rm")]
    // Read only through `Debug`.
    #[allow(dead_code)]
    Remove {
        /// Say what would be removed.
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// The packages to remove.
        #[arg(positional, complete = packages)]
        packages: Vec<String>,
    },
}

/// Install packages.
#[derive(Args, Debug)]
struct Install {
    /// Update what is installed.
    #[arg(short, long)]
    update: bool,
    /// Say what would be done.
    #[arg(short, long)]
    pretend: bool,
    /// The packages to install.
    #[arg(positional, complete = packages)]
    packages: Vec<String>,
}

fn packages(_typed: &str) -> Vec<Candidate> {
    ["firefox", "fish", "vim", "virtualbox"]
        .into_iter()
        .map(|value| Candidate {
            value: value.to_owned(),
            help: "a package",
        })
        .collect()
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Some(answer) = Pm::completion_request(&args) {
        print!("{answer}");
        return;
    }
    if args.first().is_some_and(|a| a == "--bash-completion") {
        print!("{}", Pm::completion_script(Shell::Bash));
        return;
    }
    let pm = Pm::parse();
    println!("{:?}\n{:?}", pm.common, pm.command);
}
