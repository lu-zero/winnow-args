//! mold's command line parsed with winnow-args (`--features winnow-args`).
//!
//! Every option is a variant of [`Item`], lexed by winnow-args with GNU ld's
//! dash rules (`long_only`) and kept in command-line order; `parse_args`
//! then folds the sequence through the same handling as the built-in parser.
//! `-z` keywords are mapped to the option they stand for by [`z_opt`].
//! Generated from `parse_args`'s option chain.

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};

use winnow_args::value::Spanned;
use winnow_args::{Args, ErrorKind, Occurrence};

use crate::fatal;

@ENUM@

/// The whole command line: every option, in order.
#[derive(Args)]
#[arg(long_only, disable_help_flag, disable_version_flag, disable_help_subcommand)]
struct Cli {
    #[arg(sequence, unknown)]
    opts: Vec<Item>,
}

/// Options that only take an attached value (`--name=value`): given bare,
/// they are unknown options, as in the built-in parser.
const EQ_ONLY: &[&str] = &[
@EQ@,
];

/// Parses `raw_cmdline`, which includes the program name.
pub(crate) fn parse(raw_cmdline: &[Cow<'_, OsStr>]) -> Vec<Item> {
    let words: Vec<&winnow_args::__private::BStr> = raw_cmdline
        .iter()
        .skip(1)
        .map(|word| winnow_args::__private::BStr::new(word.as_encoded_bytes()))
        .collect();
    match Cli::parse_from(&words) {
        Ok(cli) => cli.opts,
        Err(error) => {
            let token = error.token().unwrap_or_default();
            match error.kind() {
                ErrorKind::MissingValue if token == "-z" || EQ_ONLY.contains(&token) => {
                    fatal!("unknown command line option: {token}")
                }
                ErrorKind::MissingValue => fatal!("option {token}: argument missing"),
                ErrorKind::UnexpectedValue => fatal!(
                    "unknown command line option: {token}={}",
                    error.value().unwrap_or_default()
                ),
                _ => fatal!("{}", error.message(winnow_args::help::Style::PLAIN)),
            }
        }
    }
}

@ZOPT@

/// An option's value as UTF-8, as the options that read text need it.
pub(crate) fn utf8_arg<'a>(value: &'a OsStr, opt: &str) -> &'a str {
    value.to_str().unwrap_or_else(|| fatal!("option {opt}: expected a UTF-8 argument"))
}
