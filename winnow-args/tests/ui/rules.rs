//! Selectors and the rules that use them.
use winnow_args::Args;

#[derive(Args)]
struct NoSuchFlag {
    #[arg(long, conflicts("--quiet"))]
    verbose: bool,
}

#[derive(Args)]
struct TheNegation {
    #[arg(long, negate)]
    cache: bool,
    #[arg(long, requires("--no-cache"))]
    fresh: bool,
}

#[derive(Args)]
struct NoSuchGroup {
    #[arg(long, group = "output")]
    json: bool,
}

#[derive(Args)]
struct DefaultIfArity {
    #[arg(long, default_if("--json"))]
    color: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DefaultIfOnACount {
    #[arg(short, count, default_if("--json", "1"))]
    verbose: u8,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DefaultIfComparesACount {
    #[arg(long, default_if("-v", "2", "never"))]
    color: Option<String>,
    #[arg(short, count)]
    verbose: u8,
}

#[derive(Args)]
struct SwitchDefaultIf {
    #[arg(long, default_if("--json", "yes"))]
    quiet: bool,
    #[arg(long)]
    json: bool,
}

fn main() {}
