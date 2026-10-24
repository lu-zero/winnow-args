//! `default_subcommand` takes every word an `external` variant would: refused
//! when the program is built.
use winnow_args::{Args, Subcommand};

#[derive(Subcommand)]
enum Command {
    Build,
    #[arg(external)]
    Other(Vec<String>),
}

#[derive(Args)]
#[arg(default_subcommand = "build")]
struct DefaultAndExternal {
    #[arg(subcommand)]
    command: Command,
}

fn main() {
    let _ = DefaultAndExternal::parse_from(["x"]);
}
