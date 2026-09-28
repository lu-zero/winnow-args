//! What went wrong, and where.
//!
//! [`Error`] is the error type of every parser in this crate. It implements
//! winnow's [`ParserError`] and [`ModalError`] itself rather than living inside
//! an `ErrMode`: `alt` asks [`ParserError::is_backtrack`], so one flag is all
//! the modality a command line needs, and the common case — a branch of an
//! `alt` that did not match — builds a 24-byte value without allocating.

use std::fmt;

use winnow::error::{FromExternalError, ModalError, ParserError};

use crate::help::Style;
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
    /// A required subcommand was not given.
    MissingSubcommand,
    /// `conflicting_flags`: two arguments that exclude each other were both given.
    Conflict,
    /// A required group had none of its members.
    MissingOneOf,
    /// `arg_requires_double_dash`: a word reached an argument that only takes
    /// words after `--`.
    RequiresDoubleDash,
    /// `-h`/`--help`, or a bare invocation of an `arg_required_else_help`
    /// command: show [`Error::render_help`] rather than report a failure.
    HelpRequested,
    /// `-V`/`--version`: show [`Error::version_text`].
    VersionRequested,
    /// A value that its type rejected.
    InvalidValue,
    /// `invalid_choice`: a value outside a fixed set.
    InvalidChoice,
}

/// What `--help` or `--version` asked for, and where.
#[derive(Debug)]
struct Request {
    command: &'static crate::help::Command,
    long: bool,
    /// A bare invocation rather than a flag: help belongs on stderr.
    bare: bool,
    /// Subcommand names from the root down to `command`.
    path: Vec<&'static str>,
}

#[derive(Debug, Default)]
struct Detail {
    request: Option<Request>,
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

    /// `name` requires `target`, which has no value. The offset is the end of the line.
    pub fn required_by(offset: usize, target: impl Into<String>, name: &str) -> Self {
        let mut error = Self::with_token(ErrorKind::MissingRequired, offset, target.into());
        error.detail_mut().value = Some(name.to_owned());
        error
    }

    /// `name` and `other` exclude each other and were both given.
    pub fn conflict(offset: usize, name: impl Into<String>, other: &str) -> Self {
        let mut error = Self::with_token(ErrorKind::Conflict, offset, name.into());
        error.detail_mut().value = Some(other.to_owned());
        error
    }

    /// The required group `group` had none of `members`.
    pub fn missing_one_of(offset: usize, group: impl Into<String>, members: &[&str]) -> Self {
        let mut error = Self::with_token(ErrorKind::MissingOneOf, offset, group.into());
        error.detail_mut().value = Some(members.join(", "));
        error
    }

    /// The word at `offset` reached `name`, which only takes words after `--`.
    pub fn requires_double_dash(offset: usize, name: impl Into<String>) -> Self {
        Self::with_token(ErrorKind::RequiresDoubleDash, offset, name.into())
    }

    fn request(
        kind: ErrorKind,
        command: &'static crate::help::Command,
        long: bool,
        bare: bool,
    ) -> Self {
        Self {
            kind,
            cut: true,
            offset: 0,
            detail: Some(Box::new(Detail {
                request: Some(Request {
                    command,
                    long,
                    bare,
                    path: Vec::new(),
                }),
                ..Detail::default()
            })),
        }
    }

    /// `-h` (`long == false`) or `--help` for `command`.
    pub fn help(command: &'static crate::help::Command, long: bool) -> Self {
        Self::request(ErrorKind::HelpRequested, command, long, false)
    }

    /// A bare invocation of an `arg_required_else_help` command.
    pub fn bare_help(command: &'static crate::help::Command) -> Self {
        Self::request(ErrorKind::HelpRequested, command, false, true)
    }

    /// `-V`/`--version`.
    pub fn version(command: &'static crate::help::Command) -> Self {
        Self::request(ErrorKind::VersionRequested, command, false, false)
    }

    /// Record that this came from within subcommand `name`, for the usage line.
    pub fn within(mut self, name: &'static str) -> Self {
        if let Some(request) = self.detail.as_deref_mut().and_then(|d| d.request.as_mut()) {
            request.path.insert(0, name);
        }
        self
    }

    /// The help to show for a [`ErrorKind::HelpRequested`], `program` leading
    /// the usage line; plain, see [`Error::render_help_styled`] for color.
    pub fn render_help(&self, program: &str) -> Option<String> {
        self.render_help_styled(program, Style::PLAIN)
    }

    /// `program version` for a [`ErrorKind::VersionRequested`].
    pub fn version_text(&self, program: &str) -> Option<String> {
        let request = self.detail.as_deref()?.request.as_ref()?;
        let version = request.command.version?;
        (self.kind == ErrorKind::VersionRequested).then(|| format!("{program} {version}"))
    }

    /// Help for a bare invocation, which is an error to report rather than an answer.
    pub fn is_bare_help(&self) -> bool {
        self.detail
            .as_deref()
            .and_then(|d| d.request.as_ref())
            .is_some_and(|r| r.bare)
    }

    /// A required subcommand was not given. The offset is the end of the line.
    pub fn missing_subcommand(offset: usize) -> Self {
        Self {
            kind: ErrorKind::MissingSubcommand,
            cut: true,
            offset,
            detail: None,
        }
    }

    /// `value` given to `flag` at `offset` was rejected by its type.
    pub fn invalid_value(
        offset: usize,
        flag: impl Into<String>,
        value: &[u8],
        cause: impl Into<BoxError>,
    ) -> Self {
        let cause = cause.into();
        let kind = if cause.is::<crate::value::ChoiceError>() {
            ErrorKind::InvalidChoice
        } else {
            ErrorKind::InvalidValue
        };
        let mut error = Self::with_token(kind, offset, flag.into());
        let detail = error.detail_mut();
        detail.value = Some(String::from_utf8_lossy(value).into_owned());
        detail.cause = Some(cause);
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
        f.write_str(&self.message(Style::PLAIN))
    }
}

impl Error {
    /// What went wrong, as [`Display`](fmt::Display) says it, with what was
    /// typed painted `style.invalid` and what is missing `style.valid`.
    pub fn message(&self, style: Style) -> String {
        let bad = |text: &str| Style::paint(style.invalid, text);
        let good = |text: &str| Style::paint(style.valid, text);
        let token = self.token().unwrap_or_default();
        let value = self.value().unwrap_or_default();
        match self.kind {
            ErrorKind::Backtrack => "invalid command line".to_owned(),
            ErrorKind::UnknownFlag => format!("unknown flag `{}`", bad(token)),
            ErrorKind::UnexpectedArg => format!("unexpected argument `{}`", bad(token)),
            ErrorKind::MissingValue => format!("`{}` needs a value", good(token)),
            ErrorKind::UnexpectedValue => format!(
                "`{}` does not take a value, got `{}`",
                bad(token),
                bad(value)
            ),
            ErrorKind::MissingRequired => match self.value() {
                Some(by) => format!("`{}` is required by `{}`", good(token), bad(by)),
                None => format!("`{}` is required", good(token)),
            },
            ErrorKind::Conflict => format!("`{}` cannot be used with `{}`", bad(token), bad(value)),
            ErrorKind::HelpRequested => "help requested".to_owned(),
            ErrorKind::VersionRequested => "version requested".to_owned(),
            ErrorKind::RequiresDoubleDash => {
                format!("`{}` can only be set after a `--` separator", bad(token))
            }
            ErrorKind::MissingOneOf => format!("one of {} is required ({token})", good(value)),
            ErrorKind::MissingArgument => format!("missing argument `{}`", good(token)),
            ErrorKind::MissingSubcommand => "a subcommand is required".to_owned(),
            ErrorKind::InvalidValue | ErrorKind::InvalidChoice => {
                let mut text = format!("invalid value `{}` for `{token}`", bad(value));
                if let Some(cause) = self.detail.as_deref().and_then(|d| d.cause.as_ref()) {
                    text.push_str(": ");
                    text.push_str(&cause.to_string());
                }
                text
            }
        }
    }

    /// The whole report for an error that is not help or version:
    /// `error: …` and a pointer to `--help`, as clap prints it.
    pub fn render(&self, style: Style) -> String {
        format!(
            "{} {}\n\nFor more information, try '{}'.",
            Style::paint(style.error, "error:"),
            self.message(style),
            Style::paint(style.literal, "--help"),
        )
    }

    /// [`Error::render_help`], painted with `style`.
    pub fn render_help_styled(&self, program: &str, style: Style) -> Option<String> {
        let request = self.detail.as_deref()?.request.as_ref()?;
        if self.kind != ErrorKind::HelpRequested {
            return None;
        }
        let mut path = vec![program];
        path.extend(&request.path);
        let width = crate::help::width();
        Some(crate::help::render_styled(
            request.command,
            &path,
            request.long,
            width,
            style,
        ))
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
