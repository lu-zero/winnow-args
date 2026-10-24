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

    /// Whether [`FromArg::choice_help`] says something of its own: help then
    /// asks it. `#[derive(ValueEnum)]` sets it when a visible variant has a
    /// doc comment or `help`.
    const DESCRIBES_CHOICES: bool = false;

    /// What the choice at `index` of a field's choices means, for the long
    /// help, which lists them one a line; `""` says nothing.
    ///
    /// `default` is the text written for it in the source: the field's
    /// `choices("fast" = "…")`, or the variant's doc comment. It is what the
    /// man and Markdown pages show, as they are written from the source when
    /// the code is checked, for any target. The program asks this when it
    /// renders help, so an implementation by hand can answer from what only
    /// the running program has, such as the docs another crate's enum keeps
    /// through `strum`, and fall back to `default`. The default answer is
    /// `default`.
    ///
    /// ```
    /// use winnow_args::{BStr, FromArg, error::BoxError};
    ///
    /// struct Level(u8);
    ///
    /// impl FromArg for Level {
    ///     const CHOICES: &'static [&'static str] = &["low", "high"];
    ///     const DESCRIBES_CHOICES: bool = true;
    ///
    ///     fn choice_help(index: usize, default: &'static str) -> &'static str {
    ///         match index {
    ///             1 if cfg!(unix) => "as loud as the terminal allows",
    ///             _ => default,
    ///         }
    ///     }
    ///
    ///     fn from_arg(value: &BStr) -> Result<Self, BoxError> {
    ///         Ok(Level(u8::from(&**value == b"high")))
    ///     }
    /// }
    /// # let _ = Level(0).0;
    /// ```
    fn choice_help(index: usize, default: &'static str) -> &'static str {
        let _ = index;
        default
    }

    /// The name `#[derive(ValueEnum)]` files its choices under when the
    /// documentation is built. Empty for an implementation written by hand.
    #[doc(hidden)]
    const SPEC: &'static str = "";

    /// What the value is, for the shell that completes it, when a field
    /// states no `hint`. Nothing a shell knows by default: a number, an
    /// address, a type of the program. A `PathBuf`, a `String` and an
    /// `OsString` are a file: a word is one more often than not, and a shell
    /// that offers files stops no other word from being typed.
    const HINT: crate::complete::Hint = crate::complete::Hint::Nothing;

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
    const HINT: crate::complete::Hint = crate::complete::Hint::Files;

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
    const HINT: crate::complete::Hint = crate::complete::Hint::Files;

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
    const HINT: crate::complete::Hint = crate::complete::Hint::Files;

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

/// Whether a documentation build finds `T`'s choices by `path`: it has none,
/// or `#[derive(ValueEnum)]` filed them where `path` leads. A type named
/// through an alias, or one that implements [`FromArg`] by hand, does not.
#[doc(hidden)]
pub const fn documented_choices<T: FromArg>(path: &str) -> bool {
    if T::SPEC.is_empty() {
        T::CHOICES.is_empty()
    } else {
        filed_as(T::SPEC, path)
    }
}

/// Whether a documentation build finds a type by `path`, as a field writes
/// it: `add::Opts`, `crate::Opts`, `dep::Opts`, or the name it states.
///
/// `spec` is the type's, `CRATE:MODULES:Name` from a derive in such a build.
/// The name is the last of `path`. What is before it starts at a crate's
/// root (`crate::`, or the name of the type's crate), or names modules of the
/// type, one after the other: the module it is in, or one above that brings
/// it out with a `pub use`. `winnow-args-spec` looks for the same, so that it
/// never takes another type of the name for this. An empty `spec` is a type written by hand, and
/// filed by hand.
#[doc(hidden)]
pub const fn filed_as(spec: &str, path: &str) -> bool {
    let (spec, path) = (spec.as_bytes(), path.as_bytes());
    if spec.is_empty() {
        return true;
    }
    // CRATE:MODULES:Name, or the name alone.
    let (mut first, mut last, mut at) = (spec.len(), 0, 0);
    while at < spec.len() {
        if spec[at] == b':' {
            if first == spec.len() {
                first = at;
            }
            last = at + 1;
        }
        at += 1;
    }
    // `a::b::Name`: the name, and the modules before it.
    let mut name = path.len();
    while name > 0 && path[name - 1] != b':' {
        name -= 1;
    }
    if !same_bytes(spec, last, spec.len(), path, name, path.len()) {
        return false;
    }
    if first == spec.len() || name == 0 {
        return true;
    }
    // From a crate's root, a `pub use` may bring the type out anywhere.
    let to = name - 2;
    if starts_with(path, b"crate::") || first + 2 >= last {
        return true;
    }
    if among_modules(spec, first + 1, last - 1, path, 0, to) {
        return true;
    }
    let mut end = 0;
    while end < to && path[end] != b':' {
        end += 1;
    }
    same_bytes(spec, 0, first, path, 0, end)
}

/// Whether the modules `a/b/c` in `spec` have the modules `a::b` or `b::c` in
/// `path` among them, in a row.
const fn among_modules(
    spec: &[u8],
    start: usize,
    mut end: usize,
    path: &[u8],
    from: usize,
    to: usize,
) -> bool {
    loop {
        if ends_with_modules(spec, start, end, path, from, to) {
            return true;
        }
        while end > start && spec[end - 1] != b'/' {
            end -= 1;
        }
        if end == start {
            return false;
        }
        end -= 1;
    }
}

/// Whether the modules `a/b/c` in `spec` end with the modules `b::c` in `path`.
const fn ends_with_modules(
    spec: &[u8],
    start: usize,
    mut end: usize,
    path: &[u8],
    from: usize,
    mut to: usize,
) -> bool {
    loop {
        let (mut module, mut written) = (end, to);
        while module > start && spec[module - 1] != b'/' {
            module -= 1;
        }
        while written > from && path[written - 1] != b':' {
            written -= 1;
        }
        if !same_bytes(spec, module, end, path, written, to) {
            return false;
        }
        if written == from {
            return true;
        }
        if module == start {
            return false;
        }
        (end, to) = (module - 1, written - 2);
    }
}

const fn starts_with(text: &[u8], prefix: &[u8]) -> bool {
    text.len() >= prefix.len() && same_bytes(text, 0, prefix.len(), prefix, 0, prefix.len())
}

const fn same_bytes(a: &[u8], from: usize, to: usize, b: &[u8], start: usize, end: usize) -> bool {
    if to - from != end - start {
        return false;
    }
    let mut at = 0;
    while at < to - from {
        if a[from + at] != b[start + at] {
            return false;
        }
        at += 1;
    }
    true
}

/// Whether the choices a field states are those of its type `T`, when `T`
/// has any: what a documentation build shows is then what is accepted.
#[doc(hidden)]
pub const fn stated_choices<T: FromArg>(stated: &[&str]) -> bool {
    if T::CHOICES.is_empty() {
        return true;
    }
    if T::CHOICES.len() != stated.len() {
        return false;
    }
    let mut at = 0;
    while at < stated.len() {
        if !same(T::CHOICES[at], stated[at]) {
            return false;
        }
        at += 1;
    }
    true
}

const fn same(a: &str, b: &str) -> bool {
    same_bytes(a.as_bytes(), 0, a.len(), b.as_bytes(), 0, b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_leads_to_a_type_by_its_modules_or_its_crate() {
        let spec = "app:src/cli/add:Opts";
        for path in [
            "Opts",
            "add::Opts",
            "cli::add::Opts",
            "cli::Opts",
            "crate::Opts",
            "crate::cli::add::Opts",
            "app::Opts",
            "app::cli::Opts",
            "app::src::cli::add::Opts",
            "app::prelude::Opts",
            "crate::prelude::Opts",
        ] {
            assert!(filed_as(spec, path), "{path}");
        }
        for path in [
            "AddOpts",
            "remove::Opts",
            "add::cli::Opts",
            "src::add::Opts",
            "dep::Opts",
            "dep::cli::add::Opts",
            "std::net::Opts",
        ] {
            assert!(!filed_as(spec, path), "{path}");
        }
        // The name alone, as any other build has it, and a type written by hand.
        assert!(filed_as("Opts", "anywhere::Opts"));
        assert!(!filed_as("Opts", "anywhere::Other"));
        assert!(filed_as("", "anywhere::Opts"));
        assert!(filed_as("app::Opts", "x::Opts"));
    }

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
