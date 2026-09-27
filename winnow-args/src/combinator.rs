//! Flag parsers, bpaf-style: name an item, then say what it is.
//!
//! ```
//! use winnow::prelude::*;
//! use winnow::combinator::alt;
//! use winnow::stream::BStr;
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

use winnow::combinator::{repeat, trace};
use winnow::error::{ModalError as _, ParserError as _};
use winnow::prelude::*;

use crate::error::Error;
use crate::stream::Argv;
use crate::token::{self, Arg};
use crate::value::FromArg;
use winnow::stream::BStr;

/// The names a flag answers to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Named {
    short: Option<char>,
    long: Option<&'static str>,
}

/// A flag named `-c`.
pub const fn short(c: char) -> Named {
    Named {
        short: Some(c),
        long: None,
    }
}

/// A flag named `--name`.
pub const fn long(name: &'static str) -> Named {
    Named {
        short: None,
        long: Some(name),
    }
}

impl Named {
    /// Also answer to `-c`.
    pub const fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Also answer to `--name`.
    pub const fn long(mut self, name: &'static str) -> Self {
        self.long = Some(name);
        self
    }

    /// Whether `arg` is this flag.
    #[inline]
    pub fn matches(&self, arg: &Arg<'_>) -> bool {
        match arg {
            Arg::Long(f) => self.long.is_some_and(|l| f.name == l.as_bytes()),
            Arg::Short(f) => self.short == Some(f.letter),
            Arg::Word(_) | Arg::Separator { .. } => false,
        }
    }

    /// How to name the flag in a message: the long name if it has one.
    pub fn display(&self) -> String {
        match (self.long, self.short) {
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
            self.parse_next(input)?.value().parse_next(input)
        })
    }

    /// An option taking one value, converted with [`FromArg`].
    pub fn argument_as<'i, T: FromArg>(mut self) -> impl Parser<Argv<'i>, T, Error> {
        trace("argument_as", move |input: &mut Argv<'i>| {
            self.parse_next(input)?.value_as().parse_next(input)
        })
    }
}

/// One occurrence of the flag's name, value not yet read.
impl<'i> Parser<Argv<'i>, Arg<'i>, Error> for Named {
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

/// A subcommand: the word `name`, then `inner` for the rest of the line.
///
/// Once the word matches, `inner`'s errors are committed. After `--` no word is
/// a subcommand.
pub fn command<'i, O, P>(name: &'static str, mut inner: P) -> impl Parser<Argv<'i>, O, Error>
where
    P: Parser<Argv<'i>, O, Error>,
{
    trace("command", move |input: &mut Argv<'i>| {
        let start = input.checkpoint();
        match token::word(input) {
            Ok(word) if *word.value == *name.as_bytes() && !word.after_separator => {
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
