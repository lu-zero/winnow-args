//! What went wrong, and where.
//!
//! [`Error`] is the error type of every parser in this crate. It implements
//! winnow's [`ParserError`] and [`ModalError`] itself rather than living inside
//! an `ErrMode`: `alt` asks [`ParserError::is_backtrack`], so one flag is all
//! the modality a command line needs, and the common case — a branch of an
//! `alt` that did not match — builds a 24-byte value without allocating.

use std::fmt;

use winnow::error::{FromExternalError, ModalError, ParserError};

use crate::stream::Argv;

/// Boxed cause of an [`ErrorKind::InvalidValue`].
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The class of failure, after usage's argv grammar error codes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A parser did not match here; another one may.
    Backtrack,
    /// `unknown_flag`: a flag-like word matched no flag.
    UnknownFlag,
    /// `unexpected_arg`: a word nothing could hold.
    UnexpectedArg,
    /// `missing_flag_value`: a flag needing a value did not get one.
    MissingValue,
    /// A flag that takes no value was given one: `--verbose=yes`.
    UnexpectedValue,
    /// `missing_required_flag`: a required flag never appeared.
    MissingRequired,
    /// `missing_required_arg`: a required positional argument was never filled.
    MissingArgument,
    /// A value that its type rejected.
    InvalidValue,
}

#[derive(Debug, Default)]
struct Detail {
    /// The flag as spelled on the command line (or its display name), or the word.
    token: String,
    value: Option<String>,
    cause: Option<BoxError>,
}

/// A parse failure.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cut: bool,
    offset: usize,
    detail: Option<Box<Detail>>,
}

impl Error {
    fn with_token(kind: ErrorKind, offset: usize, token: String) -> Self {
        Self {
            kind,
            cut: true,
            offset,
            detail: Some(Box::new(Detail {
                token,
                ..Detail::default()
            })),
        }
    }

    /// A flag-like word, at `offset`, that matched nothing.
    pub fn unknown_flag(offset: usize, token: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::UnknownFlag, offset, token.into())
    }

    /// A word, at `offset`, that nothing could hold.
    pub fn unexpected_arg(offset: usize, token: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::UnexpectedArg, offset, token.into())
    }

    /// `flag`, at `offset`, needed a value and did not get one.
    pub fn missing_value(offset: usize, flag: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::MissingValue, offset, flag.into())
    }

    /// `flag`, at `offset`, takes no value but was given `value`.
    pub fn unexpected_value(offset: usize, flag: impl Into<String>, value: &[u8]) -> Self {
        let mut error = Self::with_token(ErrorKind::UnexpectedValue, offset, flag.into());
        error.detail_mut().value = Some(String::from_utf8_lossy(value).into_owned());
        error
    }

    /// The required `flag` never appeared. The offset is the end of the line.
    pub fn missing_required(offset: usize, flag: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::MissingRequired, offset, flag.into())
    }

    /// The required positional `name` was never filled. The offset is the end of the line.
    pub fn missing_argument(offset: usize, name: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::MissingArgument, offset, name.into())
    }

    /// `value` given to `flag` at `offset` was rejected by its type.
    pub fn invalid_value(
        offset: usize,
        flag: impl Into<String>,
        value: &[u8],
        cause: impl Into<BoxError>,
    ) -> Self {
        let mut error = Self::with_token(ErrorKind::InvalidValue, offset, flag.into());
        let detail = error.detail_mut();
        detail.value = Some(String::from_utf8_lossy(value).into_owned());
        detail.cause = Some(cause.into());
        error
    }

    fn detail_mut(&mut self) -> &mut Detail {
        self.detail.get_or_insert_with(Default::default)
    }

    /// The class of failure.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Byte offset into the flattened command line.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The flag or word the failure is about, if there is one.
    pub fn token(&self) -> Option<&str> {
        self.detail.as_deref().map(|d| d.token.as_str())
    }

    /// The offending value, for [`ErrorKind::UnexpectedValue`] and [`ErrorKind::InvalidValue`].
    pub fn value(&self) -> Option<&str> {
        self.detail.as_deref()?.value.as_deref()
    }

    /// Whether backtracking past this error is forbidden.
    pub fn is_cut(&self) -> bool {
        self.cut
    }
}

impl PartialEq for Error {
    /// Equal when they report the same thing at the same place; causes are not compared.
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.offset == other.offset
            && self.token() == other.token()
            && self.value() == other.value()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token = self.token().unwrap_or_default();
        match self.kind {
            ErrorKind::Backtrack => f.write_str("invalid command line"),
            ErrorKind::UnknownFlag => write!(f, "unknown flag `{token}`"),
            ErrorKind::UnexpectedArg => write!(f, "unexpected argument `{token}`"),
            ErrorKind::MissingValue => write!(f, "`{token}` needs a value"),
            ErrorKind::UnexpectedValue => write!(
                f,
                "`{token}` does not take a value, got `{}`",
                self.value().unwrap_or_default()
            ),
            ErrorKind::MissingRequired => write!(f, "`{token}` is required"),
            ErrorKind::MissingArgument => write!(f, "missing argument `{token}`"),
            ErrorKind::InvalidValue => {
                write!(
                    f,
                    "invalid value `{}` for `{token}`",
                    self.value().unwrap_or_default()
                )?;
                if let Some(cause) = self.detail.as_deref().and_then(|d| d.cause.as_ref()) {
                    write!(f, ": {cause}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let cause = self.detail.as_deref()?.cause.as_ref()?;
        Some(&**cause as &(dyn std::error::Error + 'static))
    }
}

impl<'i> ParserError<Argv<'i>> for Error {
    type Inner = Self;

    #[inline]
    fn from_input(input: &Argv<'i>) -> Self {
        Self {
            kind: ErrorKind::Backtrack,
            cut: false,
            offset: input.offset(),
            detail: None,
        }
    }

    /// Keep whichever of the two says more: a specific failure beats a bare
    /// "did not match", and otherwise the later branch wins, as in winnow.
    #[inline]
    fn or(self, other: Self) -> Self {
        if other.kind == ErrorKind::Backtrack && self.kind != ErrorKind::Backtrack {
            self
        } else {
            other
        }
    }

    #[inline(always)]
    fn is_backtrack(&self) -> bool {
        !self.cut
    }

    #[inline(always)]
    fn into_inner(self) -> Result<Self::Inner, Self> {
        Ok(self)
    }
}

impl ModalError for Error {
    #[inline(always)]
    fn cut(mut self) -> Self {
        self.cut = true;
        self
    }

    #[inline(always)]
    fn backtrack(mut self) -> Self {
        self.cut = false;
        self
    }
}

/// Lets `try_map` report a conversion failure. It backtracks, like any winnow
/// error; the flag parsers in [`crate::combinator`] convert values themselves
/// and commit, so their errors can name the flag.
impl<'i, E> FromExternalError<Argv<'i>, E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from_external_error(input: &Argv<'i>, e: E) -> Self {
        let mut error = Self::from_input(input);
        error.kind = ErrorKind::InvalidValue;
        error.detail_mut().cause = Some(Box::new(e));
        error
    }
}
