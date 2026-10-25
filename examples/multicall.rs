//! Multicall and external subcommands, as a busybox-like `box` has them: run
//! as `ls`, `dir` or `echo`, it is that command; under any other name it reads
//! the command from its first word, and a word that names none is an external
//! command, as `cargo foo` runs `cargo-foo`.
//!
//!     cargo build --example multicall
//!     ln -s multicall target/debug/examples/ls
//!     target/debug/examples/ls -l /tmp                   # `box ls -l /tmp`
//!     target/debug/examples/ls --help                    # help names it `ls`
//!     cargo run --example multicall -- echo -n hi
//!     cargo run --example multicall -- frob --fast x     # an external command

use winnow_args::{Args, Subcommand};

/// Small tools in one binary.
#[derive(Args, Debug)]
#[arg(name = "box", version = "1.0.0", multicall)]
struct Multicall {
    #[arg(subcommand)]
    tool: Tool,
}

#[derive(Subcommand, Debug)]
enum Tool {
    /// List a directory.
    #[arg(alias = "dir")]
    Ls {
        /// One entry a line, with details.
        #[arg(short)]
        long: bool,
        /// The directories to list.
        #[arg(positional)]
        paths: Vec<String>,
    },
    /// Print the words given.
    Echo {
        /// Leave out the newline.
        #[arg(short)]
        n: bool,
        #[arg(positional)]
        words: Vec<String>,
    },
    /// Any other name runs `box-NAME`.
    #[arg(external)]
    Other(Vec<String>),
}

fn main() {
    match Multicall::parse().tool {
        Tool::Ls { long, paths } => println!("ls: long {long}, paths {paths:?}"),
        Tool::Echo { n, words } => print!("{}{}", words.join(" "), if n { "" } else { "\n" }),
        Tool::Other(words) => println!("would run box-{} with {:?}", words[0], &words[1..]),
    }
}
