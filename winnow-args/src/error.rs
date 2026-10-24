//! What went wrong, and where.
//!
//! [`Error`] is the error type of every parser in this crate. It implements
//! winnow's [`ParserError`] and [`ModalError`] itself rather than living inside
//! an `ErrMode`: `alt` asks [`ParserError::is_backtrack`], so one flag is all
//! the modality a command line needs, and the common case — a branch of an
//! `alt` that did not match — builds a 24-byte value without allocating.
//!
//! An error that reaches the user is printed by [`Error::render`]: the
//! message, then a tip when a flag, a subcommand or a choice close to what
//! was typed is accepted there ([`Error::similar`]). Only a failed parse
//! looks for one.

use std::fmt;

use winnow::error::{FromExternalError, ModalError, ParserError};

use crate::help::Style;
use crate::stream::Argv;

const NOT_PROVIDED: &str = "the following required arguments were not provided";

mod suggest;

/// Boxed cause of an [`ErrorKind::InvalidValue`].
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The class of failure.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A parser did not match here; another one may.
    Backtrack,
    /// A flag-like word matched no flag.
    UnknownFlag,
    /// A word nothing could hold.
    UnexpectedArg,
    /// A flag needing a value did not get one.
    MissingValue,
    /// A flag that takes no value was given one: `--verbose=yes`.
    UnexpectedValue,
    /// A required flag never appeared.
    MissingRequired,
    /// A required positional argument was never filled.
    MissingArgument,
    /// A required subcommand was not given.
    MissingSubcommand,
    /// Two arguments that exclude each other were both given.
    Conflict,
    /// A required group had none of its members.
    MissingOneOf,
    /// A word reached an argument that only takes words after `--`.
    RequiresDoubleDash,
    /// `-h`/`--help`, or a bare invocation of an `arg_required_else_help`
    /// command: show [`Error::render_help`] rather than report a failure.
    HelpRequested,
    /// `-V`/`--version`: show [`Error::version_text`].
    VersionRequested,
    /// A value that its type rejected.
    InvalidValue,
    /// A value outside a fixed set.
    InvalidChoice,
    /// The parsed values failed the program's own check: a struct's
    /// `validate` function returned [`Error::invalid`].
    Invalid,
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
    /// The command the names start from, once [`Error::suggested`] says:
    /// its page lists the global flags of the commands above it.
    root: Option<&'static crate::help::Command>,
    /// The program was run under its subcommand's name (`multicall`): the
    /// first name in `path` is the one the program was run by.
    applet: bool,
}

#[derive(Debug, Default)]
struct Detail {
    request: Option<Request>,
    /// The flag as spelled on the command line (or its display name), or the word.
    token: String,
    value: Option<String>,
    cause: Option<BoxError>,
    /// Subcommand names from the root down to where it failed.
    path: Vec<&'static str>,
    /// What was probably meant, and what it is: "flag".
    similar: Option<(&'static str, String)>,
    /// The required flags and positionals left without a value after the
    /// first, which `token` names: `--jobs`, `<FILE>`.
    also_missing: Vec<String>,
    /// The help rows of every one of them, once [`Error::suggested`] finds them all.
    rows: Vec<&'static crate::help::Item>,
    /// The program's own message, said in place of the default one.
    wording: Option<&'static str>,
    /// The field whose `requires` named the group a `MissingOneOf` is about.
    by: Option<String>,
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

    /// `self`, a required flag or positional left without a value, also
    /// naming the one `other` reports: every one is listed together.
    #[cold]
    #[inline(never)]
    pub(crate) fn also_missing(mut self, other: Error) -> Self {
        let name = other.missing().next().unwrap_or_default();
        self.detail_mut().also_missing.push(name);
        self
    }

    /// Each required flag or positional left without a value, as a message
    /// spells it: `--jobs`, `<FILE>`.
    fn missing(&self) -> impl Iterator<Item = String> + '_ {
        let detail = self.detail.as_deref();
        let first = detail.map(|d| match self.kind {
            ErrorKind::MissingArgument => format!("<{}>", d.token),
            _ => d.token.clone(),
        });
        let rest = detail
            .into_iter()
            .flat_map(|d| d.also_missing.iter().cloned());
        first.into_iter().chain(rest)
    }

    /// Whether this says required flags or positionals have no value, rather
    /// than that one requires another.
    fn is_not_provided(&self) -> bool {
        match self.kind {
            ErrorKind::MissingArgument => true,
            ErrorKind::MissingRequired => self.value().is_none(),
            _ => false,
        }
    }

    /// `self`, said with `wording` in place of its default message: `{0}` the
    /// field that declares the rule and `{1}` the other, or a group's name
    /// and its members when none of them was given.
    #[cold]
    #[inline(never)]
    pub(crate) fn worded(mut self, wording: &'static str) -> Self {
        self.detail_mut().wording = Some(wording);
        self
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

    /// `name` requires a member of `group`, and none has a value. The offset
    /// is the end of the line.
    pub fn one_of_required_by(
        offset: usize,
        group: impl Into<String>,
        members: &[&str],
        name: &str,
    ) -> Self {
        let mut error = Self::missing_one_of(offset, group, members);
        error.detail_mut().by = Some(name.to_owned());
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
                    root: None,
                    applet: false,
                }),
                ..Detail::default()
            })),
        }
    }

    /// `-h` (`long == false`) or `--help` for `command`: the short page
    /// either way when it has `brief_help`.
    pub fn help(command: &'static crate::help::Command, long: bool) -> Self {
        // `brief_help` makes every request the short page: `--help`, and
        // `help <command>` from the command above.
        let long = long && !command.brief_help;
        Self::request(ErrorKind::HelpRequested, command, long, false)
    }

    /// A bare invocation of an `arg_required_else_help` command.
    pub fn bare_help(command: &'static crate::help::Command) -> Self {
        Self::request(ErrorKind::HelpRequested, command, false, true)
    }

    /// `-V`, or a `--version` of a command without a `long_version`.
    pub fn version(command: &'static crate::help::Command) -> Self {
        Self::request(ErrorKind::VersionRequested, command, false, false)
    }

    /// `--version`: the command's `long_version`, else its version.
    pub fn long_version(command: &'static crate::help::Command) -> Self {
        Self::request(ErrorKind::VersionRequested, command, true, false)
    }

    /// Record that this came from within subcommand `name`, for the usage line
    /// and for what [`Error::suggested`] looks among.
    #[cold]
    #[inline(never)]
    pub fn within(mut self, name: &'static str) -> Self {
        if let Some(detail) = self.detail.as_deref_mut() {
            match detail.request.as_mut() {
                Some(request) => request.path.insert(0, name),
                None => detail.path.insert(0, name),
            }
        }
        self
    }

    /// Record that the program was run under the name of the subcommand
    /// this came from within (`multicall`): its help is the program's own,
    /// and the offset counts from the first word after the name, which the
    /// line was given `name` in front of.
    #[cold]
    #[inline(never)]
    pub(crate) fn applet(mut self, name: &str) -> Self {
        self.offset = self.offset.saturating_sub(name.len() + 1);
        if let Some(request) = self.detail.as_deref_mut().and_then(|d| d.request.as_mut()) {
            request.applet = true;
        }
        self
    }

    /// Look among what `root` and the subcommand it failed in take for what
    /// an unknown flag or word was probably meant to be: [`Error::similar`]
    /// then has it. A request for a subcommand's help learns the global
    /// flags of the commands above it, which its page then lists.
    /// [`Args::parse_words`](crate::Args::parse_words) and what
    /// is built on it do this; a parse through
    /// [`Args::parse_argv`](crate::Args::parse_argv) does not.
    #[cold]
    #[inline(never)]
    pub fn suggested(mut self, root: &'static crate::help::Command) -> Self {
        let Some(detail) = self.detail.as_deref_mut() else {
            return self;
        };
        if let Some(request) = detail.request.as_mut() {
            request.root = Some(root);
            return self;
        }
        let found = match self.kind {
            ErrorKind::UnknownFlag => {
                suggest::flag(root, &detail.path, &detail.token).map(|flag| ("flag", flag))
            }
            ErrorKind::UnexpectedArg => suggest::subcommand(root, &detail.path, &detail.token)
                .map(|name| ("subcommand", name.to_owned())),
            _ => None,
        };
        if let Some(found) = found {
            detail.similar = Some(found);
        }
        if self.is_not_provided() {
            let path = self
                .detail
                .as_deref()
                .map(|d| d.path.clone())
                .unwrap_or_default();
            let rows = self.missing().map(|name| suggest::item(root, &path, &name));
            // A row for each, or none: a partial list would hide one.
            let rows: Option<Vec<_>> = rows.collect();
            self.detail_mut().rows = rows.unwrap_or_default();
        }
        self
    }

    /// What the flag, subcommand or value at fault was probably meant to be,
    /// when something close to it is accepted there.
    pub fn similar(&self) -> Option<&str> {
        let (_, similar) = self.detail.as_deref()?.similar.as_ref()?;
        Some(similar)
    }

    /// The help to show for a [`ErrorKind::HelpRequested`], `program` leading
    /// the usage line; plain, see [`Error::render_help_styled`] for color.
    pub fn render_help(&self, program: &str) -> Option<String> {
        self.render_help_styled(program, Style::PLAIN)
    }

    /// `program version` for a [`ErrorKind::VersionRequested`], or
    /// `program long_version` for a `--version` of a command that has one.
    pub fn version_text(&self, program: &str) -> Option<String> {
        let request = self.detail.as_deref()?.request.as_ref()?;
        let version = request.command.version?;
        let version = match request.command.long_version {
            Some(long) if request.long => long,
            _ => version,
        };
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
        let choices = cause.downcast_ref::<crate::value::ChoiceError>();
        let kind = match choices {
            Some(_) => ErrorKind::InvalidChoice,
            None => ErrorKind::InvalidValue,
        };
        let value = String::from_utf8_lossy(value).into_owned();
        let similar = choices
            .and_then(|error| suggest::nearest(&value, error.choices.iter().copied()))
            .map(|choice| ("value", choice.to_owned()));
        let mut error = Self::with_token(kind, offset, flag.into());
        let detail = error.detail_mut();
        detail.value = Some(value);
        detail.cause = Some(cause);
        detail.similar = similar;
        error
    }

    /// The parsed values are refused for `reason`: what a struct's `validate`
    /// function returns when the fault is not one value's, as
    /// [`Error::invalid_value`] is for one value.
    ///
    /// ```
    /// use winnow_args::{Error, ErrorKind};
    ///
    /// let error = Error::invalid("`--root` cannot be used with `crossdev`");
    /// assert_eq!(error.kind(), ErrorKind::Invalid);
    /// assert_eq!(error.to_string(), "`--root` cannot be used with `crossdev`");
    /// ```
    pub fn invalid(reason: impl Into<BoxError>) -> Self {
        let mut error = Self::with_token(ErrorKind::Invalid, 0, String::new());
        error.detail_mut().cause = Some(reason.into());
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
    /// typed painted in the palette's `invalid` and what is missing in `valid`.
    pub fn message(&self, style: Style) -> String {
        let bad = |text: &str| Style::paint(style.invalid(), text);
        let good = |text: &str| Style::paint(style.valid(), text);
        let token = self.token().unwrap_or_default();
        let value = self.value().unwrap_or_default();
        let by = self.detail.as_deref().and_then(|d| d.by.as_deref());
        if let Some(wording) = self.detail.as_deref().and_then(|d| d.wording) {
            // `required_by` holds the target first; the field that requires it
            // declared the rule.
            let (first, second) = match (self.kind, by) {
                (_, Some(by)) => (by, value),
                (ErrorKind::MissingRequired, _) => (value, token),
                _ => (token, value),
            };
            return wording.replace("{0}", first).replace("{1}", second);
        }
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
            ErrorKind::MissingRequired if !self.is_not_provided() => {
                format!("`{}` is required by `{}`", good(token), bad(value))
            }
            ErrorKind::MissingRequired | ErrorKind::MissingArgument => {
                let names: Vec<String> =
                    self.missing().map(|n| format!("`{}`", good(&n))).collect();
                format!("{NOT_PROVIDED}: {}", names.join(", "))
            }
            ErrorKind::Conflict => format!("`{}` cannot be used with `{}`", bad(token), bad(value)),
            ErrorKind::HelpRequested => "help requested".to_owned(),
            ErrorKind::VersionRequested => "version requested".to_owned(),
            ErrorKind::RequiresDoubleDash => {
                format!("`{}` can only be set after a `--` separator", bad(token))
            }
            ErrorKind::MissingOneOf => match by {
                Some(by) => {
                    let members: Vec<String> = value
                        .split(", ")
                        .map(|m| format!("`{}`", good(m)))
                        .collect();
                    format!("`{}` requires one of {}", bad(by), members.join(", "))
                }
                None => format!("one of {} is required ({token})", good(value)),
            },
            ErrorKind::MissingSubcommand => "a subcommand is required".to_owned(),
            ErrorKind::Invalid => self
                .detail
                .as_deref()
                .and_then(|d| d.cause.as_ref())
                .map_or_else(|| "invalid command line".to_owned(), ToString::to_string),
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

    /// `tip: a similar flag exists: …` for an error that has [`Error::similar`],
    /// painted as what is missing is.
    pub fn tip(&self, style: Style) -> Option<String> {
        let (what, similar) = self.detail.as_deref()?.similar.as_ref()?;
        Some(format!(
            "{} a similar {what} exists: `{}`",
            Style::paint(style.valid(), "tip:"),
            Style::paint(style.valid(), similar),
        ))
    }

    /// The whole report for an error that is not help or version:
    /// `error: …`, a tip when something close was accepted, and a pointer to
    /// `--help`. Required flags and positionals left without a value are
    /// listed as `--help` shows them, once [`Error::suggested`] found them.
    pub fn render(&self, style: Style) -> String {
        let more = format!(
            "For more information, try '{}'.",
            Style::paint(style.flag(), "--help")
        );
        let error = Style::paint(style.error(), "error:");
        if let Some(detail) = self.detail.as_deref().filter(|d| !d.rows.is_empty()) {
            let rows = crate::help::rows(&detail.rows, style);
            return format!("{error} {NOT_PROVIDED}:\n\n{rows}\n{more}");
        }
        let tip = self.tip(style).map(|tip| format!("\n\n  {tip}"));
        let tip = tip.unwrap_or_default();
        format!("{error} {}{tip}\n\n{more}", self.message(style))
    }

    /// [`Error::render_help`], painted with `style`.
    pub fn render_help_styled(&self, program: &str, style: Style) -> Option<String> {
        let request = self.detail.as_deref()?.request.as_ref()?;
        if self.kind != ErrorKind::HelpRequested {
            return None;
        }
        // Run as a subcommand, `program` is the name it was run by, in
        // place of the subcommand's, which may be an alias's.
        let names = request.path.get(usize::from(request.applet)..);
        let mut path = vec![program];
        path.extend(names.unwrap_or_default());
        let width = crate::help::width();
        let inherited = request
            .root
            .map(|root| suggest::inherited(root, &request.path, request.command))
            .unwrap_or_default();
        Some(crate::help::render_page(
            request.command,
            &path,
            request.long,
            width,
            style,
            &inherited,
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
