//! Environment variables as a source of values, after the command line and
//! before defaults.

use std::cell::RefCell;
use std::ffi::{OsStr, OsString};

use winnow::stream::BStr;

use crate::error::Error;
use crate::value::FromArg;

thread_local! {
    static OVERRIDE: RefCell<Option<Vec<(OsString, OsString)>>> = const { RefCell::new(None) };
}

/// Run `f` with parses on this thread reading `vars` instead of the process
/// environment: for tests, which then need neither `unsafe` `set_var` nor to
/// serialize against each other.
pub fn with_env<R>(vars: &[(&str, &str)], f: impl FnOnce() -> R) -> R {
    let vars = vars
        .iter()
        .map(|(k, v)| (OsString::from(k), OsString::from(v)))
        .collect();
    /// Puts the previous override back, even when `f` panics.
    struct Restore(Option<Vec<(OsString, OsString)>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            OVERRIDE.with(|o| *o.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(OVERRIDE.with(|o| o.replace(Some(vars))));
    f()
}

/// Whether [`with_env`] stands in for the environment on this thread: then
/// the process's surroundings, the terminal included, are not consulted either.
pub(crate) fn overridden() -> bool {
    OVERRIDE.with(|o| o.borrow().is_some())
}

/// The variable `name`, from the override set by [`with_env`] if there is one.
pub fn var(name: &str) -> Option<OsString> {
    OVERRIDE.with(|o| match &*o.borrow() {
        Some(vars) => vars
            .iter()
            .find(|(k, _)| k == OsStr::new(name))
            .map(|(_, v)| v.clone()),
        None => std::env::var_os(name),
    })
}

/// A switch set from the environment: true unless empty, `0`, `false`, `no` or `off`.
pub fn truthy(value: &OsStr) -> bool {
    !matches!(
        value.as_encoded_bytes(),
        b"" | b"0" | b"false" | b"no" | b"off"
    )
}

/// The variable `name` converted with [`FromArg`]; errors name it as `$name`.
pub fn value<T: FromArg>(name: &str) -> Result<Option<T>, Error> {
    let Some(raw) = var(name) else {
        return Ok(None);
    };
    let bytes = BStr::new(raw.as_encoded_bytes());
    T::from_arg(bytes)
        .map(Some)
        .map_err(|cause| Error::invalid_value(0, format!("${name}"), bytes, cause))
}
