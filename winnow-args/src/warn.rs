//! What a command line used that its program says not to use any more: a
//! deprecated flag, positional or subcommand, or a value read from a
//! deprecated environment variable.
//!
//! A parse records them and goes on; nothing here prints. [`Args::parse`]
//! prints those due on standard error; after any other entry point,
//! [`Args::warnings`] takes them, so a program can report them its own way,
//! or once its logging is up. The rules follow usage's, for the same
//! declarations: <https://usage.jdx.dev/spec/argv>.
//!
//! [`Args::parse`]: crate::Args::parse
//! [`Args::warnings`]: crate::Args::warnings

use std::cell::RefCell;
use std::cmp::Ordering;

use crate::help::Style;

thread_local! {
    static RECORDED: RefCell<Vec<Warning>> = const { RefCell::new(Vec::new()) };
}

/// Why a flag, a positional or a subcommand is deprecated, and when: what
/// help and the pages say of it, and what its warning carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Deprecation {
    /// The author's reason, `""` when none is given.
    pub message: &'static str,
    /// The release warnings start at; before it, using it is quiet.
    pub warn_at: Option<&'static str>,
    /// The release it goes away in.
    pub remove_at: Option<&'static str>,
}

impl Deprecation {
    /// A deprecation for `#[arg(deprecated = message, deprecated_warn_at =
    /// …, deprecated_remove_at = …)]`, as the derive writes it: a later
    /// release can add to it without breaking a program.
    pub const fn new(
        message: &'static str,
        warn_at: Option<&'static str>,
        remove_at: Option<&'static str>,
    ) -> Self {
        Self {
            message,
            warn_at,
            remove_at,
        }
    }
}

/// What sort of declaration a [`Warning`] is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WarningKind {
    /// A flag or a positional whose declaration says not to use it.
    Argument,
    /// A subcommand whose declaration says not to use it.
    Command,
    /// A value read from a deprecated name of an environment variable.
    Env,
}

/// One thing a command line used that its program says not to use any more.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Warning {
    /// What it was about.
    pub kind: WarningKind,
    /// What was used: `--old`, `<FILE>`, `old-command`, `OLD_VAR`.
    pub name: String,
    /// Why, and when, for a flag, a positional or a subcommand.
    pub deprecation: Option<Deprecation>,
    /// What to use instead: the variable to read for a deprecated one.
    pub replacement: Option<&'static str>,
}

impl Warning {
    /// Whether a program at `version` warns about it yet: true unless
    /// `warn_at` is a later release than `version`. A missing or unreadable
    /// version warns, so a mistake is noisy rather than silent.
    pub fn is_due(&self, version: Option<&str>) -> bool {
        let warn_at = self.deprecation.and_then(|d| d.warn_at);
        let (Some(warn_at), Some(version)) = (warn_at, version) else {
            return true;
        };
        compare(version, warn_at) != Some(Ordering::Less)
    }

    /// `warning: --old is deprecated, removed at 2.0: use --new`, plain.
    pub fn render(&self) -> String {
        self.render_styled(Style::PLAIN)
    }

    /// [`Warning::render`], painted with `style`: the label as help paints a
    /// warning.
    pub fn render_styled(&self, style: Style) -> String {
        let label = Style::paint(style.error(), "warning:");
        let subject = match self.kind {
            WarningKind::Command => format!("the {} command", self.name),
            WarningKind::Env => format!("the variable {}", self.name),
            _ => self.name.clone(),
        };
        let mut out = format!("{label} {subject} is deprecated");
        let deprecation = self.deprecation.unwrap_or(Deprecation {
            message: "",
            warn_at: None,
            remove_at: None,
        });
        if let Some(at) = deprecation.remove_at {
            out.push_str(&format!(", removed at {at}"));
        }
        match (deprecation.message, self.replacement) {
            ("", Some(instead)) => out.push_str(&format!(": use {instead}")),
            ("", None) => {}
            (message, _) => out.push_str(&format!(": {message}")),
        }
        out.push('\n');
        out
    }
}

/// Record `warning` for [`take`]; one already recorded is not repeated.
#[cold]
#[inline(never)]
pub(crate) fn record(warning: Warning) {
    RECORDED.with(|recorded| {
        let mut recorded = recorded.borrow_mut();
        if !recorded.contains(&warning) {
            recorded.push(warning);
        }
    });
}

/// The warnings parses on this thread recorded since the last call, those
/// due at `version`.
pub(crate) fn take(version: Option<&str>) -> Vec<Warning> {
    let mut taken = RECORDED.with(|recorded| std::mem::take(&mut *recorded.borrow_mut()));
    taken.retain(|warning| warning.is_due(version));
    taken
}

/// Order two versions, or `None` if either is not one: dotted integers left
/// to right, a missing one read as zero, a `-suffix` before the same
/// numbers without one, `+build` ignored. usage's rule, and a leading `v`
/// is read as none; the parts of a suffix compare as numbers where both are.
fn compare(a: &str, b: &str) -> Option<Ordering> {
    let split = |v: &str| {
        let v = v.strip_prefix('v').unwrap_or(v);
        let v = v.split('+').next().unwrap_or_default();
        match v.split_once('-') {
            Some((core, pre)) => (core.to_owned(), Some(pre.to_owned())),
            None => (v.to_owned(), None),
        }
    };
    let ((a_core, a_pre), (b_core, b_pre)) = (split(a), split(b));
    let numbers =
        |core: &str| -> Option<Vec<u64>> { core.split('.').map(|n| n.parse().ok()).collect() };
    let (a_n, b_n) = (numbers(&a_core)?, numbers(&b_core)?);
    for at in 0..a_n.len().max(b_n.len()) {
        let (x, y) = (a_n.get(at).unwrap_or(&0), b_n.get(at).unwrap_or(&0));
        match x.cmp(y) {
            Ordering::Equal => {}
            other => return Some(other),
        }
    }
    Some(match (a_pre, b_pre) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(a), Some(b)) => prerelease(&a, &b),
    })
}

/// Two suffixes, part by part: `rc.2` before `rc.10`.
fn prerelease(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.split('.'), b.split('.'));
    loop {
        let order = match (a.next(), b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match (x.parse::<u64>(), y.parse::<u64>()) {
                (Ok(x), Ok(y)) => x.cmp(&y),
                _ => x.cmp(y),
            },
        };
        if order != Ordering::Equal {
            return order;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_as_usage_does() {
        assert_eq!(compare("1.2", "1.2.0"), Some(Ordering::Equal));
        assert_eq!(compare("1.10", "1.9"), Some(Ordering::Greater));
        assert_eq!(compare("2.0-rc1", "2.0"), Some(Ordering::Less));
        assert_eq!(compare("2.0+abc", "2.0"), Some(Ordering::Equal));
        assert_eq!(compare("next", "1.0"), None);
        assert_eq!(compare("v1.2", "1.2"), Some(Ordering::Equal));
        assert_eq!(compare("2.0-rc.2", "2.0-rc.10"), Some(Ordering::Less));
    }

    #[test]
    fn a_warning_is_due_from_its_release_on() {
        let at = |warn_at| Warning {
            kind: WarningKind::Argument,
            name: "--old".into(),
            deprecation: Some(Deprecation {
                message: "",
                warn_at,
                remove_at: None,
            }),
            replacement: None,
        };
        assert!(!at(Some("2.0")).is_due(Some("1.9")));
        assert!(at(Some("2.0")).is_due(Some("2.0")));
        assert!(at(Some("2.0")).is_due(None));
        assert!(at(Some("soon")).is_due(Some("1.0")));
        assert!(at(None).is_due(Some("1.0")));
    }
}
