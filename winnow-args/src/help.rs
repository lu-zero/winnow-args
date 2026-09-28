//! Help text: `static` data the derive emits, rendered only when asked for.
//!
//! Nothing here runs during a successful parse. A command's [`Command`] sits in
//! read-only data until `--help` (or `-h`) returns it inside an
//! [`ErrorKind::HelpRequested`](crate::ErrorKind::HelpRequested), and
//! [`render`] turns it into text.

use std::fmt::Write as _;

/// One command's help: what it is, what it takes, what it contains.
#[derive(Debug)]
pub struct Command {
    /// The command's own name; empty for a root that leaves it to `argv[0]`.
    pub name: &'static str,
    /// First paragraph of the description.
    pub about: &'static str,
    /// The whole description, for `--help`.
    pub long_about: &'static str,
    /// Text after the lists, for `-h`.
    pub after_help: &'static str,
    /// Text after the lists, for `--help`.
    pub after_long_help: &'static str,
    /// Flags and positionals, in declaration order.
    pub items: &'static [Item],
    /// Subcommands, in declaration order.
    pub subcommands: &'static [Sub],
    /// Whether a subcommand must be given.
    pub subcommand_required: bool,
    /// Whether `-h`/`--help` are supplied (not declared or disabled).
    pub help_flag: bool,
    /// The version, when `-V`/`--version` are supplied.
    pub version: Option<&'static str>,
}

/// A flag or positional.
#[derive(Debug)]
pub struct Item {
    /// `-c`.
    pub short: Option<char>,
    /// `--name`.
    pub long: Option<&'static str>,
    /// `--no-name`, the spelling that sets it false.
    pub negate: Option<&'static str>,
    /// The value's placeholder; `None` for a switch.
    pub value_name: Option<&'static str>,
    /// First paragraph of the description.
    pub help: &'static str,
    /// The whole description.
    pub long_help: &'static str,
    /// Section to list it under instead of "Options".
    pub heading: Option<&'static str>,
    /// Left out of help.
    pub hide: bool,
    /// A positional rather than a flag.
    pub positional: bool,
    /// Must be given.
    pub required: bool,
    /// Takes several values or occurrences.
    pub multiple: bool,
    /// A positional that only takes words after `--`.
    pub trailing: bool,
    /// The value used when none is given.
    pub default: Option<&'static str>,
    /// The environment variable consulted.
    pub env: Option<&'static str>,
    /// The only values accepted.
    pub choices: &'static [&'static str],
}

/// A subcommand as its parent lists it.
#[derive(Debug)]
pub struct Sub {
    /// Its name.
    pub name: &'static str,
    /// Other spellings shown beside it.
    pub aliases: &'static [&'static str],
    /// Every spelling it answers to, hidden aliases included, for `help <name>`.
    pub names: &'static [&'static str],
    /// Its own help.
    pub command: &'static Command,
    /// One line about it.
    pub about: &'static str,
    /// Left out of the list.
    pub hide: bool,
}

/// The command a `help a b …` line asks about: follow each word through
/// `root`'s subcommands until one names none. Gives it and the names followed.
pub fn resolve<'w>(
    root: &'static Command,
    words: impl IntoIterator<Item = &'w [u8]>,
) -> (&'static Command, Vec<&'static str>) {
    let (mut command, mut path) = (root, Vec::new());
    for word in words {
        let Some(sub) = command
            .subcommands
            .iter()
            .find(|s| s.names.iter().any(|n| n.as_bytes() == word))
        else {
            break;
        };
        path.push(sub.name);
        command = sub.command;
    }
    (command, path)
}

/// The width help wraps to: `COLUMNS` when it is a positive number, else 100
/// (clap's width when it cannot ask the terminal).
pub fn width() -> usize {
    crate::env::var("COLUMNS")
        .and_then(|columns| columns.to_str()?.trim().parse().ok())
        .filter(|&columns| columns > 0)
        .unwrap_or(100)
}

/// Render `command`'s help, wrapped to [`width`]. `path` is the command line
/// leading to it (program name, then subcommand names); `long` selects
/// `--help` over `-h`.
pub fn render(command: &Command, path: &[&str], long: bool) -> String {
    render_width(command, path, long, width())
}

/// [`render`], wrapped to `width` columns.
pub fn render_width(command: &Command, path: &[&str], long: bool, width: usize) -> String {
    let mut out = String::new();
    let about = if long && !command.long_about.is_empty() {
        command.long_about
    } else {
        command.about
    };
    if !about.is_empty() {
        let _ = writeln!(out, "{}\n", wrap_text(about, width));
    }
    let _ = writeln!(out, "Usage: {}\n", usage(command, path));

    let visible_subs: Vec<&Sub> = command.subcommands.iter().filter(|s| !s.hide).collect();
    if !visible_subs.is_empty() {
        let rows: Vec<(String, String)> = visible_subs
            .iter()
            .map(|s| {
                let name = if s.aliases.is_empty() {
                    s.name.to_owned()
                } else {
                    format!("{}, {}", s.name, s.aliases.join(", "))
                };
                (name, s.about.to_owned())
            })
            .collect();
        section(&mut out, "Commands", &rows, false, width);
    }

    let visible: Vec<&Item> = command.items.iter().filter(|i| !i.hide).collect();
    let arguments: Vec<(String, String)> = visible
        .iter()
        .filter(|i| i.positional)
        .map(|i| (placeholder(i), describe(i, long)))
        .collect();
    section(&mut out, "Arguments", &arguments, long, width);

    let mut headings: Vec<Option<&str>> = vec![None];
    for item in visible.iter().filter(|i| !i.positional) {
        if !headings.contains(&item.heading) {
            headings.push(item.heading);
        }
    }
    for heading in headings {
        let mut rows: Vec<(String, String)> = visible
            .iter()
            .filter(|i| !i.positional && i.heading == heading)
            .map(|i| (flag_spec(i), describe(i, long)))
            .collect();
        if heading.is_none() {
            if command.help_flag {
                let what = if long {
                    "Print help (see a summary with '-h')"
                } else {
                    "Print help (see more with '--help')"
                };
                rows.push(("-h, --help".into(), what.into()));
            }
            if command.version.is_some() {
                rows.push(("-V, --version".into(), "Print version".into()));
            }
        }
        section(&mut out, heading.unwrap_or("Options"), &rows, long, width);
    }

    let after = if long && !command.after_long_help.is_empty() {
        command.after_long_help
    } else {
        command.after_help
    };
    if !after.is_empty() {
        let _ = writeln!(out, "{}", wrap_text(after, width));
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}

/// `Usage: prog [OPTIONS] <A> [B]... [COMMAND]`.
fn usage(command: &Command, path: &[&str]) -> String {
    let mut line = path.join(" ");
    if command.items.iter().any(|i| !i.positional && !i.hide) || command.help_flag {
        line.push_str(" [OPTIONS]");
    }
    for item in command
        .items
        .iter()
        .filter(|i| i.positional && !i.hide && !i.trailing)
    {
        let _ = write!(line, " {}", placeholder(item));
    }
    if !command.subcommands.is_empty() {
        line.push_str(if command.subcommand_required {
            " <COMMAND>"
        } else {
            " [COMMAND]"
        });
    }
    for item in command
        .items
        .iter()
        .filter(|i| i.positional && !i.hide && i.trailing)
    {
        let _ = write!(line, " [-- {}]", placeholder(item));
    }
    line
}

fn placeholder(item: &Item) -> String {
    let name = item.value_name.unwrap_or("VALUE");
    let dots = if item.multiple { "..." } else { "" };
    if item.required {
        format!("<{name}>{dots}")
    } else {
        format!("[{name}]{dots}")
    }
}

fn flag_spec(item: &Item) -> String {
    let mut spec = match (item.short, item.long) {
        (Some(s), Some(l)) => format!("-{s}, --{l}"),
        (Some(s), None) => format!("-{s}"),
        (None, Some(l)) => format!("    --{l}"),
        (None, None) => String::new(),
    };
    if let Some(no) = item.negate {
        let _ = write!(spec, " / --{no}");
    }
    if let Some(value) = item.value_name {
        let _ = write!(spec, " <{value}>");
    }
    if item.multiple {
        spec.push_str("...");
    }
    spec
}

fn describe(item: &Item, long: bool) -> String {
    let mut text = if long && !item.long_help.is_empty() {
        item.long_help.to_owned()
    } else {
        item.help.to_owned()
    };
    let mut add = |extra: String| {
        if !text.is_empty() {
            text.push(if long { '\n' } else { ' ' });
        }
        text.push_str(&extra);
    };
    if let Some(env) = item.env {
        add(format!("[env: {env}]"));
    }
    if let Some(default) = item.default {
        add(format!("[default: {default}]"));
    }
    if !item.choices.is_empty() {
        add(format!("[possible values: {}]", item.choices.join(", ")));
    }
    text
}

/// A titled two-column list; long help puts each description under its item.
///
/// Descriptions wrap to `width`. The column is at most two fifths of the
/// page (usage's rule), so one long spelling does not squeeze every
/// description; an item wider than that has its description on the next line.
fn section(out: &mut String, title: &str, rows: &[(String, String)], long: bool, width: usize) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(out, "{title}:");
    let longest = rows
        .iter()
        .map(|(left, _)| left.chars().count())
        .max()
        .unwrap_or(0);
    let available = width.saturating_sub(4);
    let column = longest.min(available * 2 / 5);
    // Where descriptions start, and how much room they get (never too little to read).
    let (indent, room) = if long {
        (10, width.saturating_sub(10).max(20))
    } else {
        (column + 4, width.saturating_sub(column + 4).max(20))
    };
    for (left, right) in rows {
        let beside = !long && !right.is_empty() && left.chars().count() <= column;
        if beside {
            let _ = write!(out, "  {left:column$}  ");
        } else {
            let _ = writeln!(out, "  {left}");
        }
        let mut first = beside;
        for line in right.lines() {
            if line.is_empty() {
                out.push('\n');
                continue;
            }
            for piece in Wrap::new(line, room) {
                if !first {
                    let _ = write!(out, "{:indent$}", "");
                }
                first = false;
                out.push_str(piece);
                out.push('\n');
            }
        }
        if long {
            // A blank line between items.
            out.push('\n');
        }
    }
    if !long {
        out.push('\n');
    }
}

/// Each line of `text` wrapped to `width`, blank lines kept.
fn wrap_text(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / width.max(1));
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for (j, piece) in Wrap::new(line, width.max(20)).enumerate() {
            if j > 0 {
                out.push('\n');
            }
            out.push_str(piece);
        }
    }
    out
}

/// The pieces of `line` broken at spaces into at most `width` characters; a
/// word longer than that stands alone. Leading indentation stays on the first
/// piece. Borrowed slices, so a line that fits costs one scan.
struct Wrap<'a> {
    rest: &'a str,
    width: usize,
}

impl<'a> Wrap<'a> {
    fn new(line: &'a str, width: usize) -> Self {
        Wrap {
            rest: line.trim_end(),
            width,
        }
    }
}

impl<'a> Iterator for Wrap<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }
        // The byte after the last character that fits, and the last space before it.
        let (mut end, mut space, mut count, mut text) = (self.rest.len(), None, 0, false);
        for (at, c) in self.rest.char_indices() {
            if count == self.width {
                end = at;
                break;
            }
            // Spaces before the first word are indentation, not a place to break.
            if c != ' ' {
                text = true;
            } else if text {
                space = Some(at);
            }
            count += 1;
        }
        let cut = match (end == self.rest.len(), space) {
            (true, _) => end,
            // The next character is a space: break right there.
            (false, _) if self.rest[end..].starts_with(' ') => end,
            (false, Some(at)) => at,
            // One long word: up to its end.
            (false, None) => self.rest[end..]
                .find(' ')
                .map_or(self.rest.len(), |at| end + at),
        };
        let piece = self.rest[..cut].trim_end();
        self.rest = self.rest[cut..].trim_start();
        Some(piece)
    }
}

#[cfg(test)]
mod tests {
    use super::Wrap;

    fn wrap(line: &str, width: usize) -> Vec<&str> {
        Wrap::new(line, width).collect()
    }

    #[test]
    fn pieces_break_at_spaces_within_the_width() {
        assert_eq!(wrap("aa bb cc", 5), ["aa bb", "cc"]);
        assert_eq!(wrap("aa bb", 5), ["aa bb"]);
        assert_eq!(wrap("aa  bb   cc", 6), ["aa  bb", "cc"]);
    }

    #[test]
    fn a_long_word_stands_alone() {
        assert_eq!(wrap("a verylongword b", 5), ["a", "verylongword", "b"]);
    }

    #[test]
    fn indentation_stays_on_the_first_piece() {
        assert_eq!(wrap("    $ mise use node", 12), ["    $ mise", "use node"]);
    }

    #[test]
    fn width_counts_characters() {
        assert_eq!(wrap("éé éé éé", 5), ["éé éé", "éé"]);
    }
}
