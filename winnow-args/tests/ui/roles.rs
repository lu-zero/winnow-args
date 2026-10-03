//! Field roles: what each takes, and how many a struct has.
use winnow_args::{Args, Occurrence, Subcommand};

#[derive(Args, Default)]
struct Flags {
    #[arg(long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    Run,
}

#[derive(Occurrence)]
enum Item {
    #[arg(long)]
    Lazy,
}

#[derive(Args)]
struct SkipWithMore {
    #[arg(skip, long)]
    cache: bool,
}

#[derive(Args)]
struct FlattenWithMore {
    #[arg(flatten, hide)]
    flags: Flags,
}

#[derive(Args)]
struct SubcommandWithMore {
    #[arg(subcommand, long)]
    command: Command,
}

#[derive(Args)]
struct TwoSubcommands {
    #[arg(subcommand)]
    first: Command,
    #[arg(subcommand)]
    second: Option<Command>,
}

#[derive(Args)]
struct TwoSequences {
    #[arg(sequence)]
    first: Vec<Item>,
    #[arg(sequence)]
    second: Vec<Item>,
}

#[derive(Args)]
struct SequenceOfOne {
    #[arg(sequence)]
    item: Item,
}

#[derive(Args)]
struct RequiredAfterOptional {
    #[arg(positional)]
    first: Option<String>,
    #[arg(positional)]
    second: String,
}

#[derive(Args)]
#[arg(default_subcommand = "run")]
struct DefaultOfNothing {
    #[arg(long)]
    verbose: bool,
}

fn main() {}
