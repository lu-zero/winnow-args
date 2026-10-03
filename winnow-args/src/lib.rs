//! Command line argument parsing built on [winnow].
//!
//! Describe the command line as a struct, and the derive writes its parser:
//!
// The example needs the `derive` feature.
#![cfg_attr(feature = "derive", doc = "```")]
#![cfg_attr(not(feature = "derive"), doc = "```ignore")]
//! use std::path::PathBuf;
//! use winnow_args::Args;
//!
//! /// Look somewhere.
//! #[derive(Args, Debug)]
//! struct Cli {
//!     /// Say more.
//!     #[arg(short, long)]
//!     verbose: bool,
//!     /// Where to look.
//!     #[arg(short, long)]
//!     path: Option<PathBuf>,
//! }
//!
//! let cli = Cli::parse_from(["-v", "--path=/tmp"])?;
//! assert!(cli.verbose);
//! assert_eq!(cli.path, Some(PathBuf::from("/tmp")));
//! # Ok::<(), winnow_args::Error>(())
//! ```
//!
//! # How a command line is parsed
//!
//! 1. **Words.** The command line is the list of words the shell made
//!    (`-v`, `--path=/tmp`, `a.txt`). [`Argv`] walks them in place: no word is
//!    copied, and a word need not be UTF-8.
//! 2. **Items.** The lexer, [`token::arg`], reads the next item: a long flag
//!    (`--path`, with its `=value` if attached), one short flag (each letter of
//!    `-vq` in turn), a plain word, or the `--` separator.
//! 3. **Meaning.** The parser, which knows the program's flags, says what the
//!    item is: `-v` sets a switch; `--path` takes a value, attached or the next
//!    word; a plain word is a positional or names a subcommand. An item it
//!    does not know is an error.
//! 4. **Result.** When the words run out, defaults and environment variables
//!    fill what was not given, the rules are checked (required, conflicts),
//!    and the struct is built.
//!
//! # Two ways to write the parser
//!
//! Steps 1, 2 and 4 are the same for everyone. Step 3 can be written by the
//! derive or by hand.
//!
//! **The derive** is what most programs want. Each field is a flag or a
//! positional; its type says how many values it holds and its attributes how
//! it is spelled. `#[derive(Args)]` turns that into one loop over the items
//! with a `match` on their names, so finding a flag costs the same however
//! many the program has.
//!
//! **The [combinators](combinator)** are for parsers built without a macro.
//! `short('p').long("path")` names a flag and `.argument()` or `.switch()`
//! makes it a winnow parser for one occurrence, to combine with winnow's own
//! `alt`, `map` and `repeat`. Flags are tried in turn, so it is slower than
//! the derive; both use the same lexer, values and errors.
//!
//! # Entry points
//!
//! - [`Args::completion_request`] before the others, for a program with
//!   [`complete`] scripts.
//! - [`Args::parse`] in `main`: help, version and errors are printed and the
//!   process exits (0, or 2 on failure), as [`report`] does.
//! - [`Args::try_parse`] in a `main` that handles them itself: the same,
//!   returning the [`Error`]; [`Args::program`] is the name to report it under.
//! - [`Args::parse_from`] in tests: the arguments after the program name, and
//!   an [`Error`] back. Help ([`ErrorKind::HelpRequested`]) and version
//!   ([`ErrorKind::VersionRequested`]) are `Err` too, so hand it to [`report`]
//!   rather than `?` it out of `main`.
//! - [`Args::parse_from_argv`] for a whole command line, program name first.
//! - [`Args::parse_words`] when the words are already [`BStr`]s, as after
//!   [`response::expand`].
//!
//! # From clap and usage
//!
//! The spellings that differ. Everything else is on the derive's own page.
//!
//! | | clap | usage | winnow-args |
//! |---|---|---|---|
//! | the command | `#[derive(Parser)]` | `#[derive(Cli)]` | `#[derive(Args)]` |
//! | its name, its version | `#[command(name, version)]` | `#[usage(bin, version)]` | `#[arg(name, version)]` |
//! | an attribute | `#[arg]`, `#[command]` | `#[usage]` | `#[arg]` |
//! | subcommands | `#[derive(Subcommand)]` | `#[derive(Subcommands)]` | `#[derive(Subcommand)]` |
//! | in `main` | `parse()` | `parse()` | `parse()` |
//! | words after the program name | `try_parse_from`, name first | `parse_from` | `parse_from` |
//! | the whole line | `try_parse_from(["prog", …])` | `parse_from_argv` | `parse_from_argv` |
//! | a count | `action = ArgAction::Count` | `count` | `count` |
//! | a positional | no `long` or `short` | no `short` or `long` | `#[arg(positional)]` |
//! | a field with no attribute | a positional | a positional | `--field-name` |
//! | a default, an environment variable | `default_value`, `env` | `default`, `env` | `default`, `env` |
//! | several words, a delimiter | `num_args`, `value_delimiter` | `num_args`, `delimiter` | `values`, `delimiter` |
//! | a value that may be left off | `default_missing_value` | `default_missing` | `default_missing` |
//! | choices from an enum | `#[arg(value_enum)]` | `#[usage(value_enum)]` | the type derives `ValueEnum` |
//! | a conflict | `conflicts_with = "file"` | `conflicts("--file")` | `conflicts("--file")` |
//! | the flag that clears a switch | `ArgAction::SetFalse` | `negate = "--no-force"` | `negate = "no-force"` |
//! | shared flags | `#[command(flatten)]` | `#[usage(flatten)]` | `#[arg(flatten)]` |
//! | a subcommand field | `#[command(subcommand)]` | `#[usage(subcommand)]` | `#[arg(subcommand)]` |
//! | an unknown flag | an error | a value, unless `unknown_flags = "error"` | an error, unless `unknown_flags = "value"` |
//!
//! A repeated single-value flag keeps the last value, and a long name is
//! matched in full.
//!
//! # Derive attributes
//!
//! The derives read `#[arg(...)]`, or `#[winnow_args(...)]` when another derive
//! claims `arg`. The page of each derive macro lists every attribute it accepts:
//! `Args` (struct options, field roles such as `flatten`, flag names, values,
//! positionals, rules, help), `Subcommand`, `ValueEnum`,
//! and `Occurrence` for tools whose flags mean something by their order
//! (a linker's `--as-needed a.o`).
//!
//! # Supporting modules
//!
//! - [`value`]: [`FromArg`] and the value types [`value::CInt`] (C-syntax integers), [`value::KeyValue`]
//!   (`key=value`), [`value::Parsed`] (any `FromStr`) and [`value::Spanned`] (a value with its offset).
//! - [`help`], [`color`]: the help data the derive emits, and how it is rendered and painted.
//! - [`complete`]: completion scripts for bash, zsh, fish, elvish and PowerShell that call the program back.
//! - [`response`]: `@file` response files, nested up to 10 deep and 4096 files in all.
//! - [`mod@env`]: [`with_env`], a fixed environment for deterministic tests.

pub mod color;
pub mod combinator;
pub mod complete;
pub mod env;
pub mod error;
pub mod help;
pub mod response;
pub mod stream;
pub mod token;
pub mod value;

pub use env::with_env;
pub use error::{Error, ErrorKind};
pub use stream::{Argv, words};
pub use token::Arg;
pub use value::{ChoiceError, FromArg};
/// A word of the command line, and a flag's value: bytes, not necessarily
/// UTF-8. What [`FromArg::from_arg`] is given.
pub use winnow::stream::BStr;

/// A command line as a struct: a field per flag or word.
///
/// ```
/// use winnow_args::Args;
///
/// /// Copy files.
/// #[derive(Args, Debug)]
/// #[arg(name = "cp", version)]
/// struct Cp {
///     /// Explain what is done; repeat for more.
///     #[arg(short, long, count)]
///     verbose: u8,
///     /// Overwrite without asking.
///     #[arg(short, long, negate)]
///     force: bool,
///     /// Attributes to keep.
///     #[arg(long, delimiter = ',')]
///     preserve: Vec<String>,
///     /// Jobs to run.
///     #[arg(short, long, env = "CP_JOBS", default = "1")]
///     jobs: usize,
///     #[arg(positional)]
///     source: String,
///     #[arg(positional)]
///     dest: Option<String>,
/// }
///
/// let cp = Cp::parse_from(["-vvf", "--preserve=mode,links", "a", "--no-force", "b"])?;
/// assert_eq!((cp.verbose, cp.force, cp.jobs), (2, false, 1));
/// assert_eq!(cp.preserve, ["mode", "links"]);
/// assert_eq!((cp.source.as_str(), cp.dest.as_deref()), ("a", Some("b")));
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
/// What a field's type means:
///
/// | Type | Flag | Positional |
/// |---|---|---|
/// | `bool` | a switch | |
/// | integer, with `count` | how many times it was given | |
/// | `T` | one value; an error if absent and without `env` or `default` | a required word |
/// | `Option<T>` | one value, or `None` | an optional word |
/// | `Vec<T>` | one value per occurrence | every word left |
/// | `Option<bool>`, with `plus` | `-x` is `Some(true)`, `+x` `Some(false)` | |
///
#[cfg(feature = "derive")]
pub use winnow_args_derive::Args;

/// Subcommands as an enum: a variant per word that selects one.
///
/// ```
/// use winnow_args::{Args, Subcommand};
///
/// #[derive(Args, Debug, PartialEq)]
/// struct Add {
///     #[arg(positional)]
///     paths: Vec<String>,
/// }
///
/// #[derive(Subcommand, Debug, PartialEq)]
/// enum Command {
///     /// Stage files.
///     Add(Add),
///     /// Show what changed.
///     #[arg(alias = "st")]
///     Status,
/// }
///
/// #[derive(Args, Debug)]
/// struct Git {
///     /// Say less.
///     #[arg(short, long, global)]
///     quiet: bool,
///     #[arg(subcommand)]
///     command: Command,
/// }
///
/// let git = Git::parse_from(["add", "-q", "a", "b"])?;
/// assert!(git.quiet);
/// assert_eq!(git.command, Command::Add(Add { paths: vec!["a".into(), "b".into()] }));
/// assert_eq!(Git::parse_from(["st"])?.command, Command::Status);
///
/// match git.command {
///     Command::Add(Add { paths }) => println!("staging {paths:?}"),
///     Command::Status => println!("on branch main"),
/// }
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
#[cfg(feature = "derive")]
pub use winnow_args_derive::Subcommand;

/// Flags and words kept in the order given, as an enum's variants.
///
/// ```
/// use winnow_args::{Args, Occurrence};
///
/// #[derive(Occurrence, Debug, PartialEq)]
/// enum Item {
///     /// Link what follows only if needed.
///     #[arg(long)]
///     AsNeeded,
///     /// Search for a library.
///     #[arg(short = 'l', prefix)]
///     Library(String),
///     #[arg(positional, value_name = "FILE")]
///     Input(String),
/// }
///
/// #[derive(Args, Debug)]
/// struct Ld {
///     #[arg(short, long)]
///     output: Option<String>,
///     #[arg(sequence)]
///     items: Vec<Item>,
/// }
///
/// let ld = Ld::parse_from(["a.o", "--as-needed", "-lm", "-o", "out", "b.o"])?;
/// assert_eq!(ld.output.as_deref(), Some("out"));
/// assert_eq!(
///     ld.items,
///     [
///         Item::Input("a.o".into()),
///         Item::AsNeeded,
///         Item::Library("m".into()),
///         Item::Input("b.o".into()),
///     ]
/// );
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
#[cfg(feature = "derive")]
pub use winnow_args_derive::Occurrence;

/// A value that is one of an enum's variants.
///
/// ```
/// use winnow_args::{Args, ErrorKind, ValueEnum};
///
/// #[derive(ValueEnum, Debug, PartialEq)]
/// enum Color {
///     Always,
///     #[arg(alias = "no")]
///     Never,
///     #[arg(name = "auto")]
///     WhenTerminal,
/// }
///
/// #[derive(Args, Debug)]
/// struct Cli {
///     #[arg(long)]
///     color: Option<Color>,
/// }
///
/// assert_eq!(Cli::parse_from(["--color=auto"])?.color, Some(Color::WhenTerminal));
/// assert_eq!(Cli::parse_from(["--color", "no"])?.color, Some(Color::Never));
/// let error = Cli::parse_from(["--color=red"]).unwrap_err();
/// assert_eq!(error.kind(), ErrorKind::InvalidChoice);
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
#[cfg(feature = "derive")]
pub use winnow_args_derive::ValueEnum;

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
        help_short: false,
        version: None,
    };

    /// Answer a completion script's callback (`PROG __complete_word__ …`)
    /// from this command's help data; `None` when `args` (the command line
    /// after the program's name) is not one. See [`complete`].
    fn completion_request(args: &[std::ffi::OsString]) -> Option<String> {
        complete::answer(Self::HELP, args)
    }

    /// The completion script for `shell`, calling back the program named in
    /// the help data (`#[arg(name = "…")]`).
    ///
    /// # Panics
    ///
    /// If the command has no name, or one that is not a plain word.
    fn completion_script(shell: complete::Shell) -> String {
        complete::script(Self::HELP.name, shell)
    }

    /// Parse `words`, the arguments after the program name, already borrowed
    /// as [`BStr`]s: what [`response::expand`] and [`words`] give. Nothing is
    /// copied.
    fn parse_words(words: &[&BStr]) -> Result<Self, Error> {
        Self::parse_argv(&mut Argv::new(words))
    }

    /// Parse `argv`, the whole command line, program name first. The name is
    /// skipped.
    fn parse_from_argv<I, S>(argv: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        Self::parse_from(argv.into_iter().skip(1))
    }

    /// Parse `args`, the arguments after the program name.
    ///
    /// ```
    /// # #[cfg(feature = "derive")] {
    /// use winnow_args::{Args, ErrorKind};
    ///
    /// #[derive(Args)]
    /// struct Cli {
    ///     #[arg(short, long)]
    ///     verbose: bool,
    /// }
    ///
    /// assert!(Cli::parse_from(["-v"]).unwrap().verbose);
    /// assert!(Cli::parse_from_argv(["prog", "-v"]).unwrap().verbose);
    /// let error = Cli::parse_from(["--loud"]).err().unwrap();
    /// assert_eq!(error.kind(), ErrorKind::UnknownFlag);
    /// # }
    /// ```
    fn parse_from<I, S>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let args: Vec<S> = args.into_iter().collect();
        Self::parse_words(&words(&args))
    }

    /// Parse the process's arguments; on help, version or a failure, print
    /// and exit as [`report`] does. It answers no completion callback: call
    /// [`Args::completion_request`] first if the program has [`complete`] scripts.
    fn parse() -> Self {
        match Self::try_parse() {
            Ok(parsed) => parsed,
            Err(error) => std::process::exit(report(&error, &Self::program())),
        }
    }

    /// Parse the process's arguments, leaving help, version and failures to
    /// the caller: [`Args::parse`] without the printing and the exit.
    ///
    /// ```no_run
    /// # #[cfg(feature = "derive")] {
    /// use winnow_args::Args;
    ///
    /// #[derive(Args)]
    /// struct Cli {
    ///     #[arg(short, long)]
    ///     verbose: bool,
    /// }
    ///
    /// let cli = match Cli::try_parse() {
    ///     Ok(cli) => cli,
    ///     Err(error) => std::process::exit(winnow_args::report(&error, &Cli::program())),
    /// };
    /// # let _ = cli.verbose;
    /// # }
    /// ```
    fn try_parse() -> Result<Self, Error> {
        Self::parse_from_argv(std::env::args_os())
    }

    /// The program's name, as messages give it: `#[arg(name = "…")]`, else
    /// the file name the process was run as.
    fn program() -> String {
        if !Self::HELP.name.is_empty() {
            return Self::HELP.name.to_owned();
        }
        let argv0 = std::env::args_os().next().unwrap_or_default();
        std::path::Path::new(&argv0)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
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
///
/// ```
/// use winnow_args::color::{Paint, Palette, Theme};
///
/// // Bold, uncolored headers on 16-color terminals; the default elsewhere.
/// const THEME: Theme = Theme {
///     ansi16: Palette { header: Paint::NONE.bold(), ..Palette::DEFAULT },
///     ..Theme::DEFAULT
/// };
/// # let _ = |e: &winnow_args::Error| winnow_args::report_with(e, "prog", &THEME);
/// ```
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

/// Flags and words kept in command-line order: an enum deriving `Occurrence`,
/// one variant per flag or positional, collected by an `#[arg(sequence)]`
/// `Vec` field of an [`Args`] struct.
///
/// For command lines where position is meaning, as `ld`'s: `--as-needed` and
/// `--whole-archive` apply to the inputs after them, so the linker folds the
/// sequence rather than reading fields.
pub trait Occurrence: Sized {
    /// Letters that always take the rest of their word (`-lfoo`), for a
    /// `long_only` parent: no single-dash long name starting with one is tried.
    const PREFIXES: &'static [u8] = &[];

    /// `arg` as one of the variants, its value read from `input`; `None` when
    /// no variant is spelled so.
    fn from_arg<'i>(arg: &Arg<'i>, input: &mut Argv<'i>) -> Result<Option<Self>, Error>;

    /// A word as the positional variant; `None` if there is none.
    fn from_word(word: &token::Word<'_>) -> Result<Option<Self>, Error>;

    /// Whether `name` is a variant's long name that may be spelled with one
    /// dash, for a `long_only` parent.
    fn is_long(name: &[u8]) -> bool;

    /// The variants as help lists them, after the parent's own flags.
    const ITEMS: &'static [help::Item] = &[];

    /// Whether there is a positional variant: it takes every word.
    const POSITIONAL: bool = false;

    /// Whether there is an `#[arg(bundle)]` variant.
    const BUNDLES: bool = false;

    /// A word of several short flags (`-sS`), whole, as the `#[arg(bundle)]`
    /// variant: an item put before those of its letters, whoever takes them,
    /// for a program that reports grouping (GNU ld deprecates it). `None` if
    /// there is none.
    fn from_bundle(word: &token::Word<'_>) -> Result<Option<Self>, Error> {
        let _ = word;
        Ok(None)
    }

    /// A flag no variant names, as the `#[arg(unknown)]` variant; `None` if
    /// there is none. Asked by a parent whose field is
    /// `#[arg(sequence, unknown)]`.
    fn from_unknown(word: &token::Word<'_>) -> Result<Option<Self>, Error> {
        let _ = word;
        Ok(None)
    }

    /// Whether `letter` is a variant's short flag, and whether it takes a
    /// value: how a parent with an `unknown` field tells an unknown bundle.
    fn short(letter: char) -> Option<bool> {
        let _ = letter;
        None
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

/// [`Globals`] from a closure. It does not answer [`Globals::short`]: the derive
/// calls `inherit`, which forwards the parent's letters, so a lenient
/// subcommand still sees them.
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
/// Unlike [`globals`], `short` sees those letters.
#[doc(hidden)]
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
    pub use crate::env;
    pub use crate::error::Error;
    pub use crate::help::concat_items;
    pub use crate::stream::Argv;
    pub use crate::token::{Arg, ValueOptions, Word, arg, arg_plus, long_only, number, split};
    pub use crate::value::{ChoiceError, FromArg};
    pub use crate::{Globals, Subcommand, globals, inherit};
    pub use winnow::stream::BStr;
    pub type BoxError = crate::error::BoxError;

    use crate::token::is_flag_like;

    /// The flags of a struct that another flattens (`#[arg(flatten)]`), derived
    /// for every `Args` struct that has flags only.
    ///
    /// The parent keeps a [`Flatten::Slots`], offers it each flag its own
    /// arms do not take, and builds the value at the end.
    #[diagnostic::on_unimplemented(
        message = "`{Self}` cannot be flattened",
        note = "a flattened struct derives `Args` and has only flags: no positionals, \
                subcommand, `sequence`, `unknown`, `keywords`, `global` flags, \
                `unknown_flags = \"value\"` or generics"
    )]
    pub trait Flatten: Sized {
        /// What has been parsed so far.
        type Slots: Default;

        /// Bind `arg` if it is one of these flags; `Ok(false)` if not.
        fn bind<'i>(
            slots: &mut Self::Slots,
            arg: &Arg<'i>,
            input: &mut Argv<'i>,
        ) -> Result<bool, Error>;

        /// Whether `-letter` is one of these flags, and whether it takes a value.
        fn short(letter: char) -> Option<bool>;

        /// Whether `name` is a long name that may take one dash (`long_only`).
        fn is_long(name: &[u8]) -> bool;

        /// Whether `-letter` always takes the rest of its word (`prefix`).
        fn is_prefix(letter: u8) -> bool;

        /// Apply environment variables, defaults and rules, and build the value.
        fn finish(slots: Self::Slots, input: &Argv<'_>) -> Result<Self, Error>;
    }

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
    ) -> Option<Word<'i>> {
        let front = input.front();
        let short = input.mode() == crate::stream::Mode::Word
            && front.len() > 2
            && front[0] == b'-'
            && front[1] != b'-';
        if !short || bundle_known(front, known) {
            return None;
        }
        let offset = input.offset();
        Some(Word {
            value: input.take_word(),
            offset,
            after_separator: false,
        })
    }

    /// A `#[arg(keywords)]` field: each value `flag` took (`-z now`) parsed as
    /// the `--now` flag of `T`, in order; an error is an invalid value of `flag`
    /// naming the keyword.
    pub fn keywords<T: crate::Args>(values: &[&BStr], flag: &str, at: usize) -> Result<T, Error> {
        let spelled: Vec<Vec<u8>> = values
            .iter()
            .map(|v| [b"--".as_slice(), v].concat())
            .collect();
        let words: Vec<&BStr> = spelled.iter().map(|w| BStr::new(w.as_slice())).collect();
        T::parse_words(&words).map_err(|e| {
            let keyword = e.token().map_or_else(String::new, |t| {
                t.strip_prefix("--").unwrap_or(t).to_owned()
            });
            Error::invalid_value(at, flag, keyword.as_bytes(), e)
        })
    }

    /// The end of a unit subcommand: only inherited global flags may follow,
    /// or a request for its `help`.
    pub fn finish_with(
        input: &mut Argv<'_>,
        globals: &mut dyn Globals,
        help: &'static crate::help::Command,
    ) -> Result<(), Error> {
        while !input.is_empty() {
            let arg = arg(input)?;
            match arg {
                Arg::Separator { .. } => {}
                Arg::Long(_) | Arg::Short(_) if globals.bind(&arg, input)? => {}
                Arg::Long(flag) if flag.name == b"help" => return Err(Error::help(help, true)),
                Arg::Short(flag) if flag.letter == 'h' => return Err(Error::help(help, false)),
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(())
    }
}
