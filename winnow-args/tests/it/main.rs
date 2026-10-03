//! The behaviour tests: one module per feature, in one binary.
#![cfg(feature = "derive")]

use winnow_args::BStr;

mod alias;
mod cfg;
mod choices;
mod compile_fail;
mod complete;
mod constraints;
mod count;
mod default_rules;
mod default_subcommand;
mod delimiter;
mod double_dash;
mod fallbacks;
mod flags;
mod flatten;
mod globals;
#[cfg(feature = "help-text")]
mod help;
#[cfg(feature = "help-text")]
mod help_text;
mod hyphen;
mod ignored;
mod interactions;
mod keep_equals;
mod keywords;
mod ld_values;
mod long_only;
mod negate;
mod negative;
#[cfg(not(feature = "help-text"))]
mod no_help_text;
mod occurrence;
mod optional_value;
mod plus_options;
mod positionals;
mod preserve;
mod repeat;
mod require_equals;
mod response;
mod sequence;
mod subcommand;
mod unknown_flags;
mod value_ranges;

/// A line's words, as a parser takes them.
fn words<'a>(line: &'a [&'a str]) -> Vec<&'a BStr> {
    line.iter().map(BStr::new).collect()
}
