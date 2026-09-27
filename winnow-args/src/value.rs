//! Turning a value's bytes into a Rust type.

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
pub trait FromArg: Sized {
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
