//! Storing values into a derived struct's fields, out of line.
//!
//! Converting an owned value (a `String`, a `PathBuf`) and dropping the one it
//! replaces is a few hundred bytes of code. Inlined into every field's `match`
//! arm it was most of a derived parser's size (docs/PERF.md, step 31); called
//! here, it is one copy per value type, for a call per value on the hot path.

use winnow::stream::BStr;

use crate::error::Error;
use crate::token::{Arg, Word, split};
use crate::value::FromArg;

/// A flag's value into an `Option<T>` field, replacing any earlier one.
#[inline(never)]
pub fn set<T: FromArg>(slot: &mut Option<T>, arg: &Arg<'_>, value: &BStr) -> Result<(), Error> {
    *slot = Some(arg.convert(value)?);
    Ok(())
}

/// A flag's value (split on `delimiter`, if any) onto a `Vec<T>` field.
#[inline(never)]
pub fn push<T: FromArg>(
    slot: &mut Vec<T>,
    arg: &Arg<'_>,
    value: &BStr,
    delimiter: Option<u8>,
) -> Result<(), Error> {
    match delimiter {
        None => slot.push(arg.convert(value)?),
        Some(d) => {
            for piece in split(value, d) {
                slot.push(arg.convert(piece)?);
            }
        }
    }
    Ok(())
}

/// A positional word into an `Option<T>` field; `name` names it in errors.
#[inline(never)]
pub fn set_word<T: FromArg>(
    slot: &mut Option<T>,
    word: &Word<'_>,
    name: &str,
) -> Result<(), Error> {
    *slot = Some(word.convert(name)?);
    Ok(())
}

/// A positional word (split on `delimiter`, if any) onto a `Vec<T>` field.
#[inline(never)]
pub fn push_word<T: FromArg>(
    slot: &mut Vec<T>,
    word: &Word<'_>,
    name: &str,
    delimiter: Option<u8>,
) -> Result<(), Error> {
    match delimiter {
        None => slot.push(word.convert(name)?),
        Some(d) => {
            for piece in word.split(d) {
                slot.push(piece.convert(name)?);
            }
        }
    }
    Ok(())
}

/// A default's or an environment variable's value into an `Option<T>` field;
/// `source` names it in errors, reported at `at`.
#[inline(never)]
pub fn set_from<T: FromArg>(
    slot: &mut Option<T>,
    value: &BStr,
    source: &str,
    at: usize,
) -> Result<(), Error> {
    *slot = Some(convert(value, source, at)?);
    Ok(())
}

/// [`set_from`] for a `Vec<T>` field, split on `delimiter` if any.
#[inline(never)]
pub fn push_from<T: FromArg>(
    slot: &mut Vec<T>,
    value: &BStr,
    source: &str,
    at: usize,
    delimiter: Option<u8>,
) -> Result<(), Error> {
    match delimiter {
        None => slot.push(convert(value, source, at)?),
        Some(d) => {
            for piece in split(value, d) {
                slot.push(convert(piece, source, at)?);
            }
        }
    }
    Ok(())
}

fn convert<T: FromArg>(value: &BStr, source: &str, at: usize) -> Result<T, Error> {
    T::from_arg(value).map_err(|cause| Error::invalid_value(at, source, value, cause))
}
