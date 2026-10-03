//! A flag spelled twice across a struct and what it flattens.
use winnow_args::Args;

#[derive(Args, Default)]
struct Common {
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
struct Cli {
    #[arg(long)]
    verbose: bool,
    #[arg(flatten)]
    common: Common,
}

fn main() {}
