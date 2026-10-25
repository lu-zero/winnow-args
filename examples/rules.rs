//! Errors and rules, as a `deploy` tool has them: two required flags,
//! one required only for production, a conflict in the program's own words,
//! and a check of the whole struct with `validate`. Its choices say what
//! they mean.
//!
//!     cargo run --example rules                                 # both missing
//!     cargo run --example rules -- --target production --release v2
//!     cargo run --example rules -- --target staging --release v2 --dry-run --wait
//!     cargo run --example rules -- --target staging --release v2 --shift 10 50
//!     cargo run --example rules -- --target staging --release v2 --strategy canry
//!     cargo run --example rules -- -h                           # choices on one line
//!     cargo run --example rules -- --help                       # what each one means

use winnow_args::{Args, Error, ValueEnum};

/// Deploy a release.
#[derive(Args, Debug)]
#[arg(name = "deploy", version = "1.0.0", validate = check)]
struct Deploy {
    /// Where to deploy.
    #[arg(long)]
    target: Target,
    /// The release to deploy.
    #[arg(long, value_name = "TAG")]
    release: String,
    /// The approver, needed for production.
    #[arg(long, required_if("--target", "production"))]
    approver: Option<String>,
    /// How to roll it out.
    #[arg(long, default = "rolling")]
    strategy: Strategy,
    /// Wait for the health checks.
    #[arg(long, bool_value)]
    wait: bool,
    /// Traffic shifted per step, as a range.
    #[arg(long, value_names("FROM", "TO"))]
    shift: Vec<u8>,
    /// Print what would be done.
    #[arg(long, conflicts("--wait", message = "{0} never waits: drop {1}"))]
    dry_run: bool,
}

#[derive(ValueEnum, Debug, PartialEq)]
enum Target {
    /// The shared test cluster
    Staging,
    /// Customers see it
    Production,
}

#[derive(ValueEnum, Debug)]
enum Strategy {
    /// Replace the instances a few at a time
    Rolling,
    /// Start a second set and switch over
    BlueGreen,
    /// Send a share of the traffic first
    Canary,
}

fn check(d: &Deploy) -> Result<(), Error> {
    if !d.shift.is_empty() && !matches!(d.strategy, Strategy::Canary) {
        return Err(Error::invalid("--shift needs --strategy canary"));
    }
    Ok(())
}

fn main() {
    println!("{:?}", Deploy::parse());
}
