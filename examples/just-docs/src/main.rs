//! A program whose pages are built by the justfile next to this file.
//!
//! ```text
//! just docs
//! just docs --out /tmp/just-docs
//! ```
//!
//! The recipe calls `xtask-docs`. That is the same generator brush would run
//! as `cargo xtask gen docs`, with a just recipe in front of it.

use winnow_args::Args;

/// Greet someone.
#[derive(Debug, Args)]
#[arg(name = "hello")]
struct Cli {
    /// Who to greet.
    #[arg(positional)]
    name: Option<String>,

    /// Greet loudly.
    #[arg(short, long)]
    shout: bool,
}

fn main() {
    let cli = Cli::parse();
    let name = cli.name.as_deref().unwrap_or("world");
    if cli.shout {
        println!("HELLO, {}!", name.to_uppercase());
    } else {
        println!("Hello, {name}.");
    }
}
