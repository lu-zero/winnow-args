//! Turning a value's bytes into a Rust type.
//!
//! A field's type says how its value is read, through [`FromArg`]:
//!
//! | Type | Reads |
//! |---|---|
//! | `String`, `char`, `bool`, integers, floats, `std::net` addresses | UTF-8 text, parsed with `str::parse` |
//! | `PathBuf`, `OsString` | the bytes as they are on Unix, UTF-8 elsewhere |
//! | `Vec<u8>` | the bytes as they are |
//! | an enum deriving `ValueEnum` | one of its variants' names |
//! | [`Parsed<T>`] | any `T: FromStr`, a type from another crate included |
//! | [`CInt<T>`] | an integer in C syntax: `0x400000`, `010000` |
//! | [`KeyValue<K, V>`] | `key=value`, split at the first `=` |
//!
//! Any other type implements [`FromArg`] itself. In an `Occurrence` variant,
//! [`Spanned<T>`] is a `T` with where it was found.

use std::ffi::OsString;
use std::path::PathBuf;

use winnow::stream::BStr;

use crate::error::BoxError;

/// A type a flag's value can be converted into.
///
/// Values arrive as bytes borrowed from the command line. Conversion is the
/// only place a value can fail, so a command line that is not valid UTF-8 still
/// parses — flags still match — and only a value that is actually converted to
/// text can be rejected for it.
///
/// For a type that is [`FromStr`](std::str::FromStr), [`Parsed<T>`] needs no
/// impl. Otherwise:
///
/// ```
/// use winnow_args::error::BoxError;
/// use winnow_args::{BStr, FromArg};
///
/// /// `WIDTHxHEIGHT`.
/// struct Size(u32, u32);
///
/// impl FromArg for Size {
///     fn from_arg(value: &BStr) -> Result<Self, BoxError> {
///         let (w, h) = std::str::from_utf8(value)?
///             .split_once('x')
///             .ok_or("expected WIDTHxHEIGHT")?;
///         Ok(Size(w.parse()?, h.parse()?))
///     }
/// }
///
/// let Size(w, h) = Size::from_arg(BStr::new("80x24")).unwrap();
/// assert_eq!((w, h), (80, 24));
/// ```
pub trait FromArg: Sized {
    /// The values accepted, when there is a fixed set: help lists them as
    /// `[possible values: …]`. Empty by default; `#[derive(ValueEnum)]` fills
    /// it with each visible variant's name.
    const CHOICES: &'static [&'static str] = &[];

    /// Convert one value.
    fn from_arg(value: &BStr) -> Result<Self, BoxError>;
}

/// The cause of an [`ErrorKind::InvalidChoice`](crate::ErrorKind::InvalidChoice):
/// the value is not one of `choices`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceError {
    /// Every accepted spelling.
    pub choices: &'static [&'static str],
}

impl std::fmt::Display for ChoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected one of {}", self.choices.join(", "))
    }
}

impl std::error::Error for ChoiceError {}

/// The value as `&str`.
pub fn to_str(value: &BStr) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(value)
}

impl FromArg for String {
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        Ok(to_str(value)?.to_owned())
    }
}

impl FromArg for Vec<u8> {
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        Ok(value.to_vec())
    }
}

impl FromArg for OsString {
    #[cfg(unix)]
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        use std::os::unix::ffi::OsStrExt as _;
        Ok(std::ffi::OsStr::from_bytes(value).to_owned())
    }

    #[cfg(not(unix))]
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        Ok(to_str(value)?.into())
    }
}

impl FromArg for PathBuf {
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        OsString::from_arg(value).map(PathBuf::from)
    }
}

macro_rules! from_str {
    ($($ty:ty),* $(,)?) => {$(
        impl FromArg for $ty {
            fn from_arg(value: &BStr) -> Result<Self, BoxError> {
                Ok(to_str(value)?.parse::<$ty>()?)
            }
        }
    )*};
}

from_str!(
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
);

/// Any [`FromStr`](std::str::FromStr) type as a value: the text is parsed with
/// `str::parse`, and its error becomes the invalid value's cause.
///
/// For a type from another crate, which can implement neither [`FromArg`]
/// here nor derive `ValueEnum` (a `strum` enum, say).
///
/// ```
/// use winnow_args::FromArg;
/// use winnow_args::value::Parsed;
///
/// let Parsed(ip) = Parsed::<std::net::Ipv4Addr>::from_arg("127.0.0.1".into())?;
/// assert!(ip.is_loopback());
/// assert!(Parsed::<std::net::Ipv4Addr>::from_arg("localhost".into()).is_err());
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Parsed<T>(pub T);

impl<T> FromArg for Parsed<T>
where
    T: std::str::FromStr,
    T::Err: Into<BoxError>,
{
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        to_str(value)?.parse().map(Parsed).map_err(Into::into)
    }
}

/// A value with where it was found: an `Occurrence` variant holding a
/// `Spanned<T>` gets the flag's (or word's) offset and whether the value was
/// attached (`-zfoo`, `--name=value`) or the next word (`-z foo`).
///
/// For errors that point at an item of a sequence (`--pop-state` with nothing
/// pushed), and for messages that echo the spelling given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The value.
    pub value: T,
    /// Byte offset of the flag, or of the word for a positional, as
    /// [`Argv::offset`](crate::Argv::offset) counts.
    pub offset: usize,
    /// Whether the value was in the flag's own word.
    pub attached: bool,
}

/// An integer in C syntax, as linkers and assemblers read one: `0x`/`0X` hex,
/// a leading `0` for octal, otherwise decimal, with a sign for signed types.
/// `--image-base=0x400000`, `-z max-page-size=0x1000`, `-Ttext=010000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CInt<T>(pub T);

/// The error of a malformed [`CInt`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CIntError(&'static str);

impl std::fmt::Display for CIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for CIntError {}

/// The sign, radix and digits of a C integer.
fn c_digits(text: &str) -> Result<(bool, u32, &str), CIntError> {
    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (radix, digits) =
        if let Some(hex) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            (16, hex)
        } else if rest.len() > 1 && rest.starts_with('0') {
            (8, &rest[1..])
        } else {
            (10, rest)
        };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return Err(CIntError("expected a number: decimal, 0x hex or 0 octal"));
    }
    Ok((negative, radix, digits))
}

macro_rules! c_int {
    ($($ty:ty),* $(,)?) => {$(
        impl FromArg for CInt<$ty> {
            fn from_arg(value: &BStr) -> Result<Self, BoxError> {
                let (negative, radix, digits) = c_digits(to_str(value)?)?;
                let magnitude = u128::from_str_radix(digits, radix)?;
                let n = if negative {
                    0i128.checked_sub_unsigned(magnitude).and_then(|n| <$ty>::try_from(n).ok())
                } else {
                    <$ty>::try_from(magnitude).ok()
                };
                n.map(CInt).ok_or_else(|| CIntError("number out of range").into())
            }
        }
    )*};
}

c_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

/// `key=value`, split at the first `=`: `--defsym=sym=expr`,
/// `-z max-page-size=4096`, `--section-start=.text=0x1000`. The value may hold
/// more `=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyValue<K, V> {
    /// Before the first `=`.
    pub key: K,
    /// After it.
    pub value: V,
}

/// The error of a [`KeyValue`] with no `=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingEquals;

impl std::fmt::Display for MissingEquals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected `key=value`")
    }
}

impl std::error::Error for MissingEquals {}

impl<K: FromArg, V: FromArg> FromArg for KeyValue<K, V> {
    fn from_arg(value: &BStr) -> Result<Self, BoxError> {
        let eq = value.iter().position(|&b| b == b'=').ok_or(MissingEquals)?;
        Ok(KeyValue {
            key: K::from_arg(BStr::new(&value[..eq]))?,
            value: V::from_arg(BStr::new(&value[eq + 1..]))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c<T>(text: &str) -> Result<T, String>
    where
        CInt<T>: FromArg,
    {
        CInt::<T>::from_arg(BStr::new(text))
            .map(|CInt(n)| n)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn c_integers() {
        assert_eq!(c::<u64>("0x400000"), Ok(0x40_0000));
        assert_eq!(c::<u64>("0X1f"), Ok(31));
        assert_eq!(c::<u64>("010"), Ok(8));
        assert_eq!(c::<u64>("0"), Ok(0));
        assert_eq!(c::<u64>("4096"), Ok(4096));
        assert_eq!(c::<i64>("-0x10"), Ok(-16));
        assert_eq!(c::<i8>("-128"), Ok(-128));
        assert!(c::<u8>("256").is_err());
        assert!(c::<u64>("-1").is_err());
        assert!(c::<u64>("08").is_err());
        assert!(c::<u64>("0x").is_err());
        assert!(c::<u64>("").is_err());
    }

    #[test]
    fn key_values() {
        let kv = KeyValue::<String, String>::from_arg(BStr::new("sym=a=b")).unwrap();
        assert_eq!((kv.key.as_str(), kv.value.as_str()), ("sym", "a=b"));
        let kv = KeyValue::<String, CInt<u64>>::from_arg(BStr::new(".text=0x1000")).unwrap();
        assert_eq!((kv.key.as_str(), kv.value.0), (".text", 0x1000));
        assert!(KeyValue::<String, String>::from_arg(BStr::new("nothing")).is_err());
    }
}
