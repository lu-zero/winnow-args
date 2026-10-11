//! What a word that nothing accepts was probably meant to be.
//!
//! Only an error reaches this: a parse that succeeds builds none of it.

use crate::help::{Command, Item};

/// The command the names of `path` lead to from `root`, and the global flags
/// of the commands above it.
fn reached(root: &'static Command, path: &[&str]) -> (&'static Command, Vec<&'static Item>) {
    let (mut command, mut globals) = (root, Vec::new());
    for name in path {
        let Some(sub) = command.subcommands.iter().find(|sub| sub.name == *name) else {
            break;
        };
        globals.extend(command.items.iter().filter(|item| item.global));
        command = sub.command;
    }
    (command, globals)
}

/// The long flag closest to `typed`, `--name` or `--name=value`, among those
/// the command at `path` shows.
pub(super) fn flag(root: &'static Command, path: &[&str], typed: &str) -> Option<String> {
    let typed = typed.strip_prefix("--")?;
    let typed = typed.split_once('=').map_or(typed, |(name, _)| name);
    let (command, globals) = reached(root, path);
    let own = command.items.iter().filter(|item| !item.positional);
    let names = own
        .chain(globals)
        .filter(|item| !item.hide)
        .flat_map(|item| [item.long, item.negate])
        .flatten()
        .chain(command.help_flag.then_some("help"))
        .chain(command.version.map(|_| "version"));
    nearest(typed, names).map(|name| format!("--{name}"))
}

/// The subcommand closest to `typed` among those the command at `path` shows.
pub(super) fn subcommand(
    root: &'static Command,
    path: &[&str],
    typed: &str,
) -> Option<&'static str> {
    let (command, _) = reached(root, path);
    let shown = command.subcommands.iter().filter(|sub| !sub.hide);
    nearest(
        typed,
        shown.flat_map(|sub| [sub.name].into_iter().chain(sub.aliases.iter().copied())),
    )
}

/// The one of `names` most like `typed`, when one is like enough: the first
/// of the closest. The measure and the bar are clap's, Jaro at 0.7.
pub(super) fn nearest<'a>(typed: &str, names: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut best = (0.7, None);
    for name in names {
        let likeness = jaro(typed, name);
        if likeness > best.0 {
            best = (likeness, Some(name));
        }
    }
    best.1
}

/// The Jaro similarity of two words: 1 for the same word, 0 for words with
/// nothing in common.
fn jaro(a: &str, b: &str) -> f64 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.is_empty() || b.is_empty() {
        return f64::from(u8::from(a == b));
    }
    // A letter matches the same letter no further away than this.
    let reach = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut taken = vec![false; b.len()];
    let mut matched = Vec::new();
    for (at, letter) in a.iter().enumerate() {
        let from = at.saturating_sub(reach);
        let to = (at + reach + 1).min(b.len());
        if let Some(found) = (from..to).find(|&i| !taken[i] && b[i] == *letter) {
            taken[found] = true;
            matched.push(*letter);
        }
    }
    if matched.is_empty() {
        return 0.0;
    }
    let theirs = b.iter().zip(&taken).filter(|(_, taken)| **taken);
    let moved = matched
        .iter()
        .zip(theirs)
        .filter(|(a, (b, _))| a != b)
        .count();
    let matches = matched.len() as f64;
    (matches / a.len() as f64 + matches / b.len() as f64 + (matches - moved as f64 / 2.0) / matches)
        / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_closest_name_is_found_when_one_is_close() {
        let names = ["force", "format", "verbose"];
        assert_eq!(nearest("fore", names.into_iter()), Some("force"));
        assert_eq!(nearest("verbos", names.into_iter()), Some("verbose"));
        assert_eq!(nearest("fromat", names.into_iter()), Some("format"));
        assert_eq!(nearest("quiet", names.into_iter()), None);
        assert_eq!(nearest("", names.into_iter()), None);
        // The measure, on the pairs it is usually shown with.
        assert!((jaro("martha", "marhta") - 0.9444).abs() < 0.0001);
        assert!((jaro("dixon", "dicksonx") - 0.7667).abs() < 0.0001);
        assert_eq!(jaro("same", "same"), 1.0);
        assert_eq!(jaro("abc", "xyz"), 0.0);
    }
}
