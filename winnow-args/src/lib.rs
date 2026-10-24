//! Command line parsing built from [winnow] parsers.
//!
//! winnow-args parses a program's arguments into typed values, typically a
//! struct with one field per flag or positional. It distinguishes:
//!
//! - **flags**, long (`--verbose`) or short (`-v`); short flags can be bundled,
//!   `-vq` for `-v -q`;
//! - **values** of flags, given as the next word (`--path /tmp`, `-p /tmp`) or
//!   attached (`--path=/tmp`, `-p/tmp`);
//! - **positionals**, words identified by their position: `a` and `b` in
//!   `cp a b`;
//! - **subcommands**, words that select a command with arguments of its own:
//!   `add` in `git add -p`;
//! - `--`, after which every word is a positional.
//!
//! A failed parse returns an [`Error`] with its kind, the word at fault and its
//! offset.
//!
//! There are two ways to say which of these a program takes:
//!
//! - `#[derive(Args)]` on a struct. A field is a flag or a positional, its type
//!   says how many values it takes and what they convert to, and its doc comment
//!   is its help. Most programs use the derive.
//! - By hand, with [`combinator`]. A flag is a [winnow] parser, combined with
//!   `alt`, or with `dispatch!` on the flag's name.
//!
//! Either way the words are read where the shell left them: nothing is copied or
//! re-split, and the bytes need not be UTF-8 until a value's type asks for text.
//! Both also cover command lines older than `--long`: `+x` options as in a bash
//! builtin, and flags whose order matters as in `ld`.
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
//!     #[arg(positional)]
//!     files: Vec<String>,
//! }
//!
//! // In `main`: `let cli = Cli::parse();`
//! let cli = Cli::parse_from(["-v", "--path=/tmp", "a", "b"])?;
//! assert!(cli.verbose);
//! assert_eq!(cli.path, Some(PathBuf::from("/tmp")));
//! assert_eq!(cli.files, ["a", "b"]);
//! # Ok::<(), winnow_args::Error>(())
//! ```
//!
#![cfg_attr(
    feature = "derive",
    doc = "- [`derive@Args`], [`derive@Subcommand`], [`derive@ValueEnum`] and [`derive@Occurrence`].
  Each page lists that derive's attributes."
)]
//! - [`combinator`] is the parser written by hand. [`token`] is the lexer.
//! - [`FromArg`] reads a value's bytes as a type. [`value`] covers a `FromStr`
//!   type, a C-syntax number and `key=value`.
//! - [`Error`] is a failed parse. [`report`] prints it, and names a close
//!   spelling when there is one.
//! - [`help`] is the help text and [`color`] its theme. A `` `quoted` ``,
//!   `*emphasized*` or `**strong**` span is painted, each by its own paint in
//!   [`color::Palette`]. Backticks are kept, and stars are dropped where the
//!   span is painted.
//! - [`complete`] writes completion scripts for bash, zsh, fish, elvish and
//!   PowerShell.
//! - [`response`] reads `@file` words.
//! - [`run`] calls the command the line selected. `#[arg(run)]` is on the
//!   subcommand enum.
//! - [`with_env`] sets environment variables for one test.
//!
//! `winnow-args-man` and `winnow-args-markdown` build man and markdown pages
//! from what the derives write when `WINNOW_ARGS_SPEC` names a directory. The
//! derive crate's documentation says how.
//!
//! # Entry points
//!
//! - [`Args::parse`] prints help, version or a failure, and the process exits.
//! - [`Args::try_parse`] returns the [`Error`].
//! - [`Args::parse_from`] takes the words after the program name.
//! - [`Args::parse_from_argv`] takes the whole line, program name first.
//! - [`Args::parse_words`] takes words that are already [`BStr`]s.
//! - [`Args::completion_request`] answers one completion, before [`Args::parse`].
//!
//! # Performance
//!
//! The parser is generated at compile time, so a parse costs the same however
//! many commands the program declares. The measurements and the method are in
//! the repository's
//! [benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).
//!
//! # Cargo features
//!
//! - `derive` (default): the derives.
//! - `help-text` (default): the prose of derived help. Without it, help still
//!   lists commands, flags, values and defaults, and the binary is smaller.
//! - `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.
//! - `unicode`: measure help in terminal columns, so a wide character (CJK,
//!   emoji) takes two and a combining mark none. Without it, one a character.
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
//! | words after the program name | none: the name comes first | `parse_from` | `parse_from` |
//! | the whole line | `try_parse_from` | `parse_from_argv` | `parse_from_argv` |
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

pub mod color;
pub mod combinator;
pub mod complete;
pub mod env;
pub mod error;
pub mod help;
pub mod response;
pub mod run;
pub mod stream;
pub mod token;
pub mod value;
pub mod warn;

pub use env::with_env;
pub use error::{Error, ErrorKind};
pub use stream::{Argv, words};
pub use token::Arg;
pub use value::{ChoiceError, FromArg};
/// A word of the command line, and a flag's value: bytes, not necessarily
/// UTF-8. What [`FromArg::from_arg`] is given.
pub use winnow::stream::BStr;

#[cfg(feature = "derive")]
pub use winnow_args_derive::{Args, Occurrence, Subcommand, ValueEnum};

// The README's example is compiled as a doctest.
#[cfg(all(doctest, feature = "derive"))]
#[doc = include_str!("../README.md")]
pub struct Readme;

// The derive crate cannot call `parse_from`, so these examples are tested here.
#[cfg(all(doctest, feature = "derive"))]
pub mod derive_examples {
    //! ```
    //! use winnow_args::Args;
    //!
    //! /// Copy files.
    //! #[derive(Args, Debug)]
    //! #[arg(name = "cp", version)]
    //! struct Cp {
    //!     /// Explain what is done; repeat for more.
    //!     #[arg(short, long, count)]
    //!     verbose: u8,
    //!     /// Overwrite without asking.
    //!     #[arg(short, long, negate)]
    //!     force: bool,
    //!     /// Attributes to keep.
    //!     #[arg(long, delimiter = ',')]
    //!     preserve: Vec<String>,
    //!     /// Jobs to run.
    //!     #[arg(short, long, env = "CP_JOBS", default = "1")]
    //!     jobs: usize,
    //!     #[arg(positional)]
    //!     source: String,
    //!     #[arg(positional)]
    //!     dest: Option<String>,
    //! }
    //!
    //! let cp = Cp::parse_from(["-vvf", "--preserve=mode,links", "a", "--no-force", "b"])?;
    //! assert_eq!((cp.verbose, cp.force, cp.jobs), (2, false, 1));
    //! assert_eq!(cp.preserve, ["mode", "links"]);
    //! assert_eq!((cp.source.as_str(), cp.dest.as_deref()), ("a", Some("b")));
    //! # Ok::<(), winnow_args::Error>(())
    //! ```
    //!
    //! ```
    //! use winnow_args::{Args, Subcommand};
    //!
    //! #[derive(Args, Debug, PartialEq)]
    //! struct Add {
    //!     #[arg(positional)]
    //!     paths: Vec<String>,
    //! }
    //!
    //! #[derive(Subcommand, Debug, PartialEq)]
    //! enum Command {
    //!     /// Stage files.
    //!     Add(Add),
    //!     /// Show what changed.
    //!     #[arg(alias = "st")]
    //!     Status,
    //! }
    //!
    //! #[derive(Args, Debug)]
    //! struct Git {
    //!     /// Say less.
    //!     #[arg(short, long, global)]
    //!     quiet: bool,
    //!     #[arg(subcommand)]
    //!     command: Command,
    //! }
    //!
    //! let git = Git::parse_from(["add", "-q", "a", "b"])?;
    //! assert!(git.quiet);
    //! assert_eq!(git.command, Command::Add(Add { paths: vec!["a".into(), "b".into()] }));
    //! assert_eq!(Git::parse_from(["st"])?.command, Command::Status);
    //!
    //! match git.command {
    //!     Command::Add(Add { paths }) => println!("staging {paths:?}"),
    //!     Command::Status => println!("on branch main"),
    //! }
    //! # Ok::<(), winnow_args::Error>(())
    //! ```
    //!
    //! ```
    //! use winnow_args::{Args, Occurrence};
    //!
    //! #[derive(Occurrence, Debug, PartialEq)]
    //! enum Item {
    //!     /// Link what follows only if needed.
    //!     #[arg(long)]
    //!     AsNeeded,
    //!     /// Search for a library.
    //!     #[arg(short = 'l', prefix)]
    //!     Library(String),
    //!     #[arg(positional, value_name = "FILE")]
    //!     Input(String),
    //! }
    //!
    //! #[derive(Args, Debug)]
    //! struct Ld {
    //!     #[arg(short, long)]
    //!     output: Option<String>,
    //!     #[arg(sequence)]
    //!     items: Vec<Item>,
    //! }
    //!
    //! let ld = Ld::parse_from(["a.o", "--as-needed", "-lm", "-o", "out", "b.o"])?;
    //! assert_eq!(ld.output.as_deref(), Some("out"));
    //! assert_eq!(
    //!     ld.items,
    //!     [
    //!         Item::Input("a.o".into()),
    //!         Item::AsNeeded,
    //!         Item::Library("m".into()),
    //!         Item::Input("b.o".into()),
    //!     ]
    //! );
    //! # Ok::<(), winnow_args::Error>(())
    //! ```
    //!
    //! ```
    //! use winnow_args::{Args, ErrorKind, ValueEnum};
    //!
    //! #[derive(ValueEnum, Debug, PartialEq)]
    //! enum Color {
    //!     Always,
    //!     #[arg(alias = "no")]
    //!     Never,
    //!     #[arg(name = "auto")]
    //!     WhenTerminal,
    //! }
    //!
    //! #[derive(Args, Debug)]
    //! struct Cli {
    //!     #[arg(long)]
    //!     color: Option<Color>,
    //! }
    //!
    //! assert_eq!(Cli::parse_from(["--color=auto"])?.color, Some(Color::WhenTerminal));
    //! assert_eq!(Cli::parse_from(["--color", "no"])?.color, Some(Color::Never));
    //! let error = Cli::parse_from(["--color=red"]).unwrap_err();
    //! assert_eq!(error.kind(), ErrorKind::InvalidChoice);
    //! # Ok::<(), winnow_args::Error>(())
    //! ```
}

/// A type parsed from a whole command line.
pub trait Args: Sized {
    /// The name the derive files this type under when the documentation is
    /// built. Empty for an implementation written by hand.
    #[doc(hidden)]
    const SPEC: &'static str = "";

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

    /// [`Args::HELP`], from where the type is. A constant is built again in
    /// each crate that names it, and help is large: the derive writes this
    /// beside the type, so a program in another crate has the one copy.
    #[doc(hidden)]
    fn help_data() -> &'static help::Command {
        Self::HELP
    }

    /// This command's help, rendered only when asked for.
    const HELP: &'static help::Command = &help::Command::EMPTY;

    /// Answer a completion script's callback (`PROG __complete_word__ …`)
    /// from this command's help data; `None` when `args` (the command line
    /// after the program's name) is not one. See [`complete`].
    fn completion_request(args: &[std::ffi::OsString]) -> Option<String> {
        complete::answer(Self::help_data(), args)
    }

    /// The completion script for `shell`, calling back the program named in
    /// the help data (`#[arg(name = "…")]`). A `multicall` command's also
    /// completes under each subcommand's name and shown aliases that are
    /// plain words, asking the program with the line as typed.
    ///
    /// # Panics
    ///
    /// If the command has no name, or one that is not a plain word.
    fn completion_script(shell: complete::Shell) -> String {
        let help = Self::help_data();
        if !help.multicall {
            return complete::script(help.name, shell);
        }
        // Each command it may be linked as, by every name help shows.
        let shown = help.subcommands.iter().filter(|sub| !sub.hide);
        let names = shown.flat_map(|sub| std::iter::once(&sub.name).chain(sub.aliases));
        let mut bins = vec![help.name];
        for &name in names {
            if complete::plain_name(name) && !bins.contains(&name) {
                bins.push(name);
            }
        }
        complete::scripts(&bins, shell)
    }

    /// Parse `words`, the arguments after the program name, already borrowed
    /// as [`BStr`]s: what [`response::expand`] and [`words`] give. Nothing is
    /// copied.
    ///
    /// An unknown flag or subcommand that is close to one of the program's
    /// has it as [`Error::similar`].
    fn parse_words(words: &[&BStr]) -> Result<Self, Error> {
        Self::parse_argv(&mut Argv::new(words)).map_err(|error| error.suggested(Self::help_data()))
    }

    /// Parse `argv`, the whole command line, program name first. The name is
    /// skipped, unless the command is `multicall` and the name is one of its
    /// subcommands': the line is then that subcommand's.
    fn parse_from_argv<I, S>(argv: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut argv = argv.into_iter();
        let first = argv.next();
        if Self::HELP.multicall
            && let Some(name) = first.and_then(|first| applet(Self::help_data(), first.as_ref()))
        {
            let rest: Vec<S> = argv.collect();
            let mut words = words(&rest);
            words.insert(0, BStr::new(name));
            return Self::parse_words(&words).map_err(|error| error.applet(name));
        }
        Self::parse_from(argv)
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
    ///
    /// The [`warn::Warning`]s due are printed on standard error first.
    fn parse() -> Self {
        let parsed = Self::try_parse();
        for warning in Self::warnings() {
            eprint!("{}", warning.render_styled(help::Style::auto_stderr()));
        }
        match parsed {
            Ok(parsed) => parsed,
            Err(error) => std::process::exit(report(&error, &Self::program())),
        }
    }

    /// What the parses on this thread used that this program says not to
    /// use any more, since the last call: deprecated flags, positionals and
    /// subcommands, and values read from deprecated variables. Those whose
    /// `deprecated_warn_at` is a later release than this program's `version`
    /// are left out. [`Args::parse`] prints them; after any other entry
    /// point the program reports them its own way, or not at all.
    fn warnings() -> Vec<warn::Warning> {
        warn::take(Self::help_data().version)
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
    /// the file name the process was run as. A `multicall` program run under
    /// a subcommand's name is that name.
    fn program() -> String {
        let argv0 = || std::env::args_os().next().unwrap_or_default();
        if Self::HELP.multicall
            && let Some(name) = applet(Self::help_data(), &argv0())
        {
            return name.to_owned();
        }
        if !Self::HELP.name.is_empty() {
            return Self::HELP.name.to_owned();
        }
        std::path::Path::new(&argv0())
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// The spelling of `command`'s subcommand that `argv0`, the program's path,
/// is named: `/bin/ls` and, on Windows, `ls.exe` and `LS.EXE` are `ls`.
pub(crate) fn applet(
    command: &'static help::Command,
    argv0: &std::ffi::OsStr,
) -> Option<&'static str> {
    let name = std::path::Path::new(argv0).file_name()?.to_str()?;
    // Windows names files in any case.
    let same = |a: &str, b: &str| a == b || (cfg!(windows) && a.eq_ignore_ascii_case(b));
    let suffix = std::env::consts::EXE_SUFFIX;
    let at = name.len().saturating_sub(suffix.len());
    let name = match name.get(at..) {
        Some(end) if same(end, suffix) => &name[..at],
        _ => name,
    };
    let mut names = command.subcommands.iter().flat_map(|sub| sub.names);
    names.find(|spelling| same(spelling, name)).copied()
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
    /// The name the derive files this type under when the documentation is
    /// built. Empty for an implementation written by hand.
    #[doc(hidden)]
    const SPEC: &'static str = "";

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

    /// Whether there is a positional variant: each plain word is that variant.
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
    /// The name the derive files this type under when the documentation is
    /// built. Empty for an implementation written by hand.
    #[doc(hidden)]
    const SPEC: &'static str = "";

    /// Whether `name` names a subcommand. Asked of every eligible word, so kept
    /// apart from [`Subcommand::parse_subcommand`] and cheap to inline.
    fn has(name: &[u8]) -> bool;

    /// Whether a word that names no subcommand, where nothing else takes
    /// it, is one all the same: an external subcommand, as `cargo foo` runs
    /// `cargo-foo`. [`Subcommand::parse_subcommand`] then takes any name.
    const EXTERNAL: bool = false;

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

/// [`Globals`] that are `shorts` (letter, takes a value), those `flattened`
/// answers for, and `bind`, then `parent`'s: what the derive hands a
/// subcommand of a struct with global flags, its own or of a struct it
/// flattens. Unlike [`globals`], `short` sees those letters.
#[doc(hidden)]
pub fn inherit<'a, F>(
    shorts: &'static [(char, bool)],
    flattened: fn(char) -> Option<bool>,
    parent: &'a mut dyn Globals,
    bind: F,
) -> impl Globals + 'a
where
    F: for<'i> FnMut(&Arg<'i>, &mut Argv<'i>, &mut dyn Globals) -> Result<bool, Error> + 'a,
{
    struct Inherit<'a, F> {
        shorts: &'static [(char, bool)],
        flattened: fn(char) -> Option<bool>,
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
                None => (self.flattened)(letter).or_else(|| self.parent.short(letter)),
            }
        }
    }
    Inherit {
        shorts,
        flattened,
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
    (@expr $text:expr) => {
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
    (@expr $text:expr) => {{
        let _: &'static str = $text;
        ""
    }};
}

#[doc(hidden)]
pub mod __private {
    pub use crate::env;
    pub use crate::error::Error;
    pub use crate::help::{concat_headings, concat_items};
    pub use crate::stream::Argv;
    pub use crate::token::{Arg, ValueOptions, Word, arg, arg_plus, long_only, number, split};
    pub use crate::value::{ChoiceError, FromArg, documented_choices, filed_as, stated_choices};
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
                subcommand, `sequence`, `unknown`, `keywords`, \
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

        /// Bind `arg` if it is one of these flags that is `global`: what a
        /// subcommand of the struct that flattens them is offered.
        fn bind_global<'i>(
            slots: &mut Self::Slots,
            arg: &Arg<'i>,
            input: &mut Argv<'i>,
        ) -> Result<bool, Error>;

        /// Whether `-letter` is one of these flags that is `global`, and
        /// whether it takes a value.
        fn global_short(letter: char) -> Option<bool>;

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

    /// `error`, a required flag or positional left without a value, added to
    /// those `found` already reports.
    #[cold]
    #[inline(never)]
    pub fn missing(found: &mut Option<Error>, error: Error) {
        *found = Some(match found.take() {
            Some(first) => first.also_missing(error),
            None => error,
        });
    }

    /// `error`, said with the program's `wording` in place of its default message.
    pub fn worded(error: Error, wording: &'static str) -> Error {
        error.worded(wording)
    }

    /// Whether every letter of the short bundle `token` (`-abc`) is `known`,
    /// up to the first that takes a value: the rest is that value.
    pub fn bundle_known(token: &[u8], known: impl Fn(char) -> Option<bool>) -> bool {
        let Some(mut rest) = token.strip_prefix(b"-") else {
            return false;
        };
        // Letters are ASCII in practice: the decoder is out of line.
        while let Some(&byte) = rest.first() {
            let (letter, width) = match byte {
                b if b.is_ascii() => (b as char, 1),
                _ => crate::token::decode_letter(rest),
            };
            match known(letter) {
                None => return false,
                Some(true) => return true,
                Some(false) => rest = &rest[width..],
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
            // The vocabulary's own check speaks for itself.
            if e.kind() == crate::ErrorKind::Invalid {
                return e;
            }
            let keyword = e.token().map_or_else(String::new, |t| {
                t.strip_prefix("--").unwrap_or(t).to_owned()
            });
            Error::invalid_value(at, flag, keyword.as_bytes(), e)
        })
    }

    /// Whether help prose is kept: the `help-text` feature.
    pub const HELP_TEXT: bool = cfg!(feature = "help-text");

    /// Whether `word` reads as a flag (`-x`, `--name`), and so names no
    /// external subcommand.
    #[inline]
    pub fn flag_like(word: &[u8]) -> bool {
        is_flag_like(word)
    }

    /// A deprecated flag or positional, `name` as help spells it, was given.
    #[cold]
    #[inline(never)]
    pub fn deprecated_argument(name: &str, deprecation: &'static crate::warn::Deprecation) {
        crate::warn::record(crate::warn::Warning {
            kind: crate::warn::WarningKind::Argument,
            name: name.to_owned(),
            deprecation: Some(*deprecation),
            replacement: None,
        });
    }

    /// A deprecated subcommand was named, as `name`.
    #[cold]
    #[inline(never)]
    pub fn deprecated_command(name: &[u8], deprecation: &'static crate::warn::Deprecation) {
        crate::warn::record(crate::warn::Warning {
            kind: crate::warn::WarningKind::Command,
            name: String::from_utf8_lossy(name).into_owned(),
            deprecation: Some(*deprecation),
            replacement: None,
        });
    }

    /// A value was read from `name`, a deprecated name of a variable whose
    /// current one is `instead`.
    #[cold]
    #[inline(never)]
    pub fn deprecated_env(name: &'static str, instead: Option<&'static str>) {
        crate::warn::record(crate::warn::Warning {
            kind: crate::warn::WarningKind::Env,
            name: name.to_owned(),
            deprecation: None,
            replacement: instead,
        });
    }

    /// An external subcommand's words: `name`, which has been read, then
    /// every word left, as typed, flags and `--` among them. `display` is how
    /// an error names a word that does not convert.
    pub fn external<T: FromArg>(
        name: &[u8],
        input: &mut Argv<'_>,
        display: &str,
    ) -> Result<Vec<T>, Error> {
        let mut values = Vec::with_capacity(input.words_left() + 1);
        let mut offset = input.offset().saturating_sub(name.len() + 1);
        let mut word = BStr::new(name);
        loop {
            let value = T::from_arg(word)
                .map_err(|cause| Error::invalid_value(offset, display, word, cause))?;
            values.push(value);
            if input.is_empty() {
                return Ok(values);
            }
            offset = input.offset();
            word = input.take_word();
        }
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
