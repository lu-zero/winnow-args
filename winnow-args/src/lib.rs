//! Command line argument parsing built from [winnow] parsers.
//!
//! The command line is read as a slice of [`BStr`](winnow::stream::BStr) words
//! through [`Argv`], a winnow [`Stream`](winnow::stream::Stream). Everything
//! else is a winnow parser over it, in three layers:
//!
//! - [`token`]: the lexer. [`token::arg`] reads one item (long flag, short
//!   letter, word, `--`); [`token::Arg`]'s methods finish a flag once its
//!   caller knows whether it takes a value. Matched in a `dispatch!`, the
//!   flags' names become a compiled `match`.
//! - [`combinator`]: bpaf-style named items — `short('p').long("path").argument()` —
//!   that compose with `alt`, `map` and friends.
//! - [`Args`] and its derive: a struct parsed by one generated loop that
//!   `match`es each lexed item against the struct's flags.
//!
//! ```
//! use std::path::PathBuf;
//! use winnow_args::Args;
//!
//! #[derive(Args, Debug)]
//! struct Cli {
//!     /// -v, --verbose
//!     #[arg(short, long)]
//!     verbose: bool,
//!     /// -p, --path <PATH>
//!     #[arg(short, long)]
//!     path: Option<PathBuf>,
//! }
//!
//! let cli = Cli::try_parse_from(["-v", "--path=/tmp"])?;
//! assert!(cli.verbose);
//! assert_eq!(cli.path, Some(PathBuf::from("/tmp")));
//! # Ok::<(), winnow_args::Error>(())
//! ```

pub mod combinator;
pub mod error;
pub mod stream;
pub mod token;
pub mod value;

pub use error::{Error, ErrorKind};
pub use stream::{Argv, words};
pub use value::FromArg;

#[cfg(feature = "derive")]
pub use winnow_args_derive::{Args, Subcommand};

/// A type parsed from a whole command line.
pub trait Args: Sized {
    /// Parse the command line in `input`, consuming all of it.
    ///
    /// This is a winnow parser: `Cli::parse_argv` can be passed anywhere a
    /// `Parser<Argv, Cli, Error>` is expected.
    fn parse_argv(input: &mut Argv<'_>) -> Result<Self, Error>;

    /// Parse `words`, which should not include the program name.
    fn parse_from(words: &[&winnow::stream::BStr]) -> Result<Self, Error> {
        Self::parse_argv(&mut Argv::new(words))
    }

    /// Parse `args`, which should not include the program name.
    fn try_parse_from<I, S>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let args: Vec<S> = args.into_iter().collect();
        Self::parse_from(&words(&args))
    }

    /// Parse the process's arguments, exiting with a message on failure.
    fn parse() -> Self {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        match Self::parse_from(&words(&args)) {
            Ok(parsed) => parsed,
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
    }
}

/// An enum of subcommands, selected by a word.
///
/// Derived with `#[derive(Subcommand)]`; a struct holds one in an
/// `#[arg(subcommand)]` field. The derive also implements [`Args`] for the enum,
/// so it can be the whole command line.
pub trait Subcommand: Sized {
    /// Whether `name` names a subcommand. Asked of every eligible word, so kept
    /// apart from [`Subcommand::parse_subcommand`] and cheap to inline.
    fn has(name: &[u8]) -> bool;

    /// Parse the rest of `input` as the subcommand `name`, which [`Subcommand::has`].
    fn parse_subcommand(name: &[u8], input: &mut Argv<'_>) -> Result<Self, Error>;
}

#[doc(hidden)]
pub mod __private {
    pub use crate::Subcommand;
    pub use crate::error::Error;
    pub use crate::stream::Argv;
    pub use crate::token::finish;
    pub use crate::token::{Arg, arg};
}
