//! How many values a flag takes, and where its default comes from.
use winnow_args::Args;

fn four() -> u32 {
    4
}

#[derive(Args)]
struct NoValues {
    #[arg(long, values = 0)]
    point: Vec<u32>,
}

#[derive(Args)]
struct BackwardsRange {
    #[arg(long, values = 3..=2)]
    point: Vec<u32>,
}

#[derive(Args)]
struct ValuesOnOne {
    #[arg(long, values = 1..)]
    point: u32,
}

#[derive(Args)]
struct RangeWithEquals {
    #[arg(long, values = 1.., require_equals)]
    point: Vec<u32>,
}

#[derive(Args)]
struct TerminatorOfNothing {
    #[arg(long, value_terminator = ";")]
    exec: Vec<String>,
}

#[derive(Args)]
struct TwoDefaults {
    #[arg(long, default = "1", default_fn = four)]
    jobs: u32,
}

#[derive(Args)]
struct NoteOfNothing {
    #[arg(long, default_note = "one per core")]
    jobs: Option<u32>,
}

#[derive(Args)]
struct DefaultFnOnASwitch {
    #[arg(long, default_fn = four)]
    verbose: bool,
}

fn main() {}
