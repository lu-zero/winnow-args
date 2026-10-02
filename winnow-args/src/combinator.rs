//! Flag parsers, bpaf-style: name an item, then say what it is.
//!
//! ```
//! use winnow::prelude::*;
//! use winnow::combinator::alt;
//! use winnow_args::BStr;
//! use winnow_args::{Argv, Error, combinator::{args, short}};
//!
//! let verbose = short('v').long("verbose");
//! let path = short('p').long("path");
//!
//! let words = [BStr::new("-v"), BStr::new("--path=/tmp")];
//! let mut input = Argv::new(&words);
//!
//! let (mut v, mut p) = (false, None);
//! args(alt((
//!     verbose.switch().map(|()| v = true),
//!     path.argument().map(|x| p = Some(x)),
//! )))
//! .parse_next(&mut input)?;
//!
//! assert!(v);
//! assert_eq!(p.map(|x| &**x), Some(&b"/tmp"[..]));
//! # Ok::<(), Error>(())
//! ```
//!
//! Each parser here matches **one occurrence**. Absence, repetition and
//! requiredness are decided by whatever folds the occurrences together — above,
//! two closures and [`args`]; in the derive, a `match` in a loop — because only
//! that layer knows the field's type.

use winnow::combinator::{alt, repeat, trace};
use winnow::error::{ModalError as _, ParserError as _};
use winnow::prelude::*;

use crate::error::Error;
use crate::stream::Argv;
use crate::token::{self, Arg, ValueOptions};
use crate::value::FromArg;
use winnow::stream::BStr;

/// The names a flag answers to: one short letter, and `N` long names (the first
/// is the flag's name, the rest are aliases).
///
/// `N` is part of the type so the common `Named` stays 24 bytes: the parsers
/// built from it copy it on every use, and larger copies stall
/// (docs/PERF.md, step 8). Only a flag with aliases pays for them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Named<const N: usize = 1> {
    short: Option<char>,
    longs: [Option<&'static str>; N],
    /// Sits in padding: a plain `Named` stays 24 bytes.
    options: ValueOptions,
}

/// A flag named `-c`.
pub const fn short(c: char) -> Named {
    Named {
        short: Some(c),
        longs: [None],
        options: ValueOptions::DEFAULT,
    }
}

/// A flag named `--name`.
pub const fn long(name: &'static str) -> Named {
    Named {
        short: None,
        longs: [Some(name)],
        options: ValueOptions::DEFAULT,
    }
}

impl<const N: usize> Named<N> {
    /// Also answer to `-c`.
    pub const fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Answer to `--name`, replacing any long names.
    pub const fn long(self, name: &'static str) -> Named {
        Named {
            short: self.short,
            longs: [Some(name)],
            options: self.options,
        }
    }

    /// Answer to all of these long names, replacing any: the first is the
    /// flag's name, the rest are aliases.
    pub const fn longs<const M: usize>(self, names: [&'static str; M]) -> Named<M> {
        let mut longs = [None; M];
        let mut i = 0;
        while i < M {
            longs[i] = Some(names[i]);
            i += 1;
        }
        Named {
            short: self.short,
            longs,
            options: self.options,
        }
    }

    /// Take the next word as the value whatever it looks like: `--args -x`, `--args --`.
    pub const fn allow_hyphen_values(mut self) -> Self {
        self.options.hyphen_values = true;
        self
    }

    /// `true` for this flag, `false` for `no`: usage's `negate`, as in
    /// `long("color").negated_by(long("no-color"))`. Fold with "last wins".
    pub fn negated_by<'i, const M: usize>(
        self,
        no: Named<M>,
    ) -> impl Parser<Argv<'i>, bool, Error> {
        trace(
            "negated_by",
            alt((self.switch().value(true), no.switch().value(false))),
        )
    }

    /// Only an attached value: `--name=v`, `-cv`; the next word is never taken.
    pub const fn require_equals(mut self) -> Self {
        self.options.require_equals = true;
        self
    }

    /// Take a negative number as a detached value: `--offset -1`.
    pub const fn allow_negative_numbers(mut self) -> Self {
        self.options.negative_numbers = true;
        self
    }

    /// Whether `arg` is this flag.
    #[inline]
    pub fn matches(&self, arg: &Arg<'_>) -> bool {
        match arg {
            Arg::Long(f) => self
                .longs
                .iter()
                .any(|l| l.is_some_and(|l| f.name == l.as_bytes())),
            Arg::Short(f) => self.short == Some(f.letter),
            Arg::Word(_) | Arg::Separator { .. } => false,
        }
    }

    /// How to name the flag in a message: the long name if it has one.
    pub fn display(&self) -> String {
        match (self.longs.first().copied().flatten(), self.short) {
            (Some(l), _) => format!("--{l}"),
            (None, Some(c)) => format!("-{c}"),
            (None, None) => String::new(),
        }
    }

    /// The error for a required flag that never appeared.
    pub fn missing(&self, input: &Argv<'_>) -> Error {
        Error::missing_required(input.offset(), self.display())
    }

    /// A switch: `--name` or `-c`, no value.
    pub fn switch<'i>(mut self) -> impl Parser<Argv<'i>, (), Error> {
        trace("switch", move |input: &mut Argv<'i>| {
            self.parse_next(input)?.switch().parse_next(input)
        })
    }

    /// An option taking one value, as bytes.
    pub fn argument<'i>(mut self) -> impl Parser<Argv<'i>, &'i BStr, Error> {
        trace("argument", move |input: &mut Argv<'i>| {
            let arg = self.parse_next(input)?;
            arg.read_value_with(input, self.options)
        })
    }

    /// An option taking one value, converted with [`FromArg`].
    pub fn argument_as<'i, T: FromArg>(mut self) -> impl Parser<Argv<'i>, T, Error> {
        trace("argument_as", move |input: &mut Argv<'i>| {
            self.parse_next(input)?
                .value_as_with(self.options)
                .parse_next(input)
        })
    }

    /// An option whose value may be left out: a bare `--name`, or one followed by
    /// another flag, gives `missing`. `--name=v` and `--name v` give `v`.
    pub fn argument_or<'i, T: FromArg>(
        mut self,
        missing: &'static str,
    ) -> impl Parser<Argv<'i>, T, Error> {
        trace("argument_or", move |input: &mut Argv<'i>| {
            let arg = self.parse_next(input)?;
            let value = arg.read_value_or_with(input, self.options, BStr::new(missing));
            arg.convert(value)
        })
    }

    /// An option whose value is split on `delimiter`, each piece converted:
    /// `--tags a,b` gives two.
    pub fn arguments_as<'i, T: FromArg>(
        mut self,
        delimiter: u8,
    ) -> impl Parser<Argv<'i>, Vec<T>, Error> {
        trace("arguments_as", move |input: &mut Argv<'i>| {
            let arg = self.parse_next(input)?;
            let value = arg.read_value_with(input, self.options)?;
            token::split(value, delimiter)
                .map(|v| arg.convert(v))
                .collect()
        })
    }
}

/// One occurrence of the flag's name, value not yet read.
impl<'i, const N: usize> Parser<Argv<'i>, Arg<'i>, Error> for Named<N> {
    #[inline]
    fn parse_next(&mut self, input: &mut Argv<'i>) -> Result<Arg<'i>, Error> {
        token::arg
            .verify(|arg: &Arg<'i>| self.matches(arg))
            .parse_next(input)
    }
}

/// A positional value: one word that is not a flag, converted with [`FromArg`].
/// `name` is how errors refer to it.
///
/// This matches one word; which positional a word fills is up to the caller,
/// as with flags.
pub fn positional<'i, T: FromArg>(name: &'static str) -> impl Parser<Argv<'i>, T, Error> {
    trace("positional", move |input: &mut Argv<'i>| {
        token::word(input)?.value_as(name).parse_next(input)
    })
}

/// The spellings of a subcommand: one name, or a name and its aliases.
pub trait Names {
    /// Whether `word` is one of them.
    fn contains(&self, word: &[u8]) -> bool;
}

impl Names for &str {
    #[inline]
    fn contains(&self, word: &[u8]) -> bool {
        self.as_bytes() == word
    }
}

impl<const N: usize> Names for [&str; N] {
    #[inline]
    fn contains(&self, word: &[u8]) -> bool {
        self.iter().any(|name| name.as_bytes() == word)
    }
}

/// A subcommand: the word `name` (or any of `["name", "alias", …]`), then
/// `inner` for the rest of the line.
///
/// Once the word matches, `inner`'s errors are committed. After `--` no word is
/// a subcommand.
pub fn command<'i, O, P>(name: impl Names, mut inner: P) -> impl Parser<Argv<'i>, O, Error>
where
    P: Parser<Argv<'i>, O, Error>,
{
    trace("command", move |input: &mut Argv<'i>| {
        let start = input.checkpoint();
        match token::word(input) {
            Ok(word) if name.contains(word.value) && !word.after_separator => {
                inner.parse_next(input).map_err(|e| e.cut())
            }
            Ok(_) => {
                input.reset(&start);
                Err(Error::from_input(input))
            }
            Err(e) => Err(e),
        }
    })
}

/// Apply `item` until it stops matching, then require the command line to be
/// over. What is left is reported as an unknown flag or an unexpected argument.
///
/// `item` is usually an `alt` of occurrence parsers each `map`ped into a
/// variable. The `--` separator is consumed here, so a positional parser in
/// `item` sees the words after it as plain words.
pub fn args<'i, P>(mut item: P) -> impl Parser<Argv<'i>, (), Error>
where
    P: Parser<Argv<'i>, (), Error>,
{
    trace("args", move |input: &mut Argv<'i>| {
        let separator = token::arg
            .verify(|a: &Arg<'i>| matches!(a, Arg::Separator { .. }))
            .void();
        repeat::<_, _, (), _, _>(0.., winnow::combinator::alt((item.by_ref(), separator)))
            .parse_next(input)?;
        token::finish(input).map_err(|e| e.cut())
    })
}
