//! Lexing: one command-line item at a time.
//!
//! [`arg`] reads the next item — a long flag, one short letter, a word, or the
//! `--` separator — and nothing more. Whether a flag takes a value is not the
//! lexer's business: it leaves the stream where the value *would* start, and
//! the caller, which knows the flag, either takes it with [`value`] or checks
//! there is none with [`no_value`]. That split is what lets one lexer serve
//! both a hand-written `alt` of flag parsers and a derive's `match`.
//!
//! | command line | items                                        |
//! |--------------|----------------------------------------------|
//! | `--path=x`   | `Long { name: "path", value: Some("x") }`    |
//! | `--path x`   | `Long { name: "path", value: None }`, then [`value`] reads `x` |
//! | `-vpx`       | `Short('v')`, `Short('p')`, then [`value`] reads `x` |
//! | `-p=x`       | `Short('p')`, then [`value`] reads `x`       |
//! | `-`          | `Word("-")`                                  |
//! | `--`         | `Separator`; every later word is a `Word`    |

use winnow::error::{ModalError as _, ParserError};
use winnow::stream::{BStr, Stream as _};

use crate::error::Error;
use crate::stream::{Argv, Mode};

/// One lexed command-line item.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arg<'i> {
    /// `--name` or `--name=value`. The word is consumed whole.
    Long {
        /// The text between `--` and the first `=`.
        name: &'i BStr,
        /// The text after the first `=`, if there was one.
        value: Option<&'i BStr>,
    },
    /// One letter of `-abc`. The stream is left on the next letter (or the
    /// attached value), or at the next word if this was the last letter.
    Short(char),
    /// A word that is not a flag: a positional value, a lone `-`, or anything
    /// after `--`.
    Word(&'i BStr),
    /// The first `--`.
    Separator,
}

impl Arg<'_> {
    /// The item as the user typed it, for error messages.
    pub fn spelling(&self) -> String {
        match self {
            Arg::Long { name, .. } => format!("--{name}"),
            Arg::Short(c) => format!("-{c}"),
            Arg::Word(word) => word.to_string(),
            Arg::Separator => "--".to_owned(),
        }
    }
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

/// Lex the next item. Backtracks at the end of the command line.
pub fn arg<'i>(input: &mut Argv<'i>) -> Result<Arg<'i>, Error> {
    if input.is_empty() {
        return Err(Error::from_input(input));
    }
    match input.mode() {
        Mode::Bundle => Ok(Arg::Short(letter(input))),
        Mode::Stopped => Ok(Arg::Word(input.take_word())),
        Mode::Word => match input.front() {
            b"--" => {
                input.take_word();
                input.set_mode(Mode::Stopped);
                Ok(Arg::Separator)
            }
            [b'-', b'-', ..] => {
                let word = &input.take_word()[2..];
                Ok(match word.iter().position(|&b| b == b'=') {
                    Some(eq) => Arg::Long {
                        name: BStr::new(&word[..eq]),
                        value: Some(BStr::new(&word[eq + 1..])),
                    },
                    None => Arg::Long {
                        name: BStr::new(word),
                        value: None,
                    },
                })
            }
            [b'-', _, ..] => {
                input.take_bytes(1);
                Ok(Arg::Short(letter(input)))
            }
            _ => Ok(Arg::Word(input.take_word())),
        },
    }
}

/// A word that is not a flag: a positional value.
pub fn word<'i>(input: &mut Argv<'i>) -> Result<&'i BStr, Error> {
    let start = input.checkpoint();
    match arg(input)? {
        Arg::Word(word) => Ok(word),
        _ => {
            input.reset(&start);
            Err(Error::from_input(input))
        }
    }
}

/// The value of the flag `arg` just lexed at `offset`. Commits: a flag that
/// needs a value and has none is an error, not a reason to try something else.
///
/// - `--name=value`: the attached text.
/// - `-nvalue`, `-n=value`: the rest of the bundle, less one `=`.
/// - `--name value`, `-n value`: the next word, unless it is flag-like. A lone
///   `-` is a value; `--` is not.
pub fn value<'i>(input: &mut Argv<'i>, arg: &Arg<'i>, offset: usize) -> Result<&'i BStr, Error> {
    match *arg {
        Arg::Long { value: Some(v), .. } => return Ok(v),
        Arg::Short(_) if input.mode() == Mode::Bundle => {
            let v = input.take_word();
            return Ok(BStr::new(v.strip_prefix(b"=").unwrap_or(v)));
        }
        _ => {}
    }
    if input.is_empty() || (input.mode() != Mode::Stopped && is_flag_like(input.front())) {
        return Err(Error::missing_value(offset, arg.spelling()));
    }
    Ok(input.take_word())
}

/// Check that the flag `arg` just lexed at `offset` was not given a value.
/// Only a long flag can be: `--verbose=yes`. In `-vx` the `x` is another letter.
pub fn no_value(arg: &Arg<'_>, offset: usize) -> Result<(), Error> {
    match arg {
        Arg::Long { value: Some(v), .. } => Err(Error::unexpected_value(offset, arg.spelling(), v)),
        _ => Ok(()),
    }
}

/// The error for an item, lexed at `offset`, that nothing accepted.
pub fn unexpected(arg: &Arg<'_>, offset: usize) -> Error {
    match arg {
        Arg::Long { .. } | Arg::Short(_) => Error::unknown_flag(offset, arg.spelling()),
        Arg::Word(_) | Arg::Separator => Error::unexpected_arg(offset, arg.spelling()),
    }
}

/// Succeed at the end of the command line; otherwise report what is left over.
pub fn finish(input: &mut Argv<'_>) -> Result<(), Error> {
    if input.is_empty() {
        return Ok(());
    }
    let offset = input.offset();
    let arg = arg(input)?;
    Err(unexpected(&arg, offset).cut())
}
