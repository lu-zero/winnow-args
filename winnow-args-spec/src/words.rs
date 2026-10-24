//! The words of an example's line, by what each is, read as
//! `winnow_args::help` reads them to paint the line.

use crate::{Item, Page};

/// What a piece of an example's line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Word {
    /// The first word of a command: the program, or what follows an operator.
    Program,
    /// A subcommand, on the page's path or under its command.
    Command,
    /// A flag of the command, without an attached `=value`; or `--`.
    Flag,
    /// `<NAME>` or `[NAME]`, with or without `...`.
    Placeholder,
    /// A shell operator: `|`, `&&`, `;`, `>`, ….
    Operator,
    /// `# comment`, to the end of the line.
    Comment,
    /// A value, a positional, an attached `=value`, or the space between words.
    Plain,
}

/// `line` cut into pieces, in order, that join back into it.
///
/// The program comes first, then the subcommands of `page`'s path from its
/// second word on, then those of its command. On a `multicall` program's
/// command's page, a line may start with the program's name instead, and
/// [`Page::full_path`]'s subcommands follow it. A flag is one the command or
/// its parents' globals spell; a flag that takes a value makes the next word,
/// or the next `values`, plain. After `--`, or a word that is neither,
/// nothing is a subcommand; after `--` nothing is a flag either. A quoted or
/// escaped space does not end a word.
pub fn example_words<'a>(line: &'a str, page: &Page<'_>) -> Vec<(Word, &'a str)> {
    let mut out = Vec::new();
    let mut command = Some(page.command);
    let mut ahead = page.path.get(1..).unwrap_or_default();
    let (mut program, mut flags, mut values) = (true, true, 0usize);
    let mut first = true;
    let mut at = 0;
    for (start, end) in shell_words(line) {
        if at < start {
            out.push((Word::Plain, &line[at..start]));
        }
        at = end;
        let word = &line[start..end];
        let items = || {
            let own = command.map_or(&[][..], |c| &c.items[..]).iter();
            own.chain(page.globals.iter().copied())
                .filter(|i| !i.positional)
        };
        if word.starts_with('#') {
            out.push((Word::Comment, &line[start..]));
            return out;
        }
        if word.bytes().all(|b| b"|&;<>()".contains(&b)) {
            out.push((Word::Operator, word));
            (command, program, flags, values) = (None, true, true, 0);
            continue;
        }
        let bare = word.strip_suffix("...").unwrap_or(word);
        let placeholder = (bare.starts_with('<') && bare.ends_with('>'))
            || (bare.starts_with('[') && bare.ends_with(']'));
        if placeholder && bare.len() > 2 {
            out.push((Word::Placeholder, word));
            values = values.saturating_sub(1);
            continue;
        }
        if program {
            out.push((Word::Program, word));
            // `busybox ls -l` on `ls`'s page.
            if first && page.full_path.first().is_some_and(|name| name == word) {
                ahead = &page.full_path[1..];
            }
            (program, first) = (false, false);
            continue;
        }
        if values > 0 {
            out.push((Word::Plain, word));
            values -= 1;
            continue;
        }
        if flags && word == "--" {
            out.push((Word::Flag, word));
            (command, flags) = (None, false);
            continue;
        }
        if flags && let Some(taken) = flag_word(word, items()) {
            let name = word.find('=').map_or(word, |eq| &word[..eq]);
            out.push((Word::Flag, name));
            if name.len() < word.len() {
                out.push((Word::Plain, &word[name.len()..]));
            }
            values = taken;
            continue;
        }
        let sub = match ahead.split_first() {
            Some((next, rest)) => (next == word).then(|| {
                ahead = rest;
                None
            }),
            None => command
                .and_then(|c| c.subcommands.iter().find(|s| answers(s, word)))
                .map(|s| Some(&s.command)),
        };
        match sub {
            Some(next) => {
                out.push((Word::Command, word));
                if let Some(next) = next {
                    command = Some(next);
                }
            }
            None => {
                out.push((Word::Plain, word));
                (command, ahead) = (None, &[]);
            }
        }
    }
    if at < line.len() {
        out.push((Word::Plain, &line[at..]));
    }
    out
}

/// Whether `word` selects `sub`.
fn answers(sub: &crate::Sub, word: &str) -> bool {
    sub.name == word
        || sub.aliases.iter().any(|a| a == word)
        || sub.hidden_aliases.iter().any(|a| a == word)
}

/// The words of a shell line, as byte ranges: a space inside quotes or after
/// `\` is part of its word.
fn shell_words(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut words = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at].is_ascii_whitespace() {
            at += 1;
            continue;
        }
        let start = at;
        let mut quote = None;
        while at < bytes.len() {
            let byte = bytes[at];
            match quote {
                Some(open) if byte == open => quote = None,
                Some(b'"' | b'`') if byte == b'\\' => at += 1,
                Some(_) => {}
                None if byte.is_ascii_whitespace() => break,
                None if byte == b'\\' => at += 1,
                None if matches!(byte, b'\'' | b'"' | b'`') => quote = Some(byte),
                None => {}
            }
            at += 1;
        }
        words.push((start, at.min(bytes.len())));
    }
    words
}

/// Whether `word` is a flag that one of `items` spells, and if so how many of
/// the next words are its values: none when it takes no value or has it
/// attached (`--jobs=4`, `-j4`). A negative number is not a flag.
fn flag_word<'a>(word: &str, mut items: impl Iterator<Item = &'a Item> + Clone) -> Option<usize> {
    let takes = |item: &Item| {
        if item.value_name.is_some() && !item.optional_value && !item.require_equals {
            item.values.max(1)
        } else {
            0
        }
    };
    if let Some(long) = word.strip_prefix("--") {
        let (name, attached) = long
            .split_once('=')
            .map_or((long, false), |(n, _)| (n, true));
        let item = items.find(|i| {
            i.long.as_deref() == Some(name)
                || i.aliases.iter().any(|a| a == name)
                || i.visible_aliases.iter().any(|a| a == name)
                || i.negate.as_deref() == Some(name)
        })?;
        let negated = item.negate.as_deref() == Some(name);
        return Some(if attached || negated { 0 } else { takes(item) });
    }
    let (sign, shorts) = word.split_at(word.len().min(1));
    if shorts.is_empty() || shorts.parse::<f64>().is_ok() {
        return None;
    }
    for (i, c) in shorts.char_indices() {
        let item = items.clone().find(|item| match sign {
            "-" => item.short == Some(c) || item.more_shorts.contains(&c),
            "+" => item.plus == Some(c),
            _ => false,
        })?;
        let taken = takes(item);
        if taken > 0 {
            return Some(if i + c.len_utf8() == shorts.len() {
                taken
            } else {
                0
            });
        }
    }
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::{Word, example_words};
    use crate::{Command, Item, Page, Sub};

    #[test]
    fn each_word_is_what_the_command_takes() {
        let run = Command {
            name: "run".to_owned(),
            ..Command::default()
        };
        let command = Command {
            name: "t".to_owned(),
            items: vec![
                Item {
                    short: Some('v'),
                    ..Item::default()
                },
                Item {
                    short: Some('j'),
                    value_name: Some("N".to_owned()),
                    values: 1,
                    ..Item::default()
                },
                Item {
                    long: Some("cd".to_owned()),
                    value_name: Some("DIR".to_owned()),
                    values: 1,
                    ..Item::default()
                },
            ],
            subcommands: vec![Sub {
                name: "run".to_owned(),
                command: run,
                ..Sub::default()
            }],
            ..Command::default()
        };
        let page = Page {
            path: vec!["t".to_owned()],
            full_path: Vec::new(),
            command: &command,
            globals: Vec::new(),
            deprecated: None,
        };
        let line = "t -vj 2 --cd=src run <FILE>... -x -- 'a b' | wc # count";
        let words: Vec<_> = example_words(line, &page)
            .into_iter()
            .filter(|(_, text)| !text.trim().is_empty())
            .collect();
        assert_eq!(
            words,
            [
                (Word::Program, "t"),
                (Word::Flag, "-vj"),
                (Word::Plain, "2"),
                (Word::Flag, "--cd"),
                (Word::Plain, "=src"),
                (Word::Command, "run"),
                (Word::Placeholder, "<FILE>..."),
                (Word::Plain, "-x"),
                (Word::Flag, "--"),
                (Word::Plain, "'a b'"),
                (Word::Operator, "|"),
                (Word::Program, "wc"),
                (Word::Comment, "# count"),
            ]
        );
        let all: String = example_words(line, &page).iter().map(|(_, t)| *t).collect();
        assert_eq!(all, line);
    }

    #[test]
    fn a_multicall_command_s_page_reads_either_name() {
        let ls = Command {
            items: vec![Item {
                short: Some('l'),
                ..Item::default()
            }],
            ..Command::default()
        };
        let page = Page {
            path: vec!["ls".to_owned()],
            full_path: vec!["busybox".to_owned(), "ls".to_owned()],
            command: &ls,
            globals: Vec::new(),
            deprecated: None,
        };
        let kinds = |line| -> Vec<Word> {
            let words = example_words(line, &page).into_iter();
            words
                .filter(|(_, t)| !t.trim().is_empty())
                .map(|(w, _)| w)
                .collect()
        };
        assert_eq!(kinds("ls -l"), [Word::Program, Word::Flag]);
        let full = [Word::Program, Word::Command, Word::Flag];
        assert_eq!(kinds("busybox ls -l"), full);
    }
}
