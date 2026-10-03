//! The enum derives.
use winnow_args::{Occurrence, Subcommand, ValueEnum};

#[derive(ValueEnum)]
enum Color {
    Auto,
    Rgb(u8, u8, u8),
}

#[derive(ValueEnum)]
#[arg(rename_all = "Title Case")]
enum Shout {
    Loud,
}

#[derive(ValueEnum)]
enum Twice {
    #[arg(name = "same")]
    One,
    #[arg(name = "same")]
    Two,
}

#[derive(Subcommand)]
enum Command {
    #[arg(short = 'r')]
    Run,
}

#[derive(Occurrence)]
enum TwoPositionals {
    #[arg(positional)]
    Input(String),
    #[arg(positional)]
    Output(String),
}

#[derive(Occurrence)]
enum SameFlag {
    #[arg(long = "lazy")]
    Lazy,
    #[arg(long = "lazy")]
    AlsoLazy,
}

fn main() {}
