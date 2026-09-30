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
// The example needs the `derive` feature.
#![cfg_attr(feature = "derive", doc = "```")]
#![cfg_attr(not(feature = "derive"), doc = "```ignore")]
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

pub mod color;
pub mod combinator;
pub mod env;
pub mod error;
pub mod help;
pub mod stream;
pub mod token;
pub mod value;

pub use env::with_env;
pub use error::{Error, ErrorKind};
pub use stream::{Argv, words};
pub use token::Arg;
pub use value::{ChoiceError, FromArg};

#[cfg(feature = "derive")]
pub use winnow_args_derive::{Args, Subcommand, ValueEnum};

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

    /// This command's help, rendered only when asked for.
    const HELP: &'static help::Command = &help::Command {
        name: "",
        about: "",
        long_about: "",
        after_help: "",
        after_long_help: "",
        items: &[],
        subcommands: &[],
        subcommand_required: false,
        help_flag: false,
        version: None,
    };

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

    /// Parse the process's arguments. Help and version are printed with exit
    /// status 0; a failure is reported on stderr with status 2.
    fn parse() -> Self {
        let mut args = std::env::args_os();
        let program = Self::HELP.name;
        let argv0 = args.next().unwrap_or_default();
        let program = if program.is_empty() {
            std::path::Path::new(&argv0)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            program.to_owned()
        };
        let args: Vec<_> = args.collect();
        match Self::parse_from(&words(&args)) {
            Ok(parsed) => parsed,
            Err(error) => std::process::exit(report(&error, &program)),
        }
    }
}

/// Print what a failed parse means, and give the exit status: help and version
/// on stdout with 0, a bare `arg_required_else_help` invocation's help on stderr
/// with 2, anything else as an error on stderr with 2. Colored with
/// [`color::Theme::DEFAULT`] where the stream written to shows color; see
/// [`report_with`] for another theme.
pub fn report(error: &Error, program: &str) -> i32 {
    report_with(error, program, &color::Theme::DEFAULT)
}

/// [`report`], painted with `theme`: its palette for the depth
/// [`color::Depth::detect`] finds on the stream written to.
pub fn report_with(error: &Error, program: &str, theme: &color::Theme) -> i32 {
    use std::io::IsTerminal as _;
    let stdout = || help::Style::themed(theme, std::io::stdout().is_terminal());
    let stderr = || help::Style::themed(theme, std::io::stderr().is_terminal());
    // Help goes to stdout, except a bare invocation's, which is an error.
    let bare = error.is_bare_help();
    let style = if bare { stderr() } else { stdout() };
    if let Some(help) = error.render_help_styled(program, style) {
        if bare {
            eprintln!("{help}");
            return 2;
        }
        println!("{help}");
        return 0;
    }
    if let Some(version) = error.version_text(program) {
        println!("{version}");
        return 0;
    }
    eprintln!("{}", error.render(stderr()));
    2
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

    /// Whether `-letter` is one of these flags, and if so whether it takes a
    /// value. A command with `unknown_flags = "value"` asks about every letter
    /// of a bundle before binding any.
    fn short(&self, letter: char) -> Option<bool> {
        let _ = letter;
        None
    }
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

/// [`Globals`] that are `shorts` (letter, takes a value) and `bind`, then
/// `parent`'s: what the derive hands a subcommand of a struct with global flags.
pub fn inherit<'a, F>(
    shorts: &'static [(char, bool)],
    parent: &'a mut dyn Globals,
    bind: F,
) -> impl Globals + 'a
where
    F: for<'i> FnMut(&Arg<'i>, &mut Argv<'i>, &mut dyn Globals) -> Result<bool, Error> + 'a,
{
    struct Inherit<'a, F> {
        shorts: &'static [(char, bool)],
        parent: &'a mut dyn Globals,
        bind: F,
    }
    impl<F> Globals for Inherit<'_, F>
    where
        F: for<'i> FnMut(&Arg<'i>, &mut Argv<'i>, &mut dyn Globals) -> Result<bool, Error>,
    {
        #[inline]
        fn bind<'i>(&mut self, arg: &Arg<'i>, input: &mut Argv<'i>) -> Result<bool, Error> {
            (self.bind)(arg, input, &mut *self.parent)
        }

        fn short(&self, letter: char) -> Option<bool> {
            match self.shorts.iter().find(|(c, _)| *c == letter) {
                Some(&(_, takes_value)) => Some(takes_value),
                None => self.parent.short(letter),
            }
        }
    }
    Inherit {
        shorts,
        parent,
        bind,
    }
}

/// Help prose the derive emits: kept with the `help-text` feature.
#[cfg(feature = "help-text")]
#[doc(hidden)]
#[macro_export]
macro_rules! __text {
    ($text:literal) => {
        $text
    };
}

/// Help prose the derive emits: `""` without the `help-text` feature.
#[cfg(not(feature = "help-text"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __text {
    ($text:literal) => {
        ""
    };
}

#[doc(hidden)]
pub mod __private {
    pub use crate::error::Error;
    pub use crate::stream::Argv;
    pub use crate::{Globals, Subcommand, globals, inherit};

    /// `help a b …`: the long help of the command the words name, below `root`.
    pub fn help_word(root: &'static crate::help::Command, input: &mut Argv<'_>) -> Error {
        let mut words = Vec::new();
        while !input.is_empty() && !is_flag_like(input.front()) {
            words.push(input.take_word());
        }
        let (command, path) = crate::help::resolve(root, words.iter().map(|w| &***w));
        path.into_iter()
            .rev()
            .fold(Error::help(command, true), Error::within)
    }

    /// Whether every letter of the short bundle `token` (`-abc`) is `known`,
    /// up to the first that takes a value: the rest is that value.
    pub fn bundle_known(token: &[u8], known: impl Fn(char) -> Option<bool>) -> bool {
        let Some(mut rest) = token.strip_prefix(b"-") else {
            return false;
        };
        // Letters are ASCII in practice: decode only from the first byte that is not.
        while let Some((&byte, tail)) = rest.split_first() {
            if !byte.is_ascii() {
                let Ok(letters) = core::str::from_utf8(rest) else {
                    return false;
                };
                return letters.chars().try_for_each(|letter| match known(letter) {
                    None => Err(false),
                    Some(true) => Err(true),
                    Some(false) => Ok(()),
                }) != Err(false);
            }
            match known(byte as char) {
                None => return false,
                Some(true) => return true,
                Some(false) => rest = tail,
            }
        }
        true
    }

    /// A short bundle of two or more letters naming one no flag answers to,
    /// taken whole as a word: `unknown_flags = "value"`. `None`, consuming
    /// nothing, for anything else; a single unknown letter binds nothing before
    /// it, so the derive's fallback arm handles it without this check.
    #[inline]
    pub fn unknown_bundle<'i>(
        input: &mut Argv<'i>,
        known: impl Fn(char) -> Option<bool>,
    ) -> Option<crate::token::Word<'i>> {
        let front = input.front();
        let short = input.mode() == crate::stream::Mode::Word
            && front.len() > 2
            && front[0] == b'-'
            && front[1] != b'-';
        if !short || bundle_known(front, known) {
            return None;
        }
        let offset = input.offset();
        Some(crate::token::Word {
            value: input.take_word(),
            offset,
            after_separator: false,
        })
    }

    fn is_flag_like(word: &[u8]) -> bool {
        word.len() > 1 && word[0] == b'-'
    }

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
    pub use crate::env;
    pub use crate::token::{Arg, ValueOptions, Word, arg, long_only, number, split};
    pub use crate::value::{ChoiceError, FromArg};
    pub use winnow::stream::BStr;
    pub type BoxError = crate::error::BoxError;
}
