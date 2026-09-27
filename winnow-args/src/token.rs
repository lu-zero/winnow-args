//! Lexing: one command-line item at a time.
//!
//! [`arg`] reads the next item — a long flag, one short letter, a word, or the
//! `--` separator — and nothing more. Whether a flag takes a value is not the
//! lexer's business: it leaves the stream where the value *would* start, and
//! the caller, which knows the flag, finishes it with [`Arg::switch`] or
//! [`Arg::value`]. One lexer thus serves an `alt` of flag parsers, a
//! `dispatch!` on flag names, and the derive's `match`.
//!
//! | command line | items                                        |
//! |--------------|----------------------------------------------|
//! | `--path=x`   | `Long` named `path` with value `x`           |
//! | `--path x`   | `Long` named `path`, then [`Arg::value`] reads `x` |
//! | `-vpx`       | `Short` `v`, `Short` `p`, then [`Arg::value`] reads `x` |
//! | `-p=x`       | `Short` `p`, then [`Arg::value`] reads `x`   |
//! | `-`          | `Word` `-`                                   |
//! | `--`         | `Separator`; every later word is a `Word`    |

use winnow::error::{ModalError as _, ParserError};
use winnow::prelude::*;
use winnow::stream::BStr;

use crate::error::Error;
use crate::stream::{Argv, Mode};
use crate::value::FromArg;

/// One lexed command-line item.
///
/// Its fields are public and plain so a `dispatch!` arm can match flags as
/// patterns, both spellings at once:
///
/// ```
/// use winnow::combinator::{dispatch, fail};
/// use winnow::prelude::*;
/// use winnow::stream::BStr;
/// use winnow_args::combinator::args;
/// use winnow_args::token::{Arg, LongFlag, ShortFlag, arg};
/// use winnow_args::{Argv, Error};
///
/// let words = [BStr::new("-p"), BStr::new("x"), BStr::new("f")];
/// let (mut path, mut files) = (None::<String>, Vec::<String>::new());
/// let (p, f) = (&mut path, &mut files);
/// args(dispatch! {arg;
///     a @ (Arg::Long(LongFlag { name: b"path", .. }) | Arg::Short(ShortFlag { letter: 'p', .. })) => {
///         a.value_as().map(|v| *p = Some(v))
///     },
///     Arg::Word(w) => w.value_as("FILE").map(|file| f.push(file)),
///     _ => fail,
/// })
/// .parse_next(&mut Argv::new(&words))?;
/// assert_eq!((path.as_deref(), &files[..]), (Some("x"), &["f".to_string()][..]));
/// # Ok::<(), Error>(())
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arg<'i> {
    /// `--name` or `--name=value`. The word is consumed whole.
    Long(LongFlag<'i>),
    /// One letter of `-abc`. The stream is left on the next letter (or the
    /// attached value), or at the next word if this was the last letter.
    Short(ShortFlag),
    /// A word that is not a flag: a positional value, a lone `-`, or anything
    /// after `--`.
    Word(Word<'i>),
    /// The first `--`.
    Separator {
        /// Where it is.
        offset: usize,
    },
}

/// `--name` or `--name=value`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LongFlag<'i> {
    /// The text between `--` and the first `=`, as bytes so it can be a pattern.
    pub name: &'i [u8],
    /// The text after the first `=`, if there was one.
    pub value: Option<&'i BStr>,
    /// Where the flag starts.
    pub offset: usize,
}

/// One letter of `-abc`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShortFlag {
    /// The letter.
    pub letter: char,
    /// Where the letter is.
    pub offset: usize,
}

/// A word that is not a flag.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Word<'i> {
    /// The word.
    pub value: &'i BStr,
    /// Where it starts.
    pub offset: usize,
    /// Whether it came after `--`, where no word names a subcommand.
    pub after_separator: bool,
}

impl<'i> Arg<'i> {
    /// The item as the user typed it, for error messages.
    pub fn spelling(&self) -> String {
        match self {
            Arg::Long(f) => format!("--{}", BStr::new(f.name)),
            Arg::Short(f) => format!("-{}", f.letter),
            Arg::Word(w) => w.value.to_string(),
            Arg::Separator { .. } => "--".to_owned(),
        }
    }

    /// Where the item starts.
    pub fn offset(&self) -> usize {
        match *self {
            Arg::Long(LongFlag { offset, .. })
            | Arg::Short(ShortFlag { offset, .. })
            | Arg::Word(Word { offset, .. })
            | Arg::Separator { offset } => offset,
        }
    }

    /// The error for this item when nothing accepts it.
    pub fn unexpected(&self) -> Error {
        match self {
            Arg::Long(_) | Arg::Short(_) => Error::unknown_flag(self.offset(), self.spelling()),
            Arg::Word(_) | Arg::Separator { .. } => {
                Error::unexpected_arg(self.offset(), self.spelling())
            }
        }
    }

    /// Finish a switch. Only a long flag can have been given a value,
    /// `--verbose=yes`, which is an error; in `-vx` the `x` is another letter.
    #[inline(always)]
    pub fn check_switch(&self) -> Result<(), Error> {
        match self {
            Arg::Long(LongFlag { value: Some(v), .. }) => {
                Err(Error::unexpected_value(self.offset(), self.spelling(), v))
            }
            _ => Ok(()),
        }
    }

    /// Finish an option, committing: a missing value is an error, not a reason
    /// to try something else.
    ///
    /// - `--name=value`: the attached text.
    /// - `-nvalue`, `-n=value`: the rest of the bundle, less one `=`.
    /// - `--name value`, `-n value`: the next word, unless it is flag-like. A
    ///   lone `-` is a value; `--` is not.
    #[inline(always)]
    pub fn read_value(&self, input: &mut Argv<'i>) -> Result<&'i BStr, Error> {
        self.read_value_with(input, ValueOptions::DEFAULT)
    }

    /// [`Arg::read_value`] under `options`: which detached words it may take.
    #[inline(always)]
    pub fn read_value_with(
        &self,
        input: &mut Argv<'i>,
        options: ValueOptions,
    ) -> Result<&'i BStr, Error> {
        match self.next_value(input, options) {
            Some(v) => Ok(v),
            None => Err(Error::missing_value(self.offset(), self.spelling())),
        }
    }

    /// [`Arg::read_value`] for a flag whose value may be left out: where that
    /// would report a missing value, `--color` alone or before another flag,
    /// this gives `missing` instead.
    #[inline(always)]
    pub fn read_value_or(&self, input: &mut Argv<'i>, missing: &'static BStr) -> &'i BStr {
        self.read_value_or_with(input, ValueOptions::DEFAULT, missing)
    }

    /// [`Arg::read_value_or`] under `options`.
    #[inline(always)]
    pub fn read_value_or_with(
        &self,
        input: &mut Argv<'i>,
        options: ValueOptions,
        missing: &'static BStr,
    ) -> &'i BStr {
        self.next_value(input, options).unwrap_or(missing)
    }

    #[inline(always)]
    fn next_value(&self, input: &mut Argv<'i>, options: ValueOptions) -> Option<&'i BStr> {
        match self {
            Arg::Long(LongFlag { value: Some(v), .. }) => Some(v),
            Arg::Short(_) if input.mode() == Mode::Bundle => {
                let v = input.take_word();
                Some(BStr::new(v.strip_prefix(b"=").unwrap_or(v)))
            }
            _ => {
                if input.is_empty() {
                    return None;
                }
                let next = input.front();
                if input.mode() == Mode::Word
                    && is_flag_like(next)
                    && !(options.negative_numbers && is_negative_number(next))
                {
                    return None;
                }
                Some(input.take_word())
            }
        }
    }

    /// [`Arg::read_value`], converted with [`FromArg`].
    #[inline]
    pub fn read_value_as<T: FromArg>(&self, input: &mut Argv<'i>) -> Result<T, Error> {
        let v = self.read_value(input)?;
        self.convert(v)
    }

    /// Convert `value`, one of this flag's values, naming the flag on failure.
    #[inline]
    pub fn convert<T: FromArg>(&self, value: &BStr) -> Result<T, Error> {
        T::from_arg(value)
            .map_err(|cause| Error::invalid_value(self.offset(), self.spelling(), value, cause))
    }

    /// [`Arg::read_value`] split on `delimiter`, each piece converted.
    pub fn values_as<T: FromArg>(&self, delimiter: u8) -> impl Parser<Argv<'i>, Vec<T>, Error> {
        move |input: &mut Argv<'i>| {
            let value = self.read_value(input)?;
            split(value, delimiter).map(|v| self.convert(v)).collect()
        }
    }

    /// [`Arg::check_switch`] as a parser, for a `dispatch!` arm.
    pub fn switch(&self) -> impl Parser<Argv<'i>, (), Error> {
        move |_: &mut Argv<'i>| self.check_switch()
    }

    /// [`Arg::read_value`] as a parser, for a `dispatch!` arm.
    pub fn value(&self) -> impl Parser<Argv<'i>, &'i BStr, Error> {
        move |input: &mut Argv<'i>| self.read_value(input)
    }

    /// [`Arg::read_value_or`], converted, as a parser for a `dispatch!` arm.
    pub fn value_or<T: FromArg>(&self, missing: &'static BStr) -> impl Parser<Argv<'i>, T, Error> {
        move |input: &mut Argv<'i>| {
            let v = self.read_value_or(input, missing);
            self.convert(v)
        }
    }

    /// [`Arg::read_value_with`], converted, as a parser for a `dispatch!` arm.
    pub fn value_as_with<T: FromArg>(
        &self,
        options: ValueOptions,
    ) -> impl Parser<Argv<'i>, T, Error> {
        move |input: &mut Argv<'i>| {
            let v = self.read_value_with(input, options)?;
            self.convert(v)
        }
    }

    /// [`Arg::read_value_as`] as a parser, for a `dispatch!` arm.
    pub fn value_as<T: FromArg>(&self) -> impl Parser<Argv<'i>, T, Error> {
        move |input: &mut Argv<'i>| self.read_value_as(input)
    }
}

/// The pieces of `value` between `delimiter`s; empty pieces are kept.
#[inline]
pub fn split(value: &BStr, delimiter: u8) -> impl Iterator<Item = &BStr> {
    value.split(move |&b| b == delimiter).map(BStr::new)
}

impl<'i> Word<'i> {
    /// The pieces of the word between `delimiter`s, each at its own offset.
    #[inline]
    pub fn split(&self, delimiter: u8) -> impl Iterator<Item = Word<'i>> {
        let (base, after_separator) = (self.offset, self.after_separator);
        let start = self.value.as_ptr() as usize;
        split(self.value, delimiter).map(move |value| Word {
            value,
            offset: base + (value.as_ptr() as usize - start),
            after_separator,
        })
    }

    /// Convert the word with [`FromArg`]; `name` is how errors refer to it.
    #[inline]
    pub fn convert<T: FromArg>(&self, name: &str) -> Result<T, Error> {
        T::from_arg(self.value)
            .map_err(|cause| Error::invalid_value(self.offset, name, self.value, cause))
    }

    /// [`Word::convert`] as a parser, for a `dispatch!` arm.
    pub fn value_as<T: FromArg>(&self, name: &'static str) -> impl Parser<Argv<'i>, T, Error> {
        move |_: &mut Argv<'i>| self.convert(name)
    }
}

/// How a flag takes a detached value, beyond the default of any word that is
/// not flag-like.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ValueOptions {
    /// A negative number is a value, not a flag: `--offset -1`.
    pub negative_numbers: bool,
}

impl ValueOptions {
    /// Any word that is not flag-like.
    pub const DEFAULT: Self = Self {
        negative_numbers: false,
    };
}

/// Whether `word` is a negative number: `-` then digits, at most one `.`, and an
/// optional exponent (`e` or `E`, an optional sign, at least one digit). Narrower
/// than a float parse on purpose: `-inf` and `-NaN` are likelier misspelled flags.
#[inline]
pub fn is_negative_number(word: &[u8]) -> bool {
    let Some(rest) = word.strip_prefix(b"-") else {
        return false;
    };
    let (mantissa, exponent) = match rest.iter().position(|b| matches!(b, b'e' | b'E')) {
        Some(at) => (&rest[..at], Some(&rest[at + 1..])),
        None => (rest, None),
    };
    let (mut digit, mut dot) = (false, false);
    for &b in mantissa {
        match b {
            b'0'..=b'9' => digit = true,
            b'.' if !dot => dot = true,
            _ => return false,
        }
    }
    let exponent_ok = match exponent {
        None => true,
        Some(e) => {
            let digits = e
                .strip_prefix(b"+")
                .or_else(|| e.strip_prefix(b"-"))
                .unwrap_or(e);
            !digits.is_empty() && digits.iter().all(u8::is_ascii_digit)
        }
    };
    digit && exponent_ok
}

/// A negative number standing where a word starts, as a positional value: for a
/// positional declared `allow_negative_numbers`, which would otherwise see a flag.
pub fn number<'i>(input: &mut Argv<'i>) -> Result<Word<'i>, Error> {
    if input.mode() != Mode::Word || !is_negative_number(input.front()) {
        return Err(Error::from_input(input));
    }
    let offset = input.offset();
    Ok(Word {
        after_separator: false,
        value: input.take_word(),
        offset,
    })
}

/// Whether a detached word would be read as a flag rather than a value.
#[inline(always)]
fn is_flag_like(word: &[u8]) -> bool {
    word.len() > 1 && word[0] == b'-'
}

/// Read one short letter, and note whether the bundle goes on.
#[inline(always)]
fn letter(input: &mut Argv<'_>) -> char {
    let bytes = input.front();
    let (c, width) = match bytes[0] {
        b if b.is_ascii() => (b as char, 1),
        b => {
            let width = match b {
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => 1,
            };
            match bytes.get(..width).and_then(|b| std::str::from_utf8(b).ok()) {
                Some(s) => (
                    s.chars().next().unwrap_or(char::REPLACEMENT_CHARACTER),
                    width,
                ),
                None => (char::REPLACEMENT_CHARACTER, 1),
            }
        }
    };
    input.take_bytes(width);
    if input.front().is_empty() {
        input.take_word();
        input.set_mode(Mode::Word);
    } else {
        input.set_mode(Mode::Bundle);
    }
    c
}

/// What the next item is, without consuming it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `--name` or `--name=value`.
    Long,
    /// A letter of `-abc`, at its start or part-way through.
    Short,
    /// A word that is not a flag.
    Word,
    /// The first `--`.
    Separator,
}

/// Classify the next item without consuming it; backtracks at the end of the
/// command line. Meant as a `dispatch!` scrutinee, so a word is only offered to
/// positional parsers and a flag only to flag parsers.
///
/// `dispatch!` is a `move` closure: give it `&mut` references, or a `bool` it
/// sets would be a copy.
///
/// ```
/// use winnow::combinator::{alt, dispatch, fail};
/// use winnow::prelude::*;
/// use winnow::stream::BStr;
/// use winnow_args::combinator::{args, positional, short};
/// use winnow_args::token::{Kind, kind};
/// use winnow_args::{Argv, Error};
///
/// let words = [BStr::new("a"), BStr::new("-v"), BStr::new("b")];
/// let (mut verbose, mut files) = (false, Vec::<String>::new());
/// let (v, f) = (&mut verbose, &mut files);
/// args(dispatch! {kind;
///     Kind::Long | Kind::Short => alt((short('v').long("verbose").switch().map(|()| *v = true),)),
///     Kind::Word => positional("FILE").map(|file| f.push(file)),
///     Kind::Separator => fail,
/// })
/// .parse_next(&mut Argv::new(&words))?;
/// assert!(verbose);
/// assert_eq!(files, ["a", "b"]);
/// # Ok::<(), Error>(())
/// ```
#[inline]
pub fn kind(input: &mut Argv<'_>) -> Result<Kind, Error> {
    if input.is_empty() {
        return Err(Error::from_input(input));
    }
    Ok(match input.mode() {
        Mode::Bundle => Kind::Short,
        Mode::Stopped | Mode::Values => Kind::Word,
        Mode::Word => match input.front() {
            b"--" => Kind::Separator,
            [b'-', b'-', ..] => Kind::Long,
            [b'-', _, ..] => Kind::Short,
            _ => Kind::Word,
        },
    })
}

/// Lex the next item. Backtracks at the end of the command line.
// `always`: returned out of line, `Result<Arg, Error>` goes through memory and
// stalls on store-to-load forwarding (docs/PERF.md, step 5).
#[inline(always)]
pub fn arg<'i>(input: &mut Argv<'i>) -> Result<Arg<'i>, Error> {
    let kind = kind(input)?;
    let offset = input.offset();
    Ok(match kind {
        Kind::Separator => {
            input.take_word();
            input.set_mode(Mode::Stopped);
            Arg::Separator { offset }
        }
        Kind::Long => {
            let word = &input.take_word()[2..];
            Arg::Long(match word.iter().position(|&b| b == b'=') {
                Some(eq) => LongFlag {
                    name: &word[..eq],
                    value: Some(BStr::new(&word[eq + 1..])),
                    offset,
                },
                None => LongFlag {
                    name: word,
                    value: None,
                    offset,
                },
            })
        }
        Kind::Short => {
            // A bundle's first letter is reported at its `-`, later ones at themselves.
            if input.mode() == Mode::Word {
                input.take_bytes(1);
            }
            Arg::Short(ShortFlag {
                letter: letter(input),
                offset,
            })
        }
        Kind::Word => Arg::Word(Word {
            after_separator: input.mode() == Mode::Stopped,
            value: input.take_word(),
            offset,
        }),
    })
}

/// A word that is not a flag: a positional value.
pub fn word<'i>(input: &mut Argv<'i>) -> Result<Word<'i>, Error> {
    if kind(input)? != Kind::Word {
        return Err(Error::from_input(input));
    }
    let offset = input.offset();
    Ok(Word {
        after_separator: input.mode() == Mode::Stopped,
        value: input.take_word(),
        offset,
    })
}

/// Succeed at the end of the command line; otherwise report what is left over.
pub fn finish(input: &mut Argv<'_>) -> Result<(), Error> {
    if input.is_empty() {
        return Ok(());
    }
    Err(arg(input)?.unexpected().cut())
}
