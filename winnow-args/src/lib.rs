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
pub use token::Arg;
pub use value::FromArg;

#[cfg(feature = "derive")]
pub use winnow_args_derive::{Args, Subcommand};

/// A type parsed from a whole command line.
pub trait Args: Sized {
    /// Parse the command line in `input`, consuming all of it.
    ///
    /// This is a winnow parser: `Cli::parse_argv` can be passed anywhere a
    /// `Parser<Argv, Cli, Error>` is expected.
    fn parse_argv(input: &mut Argv<'_>) -> Result<Self, Error> {
        Self::parse_argv_with(input, &mut ())
    }

    /// [`Args::parse_argv`] as a subcommand: flags it does not declare itself
    /// are offered to `globals`, its ancestors' global flags.
    fn parse_argv_with(input: &mut Argv<'_>, globals: &mut dyn Globals) -> Result<Self, Error>;

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

    /// Parse the rest of `input` as the subcommand `name`, which [`Subcommand::has`];
    /// `globals` binds its ancestors' global flags.
    fn parse_subcommand(
        name: &[u8],
        input: &mut Argv<'_>,
        globals: &mut dyn Globals,
    ) -> Result<Self, Error>;
}

/// The global flags of a subcommand's ancestors, which it accepts as its own.
///
/// A parent hands one down to its subcommand; the subcommand offers it every
/// flag it does not declare, so its own declarations take precedence.
pub trait Globals {
    /// Bind `arg`, read from `input`, if it is a global flag; `Ok(false)` if not.
    fn bind<'i>(&mut self, arg: &Arg<'i>, input: &mut Argv<'i>) -> Result<bool, Error>;
}

/// No global flags: what a command line's root is given.
impl Globals for () {
    #[inline]
    fn bind<'i>(&mut self, _: &Arg<'i>, _: &mut Argv<'i>) -> Result<bool, Error> {
        Ok(false)
    }
}

/// [`Globals`] from a closure; the derive uses it for a struct's global flags.
pub fn globals<F>(bind: F) -> impl Globals
where
    F: for<'i> FnMut(&Arg<'i>, &mut Argv<'i>) -> Result<bool, Error>,
{
    struct FromFn<F>(F);
    impl<F> Globals for FromFn<F>
    where
        F: for<'i> FnMut(&Arg<'i>, &mut Argv<'i>) -> Result<bool, Error>,
    {
        #[inline]
        fn bind<'i>(&mut self, arg: &Arg<'i>, input: &mut Argv<'i>) -> Result<bool, Error> {
            (self.0)(arg, input)
        }
    }
    FromFn(bind)
}

#[doc(hidden)]
pub mod __private {
    pub use crate::error::Error;
    pub use crate::stream::Argv;
    pub use crate::{Globals, Subcommand, globals};

    /// The end of a unit subcommand: only inherited global flags may follow.
    pub fn finish_with(input: &mut Argv<'_>, globals: &mut dyn Globals) -> Result<(), Error> {
        while !input.is_empty() {
            let arg = arg(input)?;
            match arg {
                Arg::Separator { .. } => {}
                Arg::Long(_) | Arg::Short(_) if globals.bind(&arg, input)? => {}
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(())
    }
    pub use crate::token::{Arg, arg, split};
}
