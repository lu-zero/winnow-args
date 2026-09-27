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

/// Render `command`'s help. `path` is the command line leading to it (program
/// name, then subcommand names); `long` selects `--help` over `-h`.
pub fn render(command: &Command, path: &[&str], long: bool) -> String {
    let mut out = String::new();
    let about = if long && !command.long_about.is_empty() {
        command.long_about
    } else {
        command.about
    };
    if !about.is_empty() {
        let _ = writeln!(out, "{about}\n");
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
        section(&mut out, "Commands", &rows, false);
    }

    let visible: Vec<&Item> = command.items.iter().filter(|i| !i.hide).collect();
    let arguments: Vec<(String, String)> = visible
        .iter()
        .filter(|i| i.positional)
        .map(|i| (placeholder(i), describe(i, long)))
        .collect();
    section(&mut out, "Arguments", &arguments, long);

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
        section(&mut out, heading.unwrap_or("Options"), &rows, long);
    }

    let after = if long && !command.after_long_help.is_empty() {
        command.after_long_help
    } else {
        command.after_help
    };
    if !after.is_empty() {
        let _ = writeln!(out, "{after}");
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
fn section(out: &mut String, title: &str, rows: &[(String, String)], long: bool) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(out, "{title}:");
    let width = rows
        .iter()
        .map(|(left, _)| left.chars().count())
        .max()
        .unwrap_or(0);
    for (left, right) in rows {
        if long {
            // Each item, then its description indented under it, then a blank line.
            let _ = writeln!(out, "  {left}");
            for line in right.lines() {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    let _ = writeln!(out, "          {line}");
                }
            }
            out.push('\n');
        } else if right.is_empty() {
            let _ = writeln!(out, "  {left}");
        } else {
            let _ = writeln!(out, "  {left:width$}  {right}");
        }
    }
    if !long {
        out.push('\n');
    }
}
