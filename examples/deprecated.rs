//! Deprecations, as a program at version 2.1 has them: a flag renamed, a
//! flag on its way out, one that warns only from 2.2, a positional replaced
//! by a flag, a command renamed, and a variable that has had three names.
//!
//!     cargo run --example deprecated -- --quiet build        # nothing to say
//!     cargo run --example deprecated -- -q --no-color build  # --no-color is deprecated
//!     cargo run --example deprecated -- --legacy-cache build # quiet until 2.2
//!     cargo run --example deprecated -- compile src          # a command and a positional
//!     PKG_JOBS=4 cargo run --example deprecated -- build     # the variable's name now
//!     MAKEFLAGS_JOBS=4 cargo run --example deprecated -- build
//!     OLD_PKG_JOBS=4 cargo run --example deprecated -- build # an old name: a warning
//!     cargo run --example deprecated -- --help               # what help says of them
//!     cargo run --example deprecated -- help build
//!
//! `Args::parse` prints the warnings on standard error before it returns.
//! With `--json-warnings` this program takes them itself, with
//! `Args::warnings` after `Args::try_parse`, and prints them as it likes.

use winnow_args::warn::WarningKind;
use winnow_args::{Args, Subcommand, report};

/// Build packages.
#[derive(Args)]
#[arg(name = "pkg", version = "2.1.0")]
struct Pkg {
    /// Say less.
    #[arg(short, long)]
    quiet: bool,
    /// Never paint the output.
    #[arg(long, deprecated = "use `--color never`")]
    no_color: bool,
    /// How to paint the output.
    #[arg(long, value_name = "WHEN")]
    color: Option<String>,
    /// Keep the build cache in the old layout.
    #[arg(
        long,
        deprecated,
        deprecated_warn_at = "2.2",
        deprecated_remove_at = "3.0"
    )]
    legacy_cache: bool,
    /// Report warnings as JSON lines, in place of text.
    #[arg(long)]
    json_warnings: bool,
    #[arg(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the packages in a directory.
    Build {
        /// Where the packages are.
        #[arg(long, value_name = "DIR")]
        dir: Option<String>,
        /// Where the packages are; `--dir` says it now.
        #[arg(positional, deprecated = "use `--dir`", deprecated_remove_at = "3.0")]
        path: Option<String>,
        /// How many jobs run at once.
        #[arg(
            short,
            long,
            env = "PKG_JOBS",
            env_fallback = "MAKEFLAGS_JOBS",
            deprecated_env = "OLD_PKG_JOBS"
        )]
        jobs: Option<u32>,
    },
    /// What `build` was called before 2.0.
    #[arg(
        alias = "make",
        deprecated = "use `build`",
        deprecated_remove_at = "3.0"
    )]
    Compile {
        /// Where the packages are.
        #[arg(positional)]
        path: Option<String>,
    },
}

fn main() {
    let pkg = match Pkg::try_parse() {
        Ok(pkg) => pkg,
        Err(error) => {
            // Help, version and failures, with the warnings first, as
            // `Pkg::parse()` does.
            print_warnings(false);
            std::process::exit(report(&error, &Pkg::program()));
        }
    };
    print_warnings(pkg.json_warnings);
    let _ = (pkg.quiet, pkg.no_color, &pkg.color, pkg.legacy_cache);
    match pkg.command {
        Command::Build { dir, path, jobs } => {
            let dir = dir.or(path).unwrap_or_else(|| ".".to_owned());
            println!("building {dir} with {} jobs", jobs.unwrap_or(1));
        }
        Command::Compile { path } => {
            println!("building {} with 1 jobs", path.as_deref().unwrap_or("."));
        }
    }
}

/// The warnings due at this version: rendered as `Pkg::parse()` does, or one
/// JSON object a line.
fn print_warnings(json: bool) {
    for warning in Pkg::warnings() {
        if !json {
            eprint!("{}", warning.render());
            continue;
        }
        let kind = match warning.kind {
            WarningKind::Argument => "argument",
            WarningKind::Command => "command",
            WarningKind::Env => "env",
            _ => "other",
        };
        let removed_at = warning
            .deprecation
            .and_then(|d| d.remove_at)
            .unwrap_or_default();
        let instead = warning.replacement.unwrap_or_default();
        eprintln!(
            r#"{{"kind":"{kind}","name":"{}","removed_at":"{removed_at}","instead":"{instead}"}}"#,
            warning.name
        );
    }
}
