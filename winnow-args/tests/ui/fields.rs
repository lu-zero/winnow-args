//! Field attributes used where they mean nothing.
use winnow_args::Args;

#[derive(Args)]
struct Unknown {
    #[arg(long, hdie)]
    verbose: bool,
}

#[derive(Args)]
struct DelimiterOnOne {
    #[arg(long, delimiter = ',')]
    path: String,
}

#[derive(Args)]
struct DefaultOnASwitch {
    #[arg(long, default = "true")]
    verbose: bool,
}

#[derive(Args)]
struct MissingDefaultOnASwitch {
    #[arg(long, default_missing = "x")]
    verbose: bool,
}

#[derive(Args)]
struct StopFlagsOnAFlag {
    #[arg(long, stop_flags)]
    path: String,
}

#[derive(Args)]
struct NegateOnAValue {
    #[arg(long, negate)]
    path: String,
}

#[derive(Args)]
struct PlusWithoutPlusOptions {
    #[arg(short = 'x', plus = 'x')]
    trace: Option<bool>,
}

#[derive(Args)]
struct HintOnASwitch {
    #[arg(long, hint = "dir")]
    verbose: bool,
}

#[derive(Args)]
struct HintOfNoKind {
    #[arg(long, hint = "folder")]
    path: String,
}

#[derive(Args)]
struct Tuple(bool);

fn names(_typed: &str) -> Vec<winnow_args::complete::Candidate> {
    Vec::new()
}

#[derive(Args)]
struct HintBesideAFunction {
    #[arg(long, hint = "dir", complete = names)]
    path: String,
}

#[derive(Args)]
struct FunctionOnACount {
    #[arg(short, count, complete = names)]
    verbose: u8,
}

#[derive(Args)]
struct HiddenChoicesOfNone {
    #[arg(long, hide_choices)]
    verbose: bool,
}

fn main() {}
