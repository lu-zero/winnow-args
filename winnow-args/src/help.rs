//! Help text: `static` data the derive emits, rendered only when asked for.
//!
//! Nothing here runs during a successful parse. A command's [`Command`] sits in
//! read-only data until `--help` (or `-h`) returns it inside an
//! [`ErrorKind::HelpRequested`](crate::ErrorKind::HelpRequested), and
//! [`render`] turns it into text.

use std::fmt::Write as _;

use crate::color::Paint;
pub use crate::color::{Depth, Palette, Theme};

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

/// The width help wraps to: `COLUMNS` when it is a positive number, so a user
/// or a test can say what to assume; else, with the `terminal-size` feature,
/// the width of the terminal on standard output; else 100 (clap's width when
/// it cannot ask the terminal).
///
/// Under [`with_env`](crate::with_env) the terminal is not asked: the page
/// depends only on the environment given.
pub fn width() -> usize {
    crate::env::var("COLUMNS")
        .and_then(|columns| columns.to_str()?.trim().parse().ok())
        .filter(|&columns| columns > 0)
        .or_else(terminal_width)
        .unwrap_or(100)
}

#[cfg(feature = "terminal-size")]
fn terminal_width() -> Option<usize> {
    if crate::env::overridden() {
        return None;
    }
    terminal_size::terminal_size()
        .map(|(width, _)| usize::from(width.0))
        .filter(|&width| width > 0)
}

#[cfg(not(feature = "terminal-size"))]
fn terminal_width() -> Option<usize> {
    None
}

/// How help and errors are painted: a [`Palette`] for a terminal of some
/// [`Depth`]. The escapes never count toward a column's width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// The paint for each kind of text.
    pub palette: Palette,
    /// What the terminal shows; colors deeper than this are mapped down.
    pub depth: Depth,
}

/// A paint resolved for a depth: what the renderer writes with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ink {
    paint: Paint,
    depth: Depth,
}

impl Ink {
    /// Plain text.
    const NONE: Ink = Ink {
        paint: Paint::NONE,
        depth: Depth::None,
    };
}

impl Style {
    /// Nothing painted: for pipes, files and tests.
    pub const PLAIN: Style = Style::at(Palette::PLAIN, Depth::None);

    /// [`Palette::DEFAULT`] in the 16 basic colors.
    pub const COLORED: Style = Style::at(Palette::DEFAULT, Depth::Ansi16);

    /// [`Palette::CLAP`], clap 4's colors.
    pub const CLAP: Style = Style::at(Palette::CLAP, Depth::Ansi16);

    /// `palette` on a terminal of `depth`.
    pub const fn at(palette: Palette, depth: Depth) -> Style {
        Style { palette, depth }
    }

    /// [`Theme::DEFAULT`] for standard output, at the depth
    /// [`Depth::detect`] finds.
    pub fn auto() -> Style {
        use std::io::IsTerminal as _;
        Style::themed(&Theme::DEFAULT, std::io::stdout().is_terminal())
    }

    /// [`Style::auto`] for standard error, where errors go.
    pub fn auto_stderr() -> Style {
        use std::io::IsTerminal as _;
        Style::themed(&Theme::DEFAULT, std::io::stderr().is_terminal())
    }

    /// [`Theme::DEFAULT`] for a stream that is (or is not) a terminal.
    pub fn auto_for(is_terminal: bool) -> Style {
        Style::themed(&Theme::DEFAULT, is_terminal)
    }

    /// `theme`'s palette for the depth [`Depth::detect`] finds for a stream
    /// that is (or is not) a terminal.
    pub fn themed(theme: &Theme, is_terminal: bool) -> Style {
        let depth = Depth::detect(is_terminal);
        Style::at(theme.palette(depth), depth)
    }

    fn ink(self, paint: Paint) -> Ink {
        Ink {
            paint,
            depth: self.depth,
        }
    }

    pub(crate) fn header(self) -> Ink {
        self.ink(self.palette.header)
    }

    pub(crate) fn program(self) -> Ink {
        self.ink(self.palette.program)
    }

    pub(crate) fn literal(self) -> Ink {
        self.ink(self.palette.literal)
    }

    pub(crate) fn placeholder(self) -> Ink {
        self.ink(self.palette.placeholder)
    }

    pub(crate) fn error(self) -> Ink {
        self.ink(self.palette.error)
    }

    pub(crate) fn invalid(self) -> Ink {
        self.ink(self.palette.invalid)
    }

    pub(crate) fn valid(self) -> Ink {
        self.ink(self.palette.valid)
    }

    pub(crate) fn value(self) -> Ink {
        self.ink(self.palette.value)
    }

    /// `text` in `ink`, reset after.
    pub(crate) fn paint(ink: Ink, text: &str) -> String {
        let mut out = String::new();
        push_painted(&mut out, ink, text);
        out
    }
}

/// Append `text` in `ink` (plain if it paints nothing), reset after.
fn push_painted(out: &mut String, ink: Ink, text: &str) {
    ink.paint.write(out, ink.depth, text);
}

/// Painted text and how many columns it takes on screen.
#[derive(Default)]
struct Cell {
    text: String,
    width: usize,
}

impl Cell {
    fn push(&mut self, ink: Ink, text: &str) {
        push_painted(&mut self.text, ink, text);
        self.width += text.chars().count();
    }

    /// `<NAME>` painted whole, or `[NAME]` with only the name painted, as usage
    /// does: square brackets say "optional", they are not part of the value.
    fn push_placeholder(&mut self, style: Style, required: bool, name: &str) {
        if required {
            self.push(style.placeholder(), &format!("<{name}>"));
        } else {
            self.push(Ink::NONE, "[");
            self.push(style.placeholder(), name);
            self.push(Ink::NONE, "]");
        }
    }
}

/// Render `command`'s help, plain, wrapped to [`width`]. `path` is the command
/// line leading to it (program name, then subcommand names); `long` selects
/// `--help` over `-h`.
pub fn render(command: &Command, path: &[&str], long: bool) -> String {
    render_styled(command, path, long, width(), Style::PLAIN)
}

/// [`render`], wrapped to `width` columns.
pub fn render_width(command: &Command, path: &[&str], long: bool, width: usize) -> String {
    render_styled(command, path, long, width, Style::PLAIN)
}

/// [`render`], wrapped to `width` columns and painted with `style`.
pub fn render_styled(
    command: &Command,
    path: &[&str],
    long: bool,
    width: usize,
    style: Style,
) -> String {
    let mut out = String::new();
    let about = if long && !command.long_about.is_empty() {
        command.long_about
    } else {
        command.about
    };
    if !about.is_empty() {
        let _ = writeln!(out, "{}\n", wrap_text(about, width));
    }
    push_painted(&mut out, style.header(), "Usage:");
    let _ = writeln!(out, " {}\n", usage(command, path, style));

    let visible_subs: Vec<&Sub> = command.subcommands.iter().filter(|s| !s.hide).collect();
    if !visible_subs.is_empty() {
        let rows: Vec<(Cell, String)> = visible_subs
            .iter()
            .map(|s| {
                let mut name = Cell::default();
                name.push(style.literal(), s.name);
                for alias in s.aliases {
                    name.push(Ink::NONE, ", ");
                    name.push(style.literal(), alias);
                }
                (name, s.about.to_owned())
            })
            .collect();
        section(&mut out, "Commands", &rows, false, width, style);
    }

    let visible: Vec<&Item> = command.items.iter().filter(|i| !i.hide).collect();
    let arguments: Vec<(Cell, String)> = visible
        .iter()
        .filter(|i| i.positional)
        .map(|i| {
            let mut cell = Cell::default();
            push_item_placeholder(&mut cell, style, i);
            (cell, describe(i, long, style))
        })
        .collect();
    section(&mut out, "Arguments", &arguments, long, width, style);

    let mut headings: Vec<Option<&str>> = vec![None];
    for item in visible.iter().filter(|i| !i.positional) {
        if !headings.contains(&item.heading) {
            headings.push(item.heading);
        }
    }
    let builtin = |short: &str, long_name: &str| {
        let mut cell = Cell::default();
        cell.push(style.literal(), short);
        cell.push(Ink::NONE, ", ");
        cell.push(style.literal(), long_name);
        cell
    };
    for heading in headings {
        let mut rows: Vec<(Cell, String)> = visible
            .iter()
            .filter(|i| !i.positional && i.heading == heading)
            .map(|i| (flag_spec(i, style), describe(i, long, style)))
            .collect();
        if heading.is_none() {
            if command.help_flag {
                let what = if long {
                    "Print help (see a summary with '-h')"
                } else {
                    "Print help (see more with '--help')"
                };
                rows.push((builtin("-h", "--help"), what.into()));
            }
            if command.version.is_some() {
                rows.push((builtin("-V", "--version"), "Print version".into()));
            }
        }
        section(
            &mut out,
            heading.unwrap_or("Options"),
            &rows,
            long,
            width,
            style,
        );
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

/// `prog [OPTIONS] <A> [B]... [COMMAND]`.
pub(crate) fn usage(command: &Command, path: &[&str], style: Style) -> String {
    let mut line = Style::paint(style.program(), &path.join(" "));
    let mut cell = Cell::default();
    if command.items.iter().any(|i| !i.positional && !i.hide) || command.help_flag {
        cell.push(Ink::NONE, " ");
        cell.push_placeholder(style, false, "OPTIONS");
    }
    for item in command
        .items
        .iter()
        .filter(|i| i.positional && !i.hide && !i.trailing)
    {
        cell.push(Ink::NONE, " ");
        push_item_placeholder(&mut cell, style, item);
    }
    if !command.subcommands.is_empty() {
        cell.push(Ink::NONE, " ");
        cell.push_placeholder(style, command.subcommand_required, "COMMAND");
    }
    line.push_str(&cell.text);
    for item in command
        .items
        .iter()
        .filter(|i| i.positional && !i.hide && i.trailing)
    {
        // clap paints the brackets and `--` as literals: they are typed.
        let mut cell = Cell::default();
        cell.push(Ink::NONE, " ");
        cell.push(style.literal(), "[--");
        cell.push(Ink::NONE, " ");
        push_item_placeholder(&mut cell, style, item);
        cell.push(style.literal(), "]");
        line.push_str(&cell.text);
    }
    line
}

/// A positional's placeholder: `<NAME>` or `[NAME]`, then `...` if repeated.
fn push_item_placeholder(cell: &mut Cell, style: Style, item: &Item) {
    cell.push_placeholder(style, item.required, item.value_name.unwrap_or("VALUE"));
    if item.multiple {
        cell.push(Ink::NONE, "...");
    }
}

fn flag_spec(item: &Item, style: Style) -> Cell {
    let mut spec = Cell::default();
    match (item.short, item.long) {
        (Some(s), long) => {
            spec.push(style.literal(), &format!("-{s}"));
            if let Some(l) = long {
                spec.push(Ink::NONE, ", ");
                spec.push(style.literal(), &format!("--{l}"));
            }
        }
        (None, Some(l)) => {
            spec.push(Ink::NONE, "    ");
            spec.push(style.literal(), &format!("--{l}"));
        }
        (None, None) => {}
    }
    if let Some(no) = item.negate {
        spec.push(Ink::NONE, " / ");
        spec.push(style.literal(), &format!("--{no}"));
    }
    if let Some(value) = item.value_name {
        spec.push(Ink::NONE, " ");
        spec.push(style.placeholder(), &format!("<{value}>"));
    }
    if item.multiple {
        spec.push(Ink::NONE, "...");
    }
    spec
}

/// An item's description, then `[env: …]`, `[default: …]` and
/// `[possible values: …]`, each value painted on its own (so a line never
/// breaks inside a painted span).
fn describe(item: &Item, long: bool, style: Style) -> String {
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
    let value = |text: &str| Style::paint(style.value(), text);
    if let Some(default) = item.default {
        add(format!("[default: {}]", value(default)));
    }
    if !item.choices.is_empty() {
        let choices: Vec<String> = item.choices.iter().map(|c| value(c)).collect();
        add(format!("[possible values: {}]", choices.join(", ")));
    }
    text
}

/// A titled two-column list; long help puts each description under its item.
///
/// Descriptions wrap to `width`. The column is at most two fifths of the
/// page (usage's rule), so one long spelling does not squeeze every
/// description; an item wider than that has its description on the next line.
fn section(
    out: &mut String,
    title: &str,
    rows: &[(Cell, String)],
    long: bool,
    width: usize,
    style: Style,
) {
    if rows.is_empty() {
        return;
    }
    push_painted(out, style.header(), &format!("{title}:"));
    out.push('\n');
    let longest = rows.iter().map(|(left, _)| left.width).max().unwrap_or(0);
    let available = width.saturating_sub(4);
    let column = longest.min(available * 2 / 5);
    // Where descriptions start, and how much room they get (never too little to read).
    let (indent, room) = if long {
        (10, width.saturating_sub(10).max(20))
    } else {
        (column + 4, width.saturating_sub(column + 4).max(20))
    };
    for (left, right) in rows {
        let beside = !long && !right.is_empty() && left.width <= column;
        out.push_str("  ");
        out.push_str(&left.text);
        if beside {
            // Padded by what shows, not by the escapes' bytes.
            let _ = write!(out, "{:pad$}  ", "", pad = column - left.width);
        } else {
            out.push('\n');
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
        let mut escape = false;
        for (at, c) in self.rest.char_indices() {
            // SGR sequences (`ESC [ … m`) take no columns.
            if escape {
                escape = c != 'm';
                continue;
            }
            if c == '\u{1b}' {
                escape = true;
                continue;
            }
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
    fn escapes_take_no_columns() {
        let green = |s: &str| format!("\u{1b}[32m{s}\u{1b}[0m");
        let line = format!("[possible values: {}, {}]", green("auto"), green("never"));
        let pieces = wrap(&line, 24);
        assert_eq!(
            pieces,
            [
                format!("[possible values: {},", green("auto")),
                format!("{}]", green("never"))
            ]
        );
    }

    #[test]
    fn width_counts_characters() {
        assert_eq!(wrap("éé éé éé", 5), ["éé éé", "éé"]);
    }
}
