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
///
/// Written by the derive, as [`Item`] and [`Sub`] are. Fields are added as
/// help learns to show more: a value written by hand ends in
/// `..Command::EMPTY`, and so keeps compiling when one is.
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
    /// Whether `--help` is supplied (not declared or disabled).
    pub help_flag: bool,
    /// Whether the supplied help flag has `-h` as well as `--help`.
    pub help_short: bool,
    /// The version, when `-V`/`--version` are supplied.
    pub version: Option<&'static str>,
    /// The subcommand a line that names none runs: marked `(default)` in
    /// the list.
    pub default_subcommand: Option<&'static str>,
    /// Whether a flag this command does not take selects the default
    /// subcommand too.
    pub default_subcommand_flags: bool,
    /// Whether the default subcommand's page follows this one.
    pub default_subcommand_help: bool,
    /// Whether each subcommand's usage line, description and arguments stand
    /// where the list of subcommands would.
    pub flatten_help: bool,
    /// Command lines to try, in an Examples section.
    pub examples: &'static [Example],
    /// What `--help` says under a heading, before its entries.
    pub headings: &'static [Heading],
}

impl Command {
    /// A command that takes nothing and says nothing. A value written by
    /// hand ends in `..Command::EMPTY`, and a release that adds a field does
    /// not break it.
    pub const EMPTY: Command = Command {
        name: "",
        about: "",
        long_about: "",
        after_help: "",
        after_long_help: "",
        items: &[],
        subcommands: &[],
        subcommand_required: false,
        help_flag: false,
        help_short: false,
        version: None,
        default_subcommand: None,
        default_subcommand_flags: false,
        default_subcommand_help: false,
        flatten_help: false,
        examples: &[],
        headings: &[],
    };

    /// The subcommand a line that names none runs.
    pub(crate) fn default_target(&self) -> Option<&'static Sub> {
        let name = self.default_subcommand?;
        let subs = self.subcommands;
        subs.iter()
            .find(|sub| sub.name == name)
            .or_else(|| subs.iter().find(|sub| sub.names.contains(&name)))
    }

    /// [`Command::default_target`], when help shows it.
    fn default_sub(&self) -> Option<&'static Sub> {
        self.default_target().filter(|sub| !sub.hide)
    }

    /// What is said under the heading `title`.
    fn heading_help(&self, title: &str) -> Option<&'static str> {
        let heading = self.headings.iter().find(|h| h.title == title)?;
        Some(heading.help).filter(|help| !help.is_empty())
    }
}

/// A command line to try.
#[derive(Debug, Clone, Copy)]
pub struct Example {
    /// The line, as typed: `prog build --release`.
    pub code: &'static str,
    /// What it is an example of, on a line of its own above; empty for none.
    pub header: &'static str,
    /// What it does, for `--help`; empty for none.
    pub help: &'static str,
}

/// What a section of the page says before its entries.
#[derive(Debug, Clone, Copy)]
pub struct Heading {
    /// The heading, as an item's [`Item::heading`] spells it.
    pub title: &'static str,
    /// The paragraph under it, for `--help`.
    pub help: &'static str,
}

/// A flag or positional.
#[derive(Debug, Clone, Copy)]
pub struct Item {
    /// `-c`.
    pub short: Option<char>,
    /// `--name`.
    pub long: Option<&'static str>,
    /// Other long names it answers to, not shown.
    pub aliases: &'static [&'static str],
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
    /// The value is only taken attached (`--name=value`), never the next word.
    pub require_equals: bool,
    /// Also accepted by every subcommand below.
    pub global: bool,
    /// Other letters it answers to: `-l`, `-L`.
    pub more_shorts: &'static [char],
    /// The letter of its `+c` spelling.
    pub plus: Option<char>,
    /// The value may be left out.
    pub optional_value: bool,
    /// Words each occurrence takes; 1 unless declared `values = N`.
    pub values: usize,
    /// It may take more than `values` (`values = 1..`): shown as `<V>...`.
    pub more_values: bool,
    /// What its value is, for the shell that completes it.
    pub hint: crate::complete::Hint,
    /// What its value can be, given what is typed of it: asked when a shell
    /// completes it, in place of its choices and its hint.
    pub complete: Option<fn(&str) -> Vec<crate::complete::Candidate>>,
    /// What to know before using it, shown by `--help` as `Warning: …`;
    /// empty for none.
    pub warning: &'static str,
}

impl Item {
    /// A switch with no name. A value written by hand ends in
    /// `..Item::EMPTY`, and a release that adds a field does not break it.
    pub const EMPTY: Item = Item {
        short: None,
        long: None,
        aliases: &[],
        negate: None,
        value_name: None,
        help: "",
        long_help: "",
        heading: None,
        hide: false,
        positional: false,
        required: false,
        multiple: false,
        trailing: false,
        default: None,
        env: None,
        choices: &[],
        require_equals: false,
        global: false,
        more_shorts: &[],
        plus: None,
        optional_value: false,
        values: 1,
        more_values: false,
        hint: crate::complete::Hint::Nothing,
        complete: None,
        warning: "",
    };
}

/// Whether two items of different `parts` are spelled alike (a short letter,
/// a long name or its negation): a struct and what it flattens, checked at
/// compile time.
#[doc(hidden)]
pub const fn items_clash(parts: &[&[Item]]) -> bool {
    const fn same(a: Option<&str>, b: Option<&str>) -> bool {
        let (Some(a), Some(b)) = (a, b) else {
            return false;
        };
        let (a, b) = (a.as_bytes(), b.as_bytes());
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }
        true
    }
    /// Whether `item` answers to the letter.
    const fn has_short(item: &Item, letter: char) -> bool {
        if matches!(item.short, Some(c) if c == letter) {
            return true;
        }
        let mut i = 0;
        while i < item.more_shorts.len() {
            if item.more_shorts[i] == letter {
                return true;
            }
            i += 1;
        }
        false
    }
    /// Whether `item` answers to the long name: its own, its negation, an alias.
    const fn has_long(item: &Item, name: Option<&str>) -> bool {
        if same(item.long, name) || same(item.negate, name) {
            return true;
        }
        let mut i = 0;
        while i < item.aliases.len() {
            if same(Some(item.aliases[i]), name) {
                return true;
            }
            i += 1;
        }
        false
    }
    const fn clash(a: &Item, b: &Item) -> bool {
        if a.positional || b.positional {
            return false;
        }
        if matches!(a.short, Some(c) if has_short(b, c)) {
            return true;
        }
        let mut i = 0;
        while i < a.more_shorts.len() {
            if has_short(b, a.more_shorts[i]) {
                return true;
            }
            i += 1;
        }
        if has_long(b, a.long) || has_long(b, a.negate) {
            return true;
        }
        let mut i = 0;
        while i < a.aliases.len() {
            if has_long(b, Some(a.aliases[i])) {
                return true;
            }
            i += 1;
        }
        false
    }
    let mut p = 0;
    while p < parts.len() {
        let mut q = p + 1;
        while q < parts.len() {
            let (mut i, a, b) = (0, parts[p], parts[q]);
            while i < a.len() {
                let mut j = 0;
                while j < b.len() {
                    if clash(&a[i], &b[j]) {
                        return true;
                    }
                    j += 1;
                }
                i += 1;
            }
            q += 1;
        }
        p += 1;
    }
    false
}

/// `parts`, one after another, as one array: a struct's own items and those
/// of the structs it flattens, joined at compile time. `N` is their total
/// length.
///
/// # Panics
///
/// At compile time, if `N` is not the total length.
#[doc(hidden)]
pub const fn concat_items<const N: usize>(parts: &[&[Item]]) -> [Item; N] {
    let mut out = [Item::EMPTY; N];
    let (mut n, mut p) = (0, 0);
    while p < parts.len() {
        let part = parts[p];
        let mut i = 0;
        while i < part.len() {
            out[n] = part[i];
            n += 1;
            i += 1;
        }
        p += 1;
    }
    assert!(n == N, "concat_items: the length is not the parts' total");
    out
}

/// [`concat_items`] for what is said under headings: a struct's own, then
/// those of the structs it flattens. The first that names a title is read.
///
/// # Panics
///
/// At compile time, if `N` is not the total length.
#[doc(hidden)]
pub const fn concat_headings<const N: usize>(parts: &[&[Heading]]) -> [Heading; N] {
    let mut out = [Heading {
        title: "",
        help: "",
    }; N];
    let (mut n, mut p) = (0, 0);
    while p < parts.len() {
        let part = parts[p];
        let mut i = 0;
        while i < part.len() {
            out[n] = part[i];
            n += 1;
            i += 1;
        }
        p += 1;
    }
    assert!(
        n == N,
        "concat_headings: the length is not the parts' total"
    );
    out
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
/// the width of the terminal on standard output; else 100.
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

    /// [`Palette::CLAP`]: bold and underline in help, color only in errors.
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

    pub(crate) fn flag(self) -> Ink {
        self.ink(self.palette.flag)
    }

    pub(crate) fn command(self) -> Ink {
        self.ink(self.palette.command)
    }

    pub(crate) fn dim(self) -> Ink {
        self.ink(self.palette.dim)
    }

    pub(crate) fn code(self) -> Ink {
        self.ink(self.palette.code)
    }

    pub(crate) fn emphasis(self) -> Ink {
        self.ink(self.palette.emphasis)
    }

    pub(crate) fn strong(self) -> Ink {
        self.ink(self.palette.strong)
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

    pub(crate) fn env(self) -> Ink {
        self.ink(self.palette.env)
    }

    pub(crate) fn default(self) -> Ink {
        self.ink(self.palette.default)
    }

    pub(crate) fn choice(self) -> Ink {
        self.ink(self.palette.choice)
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

    /// `<NAME>` painted whole, or `[NAME]` with only the name painted: square
    /// brackets say "optional", they are not part of the value.
    fn push_placeholder(&mut self, style: Style, required: bool, name: &str) {
        if required {
            self.push(style.placeholder(), &format!("<{name}>"));
        } else {
            self.push(style.dim(), "[");
            self.push(style.placeholder(), name);
            self.push(style.dim(), "]");
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
    render_page(command, path, long, width, style, &[])
}

/// [`render_styled`] for a subcommand, which also takes `inherited`: the
/// global flags of the commands above it.
pub(crate) fn render_page(
    command: &Command,
    path: &[&str],
    long: bool,
    width: usize,
    style: Style,
    inherited: &[&Item],
) -> String {
    let mut out = String::new();
    let layout = Layout { long, width, style };
    page(&mut out, command, path, layout, inherited);
    // The default subcommand's own page. A flattened page has it already.
    if let Some(sub) = command.default_sub()
        && command.default_subcommand_help
        && !command.flatten_help
    {
        trim_blank_lines(&mut out);
        out.push('\n');
        heading(&mut out, "Default command:", style);
        out.push(' ');
        push_painted(&mut out, style.command(), sub.name);
        out.push_str("\n  It runs when no command is named.\n\n");
        let mut path = path.to_vec();
        path.push(sub.name);
        // What it inherits is this command's, listed above.
        page(&mut out, sub.command, &path, layout, &[]);
    }
    trim_blank_lines(&mut out);
    out
}

/// End `out` with one newline.
fn trim_blank_lines(out: &mut String) {
    while out.ends_with("\n\n") {
        out.pop();
    }
}

/// What a page is rendered with.
#[derive(Clone, Copy)]
struct Layout {
    long: bool,
    width: usize,
    style: Style,
}

/// One command's page, appended to `out`.
fn page(out: &mut String, command: &Command, path: &[&str], layout: Layout, inherited: &[&Item]) {
    let Layout { long, width, style } = layout;
    let about = if long && !command.long_about.is_empty() {
        command.long_about
    } else {
        command.about
    };
    if !about.is_empty() {
        let about = painted_marks(about, style);
        let _ = writeln!(out, "{}\n", wrap_text(&about, width));
    }
    let visible_subs: Vec<&Sub> = command.subcommands.iter().filter(|s| !s.hide).collect();
    let flat = command.flatten_help && !visible_subs.is_empty();
    push_painted(out, style.header(), "Usage:");
    let mut lines = Vec::new();
    if !flat || !command.subcommand_required {
        lines.push(usage(command, path, style));
    }
    if flat {
        lines.extend(visible_subs.iter().map(|sub| {
            let mut path = path.to_vec();
            path.push(sub.name);
            usage(sub.command, &path, style)
        }));
    }
    for (n, line) in lines.iter().enumerate() {
        let _ = writeln!(out, "{}{line}", if n == 0 { " " } else { "       " });
    }
    out.push('\n');

    if !flat && !visible_subs.is_empty() {
        let default = command.default_sub().map(|sub| sub.name);
        let rows: Vec<(Cell, String)> = visible_subs
            .iter()
            .map(|s| {
                let mut name = Cell::default();
                name.push(style.command(), s.name);
                for alias in s.aliases {
                    name.push(Ink::NONE, ", ");
                    name.push(style.command(), alias);
                }
                let mut about = painted_marks(s.about, style);
                if default == Some(s.name) {
                    if !about.is_empty() {
                        about.push(' ');
                    }
                    // As the value of a `[default: …]` is.
                    push_painted(&mut about, style.default(), "(default)");
                }
                (name, about)
            })
            .collect();
        // Both pages list a subcommand beside what it is for.
        section(
            out,
            "Commands",
            None,
            &rows,
            Layout {
                long: false,
                ..layout
            },
        );
    }
    if flat {
        flat_commands(out, command, path, layout);
    }

    section(
        out,
        "Arguments",
        None,
        &argument_rows(command, layout),
        layout,
    );

    let visible: Vec<&Item> = command.items.iter().filter(|i| !i.hide).collect();
    let mut headings: Vec<Option<&str>> = vec![None];
    for item in visible.iter().filter(|i| !i.positional) {
        if !headings.contains(&item.heading) {
            headings.push(item.heading);
        }
    }
    let builtin = |short: &str, long_name: &str| {
        let mut cell = Cell::default();
        cell.push(style.flag(), short);
        cell.push(Ink::NONE, ", ");
        cell.push(style.flag(), long_name);
        cell
    };
    for title in headings {
        let mut rows: Vec<(Cell, String)> = visible
            .iter()
            .filter(|i| !i.positional && i.heading == title)
            .map(|i| (flag_spec(i, style), describe(i, long, style)))
            .collect();
        if title.is_none() {
            if command.help_flag {
                // The other spelling, painted as a flag is.
                let what = if long {
                    format!(
                        "Print help (see a summary with '{}')",
                        Style::paint(style.flag(), "-h")
                    )
                } else {
                    format!(
                        "Print help (see more with '{}')",
                        Style::paint(style.flag(), "--help")
                    )
                };
                let cell = if command.help_short {
                    builtin("-h", "--help")
                } else {
                    let mut cell = Cell::default();
                    cell.push(Ink::NONE, "    ");
                    cell.push(style.flag(), "--help");
                    cell
                };
                let what = if command.help_short {
                    what
                } else {
                    "Print help".to_owned()
                };
                rows.push((cell, what));
            }
            if command.version.is_some() {
                rows.push((builtin("-V", "--version"), "Print version".into()));
            }
        }
        let prose = title.and_then(|title| command.heading_help(title));
        section(out, title.unwrap_or("Options"), prose, &rows, layout);
    }

    let rows: Vec<(Cell, String)> = inherited
        .iter()
        .map(|i| (flag_spec(i, style), describe(i, long, style)))
        .collect();
    section(out, "Global options", None, &rows, layout);

    examples(out, command, path, inherited, layout);

    let after = if long && !command.after_long_help.is_empty() {
        command.after_long_help
    } else {
        command.after_help
    };
    if !after.is_empty() {
        let _ = writeln!(out, "{}", wrap_text(after, width));
    }
}

/// A command's positionals, as the rows of its Arguments section.
fn argument_rows(command: &Command, layout: Layout) -> Vec<(Cell, String)> {
    let shown = command.items.iter().filter(|i| i.positional && !i.hide);
    shown
        .map(|i| {
            let mut cell = Cell::default();
            push_item_placeholder(&mut cell, layout.style, i);
            (cell, describe(i, layout.long, layout.style))
        })
        .collect()
}

/// Each subcommand of a `flatten_help` command under its own command line:
/// what it is, then what it takes, and its own subcommands if it flattens
/// them too.
fn flat_commands(out: &mut String, command: &Command, path: &[&str], layout: Layout) {
    let Layout { long, style, .. } = layout;
    for sub in command.subcommands.iter().filter(|sub| !sub.hide) {
        let mut path = path.to_vec();
        path.push(sub.name);
        heading(out, &format!("{}:", path.join(" ")), style);
        out.push('\n');
        let about = if long && !sub.command.long_about.is_empty() {
            sub.command.long_about
        } else {
            sub.about
        };
        indented(out, about, 2, layout);
        let mut rows = argument_rows(sub.command, layout);
        let flags = sub
            .command
            .items
            .iter()
            .filter(|i| !i.positional && !i.hide);
        rows.extend(flags.map(|i| (flag_spec(i, style), describe(i, long, style))));
        if long && !about.is_empty() && !rows.is_empty() {
            out.push('\n');
        }
        list(out, &rows, layout);
        if !long || rows.is_empty() {
            out.push('\n');
        }
        if sub.command.flatten_help {
            flat_commands(out, sub.command, &path, layout);
        }
    }
}

/// The Examples section: each line after `$`, under its header if it has
/// one, and after what it does on the long page.
fn examples(
    out: &mut String,
    command: &Command,
    path: &[&str],
    inherited: &[&Item],
    layout: Layout,
) {
    let Layout { long, style, .. } = layout;
    let examples = command.examples;
    if examples.is_empty() {
        return;
    }
    heading(out, "Examples:", style);
    out.push('\n');
    let mut spaced = false;
    for (n, example) in examples.iter().enumerate() {
        let help = Some(example.help).filter(|help| long && !help.is_empty());
        let said = !example.header.is_empty() || help.is_some();
        // A line that is introduced stands apart from its neighbors.
        if n > 0 && (said || spaced) {
            out.push('\n');
        }
        spaced = said;
        if !example.header.is_empty() {
            let _ = writeln!(out, "  {}:", painted_marks(example.header, style));
        }
        if let Some(help) = help {
            indented(out, help, 4, layout);
        }
        out.push_str(if said { "    " } else { "  " });
        push_painted(out, style.dim(), "$");
        out.push(' ');
        push_example_line(out, example.code, command, path, inherited, style);
        out.push('\n');
    }
    out.push('\n');
}

/// An example's line as typed, each word painted by what it is: the program,
/// the subcommands that lead from `path` on, flags, `<NAME>` and `[NAME]`, and
/// a `# comment`. The values of flags, the positionals and what follows `--`
/// or a shell operator stay in the `code` ink. A quoted or escaped space does
/// not end a word, and the spacing is kept.
fn push_example_line(
    out: &mut String,
    line: &str,
    command: &Command,
    path: &[&str],
    inherited: &[&Item],
    style: Style,
) {
    let mut command = Some(command);
    let mut ahead = path.get(1..).unwrap_or_default();
    let (mut program, mut flags, mut values) = (true, true, 0usize);
    let mut at = 0;
    for (start, end) in shell_words(line) {
        out.push_str(&line[at..start]);
        at = end;
        let word = &line[start..end];
        let items = || {
            let own = command.map_or(&[][..], |c| c.items).iter();
            own.chain(inherited.iter().copied())
                .filter(|i| !i.positional)
        };
        if word.starts_with('#') {
            push_painted(out, style.dim(), &line[start..]);
            return;
        }
        if word.bytes().all(|b| b"|&;<>()".contains(&b)) {
            push_painted(out, style.dim(), word);
            (command, program, flags, values) = (None, true, true, 0);
            continue;
        }
        let bare = word.strip_suffix("...").unwrap_or(word);
        let placeholder = (bare.starts_with('<') && bare.ends_with('>'))
            || (bare.starts_with('[') && bare.ends_with(']'));
        if placeholder && bare.len() > 2 {
            push_painted(out, style.placeholder(), word);
            values = values.saturating_sub(1);
            continue;
        }
        if values > 0 || program {
            push_painted(out, style.code(), word);
            values = values.saturating_sub(1);
            program = false;
            continue;
        }
        if flags && word == "--" {
            push_painted(out, style.flag(), word);
            (command, flags) = (None, false);
            continue;
        }
        if flags && let Some(taken) = flag_word(word, items()) {
            let name = word.find('=').map_or(word, |eq| &word[..eq]);
            push_painted(out, style.flag(), name);
            if name.len() < word.len() {
                push_painted(out, style.code(), &word[name.len()..]);
            }
            values = taken;
            continue;
        }
        let sub = match ahead.split_first() {
            Some((next, rest)) => (*next == word).then(|| {
                ahead = rest;
                None
            }),
            None => command
                .and_then(|c| c.subcommands.iter().find(|s| s.names.contains(&word)))
                .map(|s| Some(s.command)),
        };
        match sub {
            Some(next) => {
                push_painted(out, style.command(), word);
                if let Some(next) = next {
                    command = Some(next);
                }
            }
            None => {
                push_painted(out, style.code(), word);
                (command, ahead) = (None, &[]);
            }
        }
    }
    out.push_str(&line[at..]);
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
            i.long == Some(name) || i.aliases.contains(&name) || i.negate == Some(name)
        })?;
        return Some(if attached || item.negate == Some(name) {
            0
        } else {
            takes(item)
        });
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
        // The brackets and `--` are typed, so they are painted as literals.
        let mut cell = Cell::default();
        cell.push(Ink::NONE, " ");
        cell.push(style.dim(), "[");
        cell.push(style.flag(), "--");
        cell.push(Ink::NONE, " ");
        push_item_placeholder(&mut cell, style, item);
        cell.push(style.dim(), "]");
        line.push_str(&cell.text);
    }
    line
}

/// A positional's placeholder: `<NAME>` or `[NAME]`, then `...` if repeated.
fn push_item_placeholder(cell: &mut Cell, style: Style, item: &Item) {
    cell.push_placeholder(style, item.required, item.value_name.unwrap_or("VALUE"));
    if item.multiple {
        cell.push(style.dim(), "...");
    }
}

fn flag_spec(item: &Item, style: Style) -> Cell {
    let mut spec = Cell::default();
    // Every spelling: `-l, -L`, `-e, +e`, `+o`, `-v, --verbose`; a long name
    // alone is indented to line up with those after a short one.
    let shorts = item.short.iter().chain(item.more_shorts);
    let mut names: Vec<String> = shorts.map(|s| format!("-{s}")).collect();
    names.extend(item.plus.map(|c| format!("+{c}")));
    if names.is_empty() && item.long.is_some() {
        spec.push(Ink::NONE, "    ");
    }
    names.extend(item.long.map(|l| format!("--{l}")));
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            spec.push(Ink::NONE, ", ");
        }
        spec.push(style.flag(), name);
    }
    if let Some(no) = item.negate {
        spec.push(Ink::NONE, " / ");
        spec.push(style.flag(), &format!("--{no}"));
    }
    if let Some(value) = item.value_name {
        // `<V>`, `=<V>` when only attached, in brackets when optional, and
        // once a word for `values = N`.
        let more = if item.more_values { "..." } else { "" };
        let value = vec![format!("<{value}>"); item.values.max(1)].join(" ") + more;
        let (open, close) = match (item.require_equals, item.optional_value) {
            (true, true) => ("[=", "]"),
            (true, false) => ("=", ""),
            (false, true) => (" [", "]"),
            (false, false) => (" ", ""),
        };
        spec.push(Ink::NONE, open);
        spec.push(style.placeholder(), &value);
        spec.push(Ink::NONE, close);
    }
    // `<V>...` already says the flag takes more words.
    if item.multiple && !item.more_values {
        spec.push(style.dim(), "...");
    }
    spec
}

/// An item's description, then `[env: …]`, `[default: …]` and
/// `[possible values: …]`, each value painted on its own (so a line never
/// breaks inside a painted span).
fn describe(item: &Item, long: bool, style: Style) -> String {
    let help = if long && !item.long_help.is_empty() {
        item.long_help
    } else {
        item.help
    };
    let mut text = painted_marks(help, style);
    let mut add = |extra: String| {
        if !text.is_empty() {
            text.push(if long { '\n' } else { ' ' });
        }
        text.push_str(&extra);
    };
    let label = |label: &str| Style::paint(style.dim(), label);
    let close = || Style::paint(style.dim(), "]");
    if let Some(env) = item.env {
        let env = Style::paint(style.env(), env);
        add(format!("{}{env}{}", label("[env: "), close()));
    }
    if let Some(default) = item.default {
        let default = Style::paint(style.default(), default);
        add(format!("{}{default}{}", label("[default: "), close()));
    }
    if !item.choices.is_empty() {
        let choices: Vec<String> = item
            .choices
            .iter()
            .map(|c| Style::paint(style.choice(), c))
            .collect();
        add(format!(
            "{}{}{}",
            label("[possible values: "),
            choices.join(", "),
            close()
        ));
    }
    if long && !item.warning.is_empty() {
        let label = Style::paint(style.error(), "Warning:");
        add(format!("{label} {}", painted_marks(item.warning, style)));
    }
    text
}

/// A heading: painted as one, and a marked span of it as both a heading and
/// what its marks say, in the heading's color. `*shown*` in a bold heading is
/// bold and italic.
fn heading(out: &mut String, title: &str, style: Style) {
    let header = style.header();
    let inks = [style.code(), style.emphasis(), style.strong()];
    let mut at = 0;
    if style.depth != Depth::None {
        while let Some((open, end, ink)) = marked(title, at, inks) {
            if at < open {
                push_painted(out, header, &title[at..open]);
            }
            let both = Paint {
                fg: header.paint.fg.or(ink.paint.fg),
                bold: header.paint.bold || ink.paint.bold,
                dim: header.paint.dim || ink.paint.dim,
                italic: header.paint.italic || ink.paint.italic,
                underline: header.paint.underline || ink.paint.underline,
            };
            push_painted(out, style.ink(both), &title[open..end]);
            at = end;
        }
    }
    push_painted(out, header, &title[at..]);
}

/// `text` with each `` `quoted` ``, `*emphasized*` and `**strong**` span
/// painted, marks kept (so the painted page is the plain one plus escapes).
/// Each word of a span is painted on its own, so wrapping never breaks inside
/// a painted run; a mark with no pair on its line is left alone.
fn painted_marks(text: &str, style: Style) -> String {
    let inks = [style.code(), style.emphasis(), style.strong()];
    if style.depth == Depth::None
        || inks.iter().all(|ink| ink.paint == Paint::NONE)
        || !text.contains(['`', '*'])
    {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + 16);
    for (n, line) in text.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        let mut at = 0;
        while let Some((open, end, ink)) = marked(line, at, inks) {
            out.push_str(&line[at..open]);
            for (i, word) in line[open..end].split(' ').enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                if !word.is_empty() {
                    push_painted(&mut out, ink, word);
                }
            }
            at = end;
        }
        out.push_str(&line[at..]);
    }
    out
}

/// The first marked span of `line` at or after `from`, marks included, and
/// its ink among those of a quoted, an emphasized and a strong one.
///
/// Stars whose ink paints nothing mark no span, so a quoted span between
/// them is still painted.
///
/// Stars mark a span only around it: the opening ones follow no letter or
/// digit and touch the span's first character, the closing ones touch its
/// last and precede no letter or digit. `2*3*4`, `*.rs or *.toml`, a
/// bullet's `* ` and `\*` are not spans. Three stars or more are left alone.
fn marked(
    line: &str,
    from: usize,
    [code, emphasis, strong]: [Ink; 3],
) -> Option<(usize, usize, Ink)> {
    let bytes = line.as_bytes();
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'`' => {
                if let Some(len) = line[at + 1..].find('`') {
                    return Some((at, at + len + 2, code));
                }
                at += 1;
            }
            b'*' => {
                let stars = run(bytes, at);
                let before = line[..at].chars().next_back();
                let after = line[at + stars..].chars().next();
                let ink = if stars == 2 { strong } else { emphasis };
                if stars <= 2
                    && ink.paint != Paint::NONE
                    && before.is_none_or(|c| !c.is_alphanumeric() && c != '\\')
                    && after.is_some_and(|c| !c.is_whitespace())
                    && let Some(end) = closing(line, at + stars, stars)
                {
                    return Some((at, end, ink));
                }
                at += stars;
            }
            _ => at += 1,
        }
    }
    None
}

/// How many stars are in a row at `at`.
fn run(bytes: &[u8], at: usize) -> usize {
    bytes[at..].iter().take_while(|b| **b == b'*').count()
}

/// The end of the span that `stars` stars opened before `from`: past the
/// next run of as many, if it closes one. If it does not, nothing was opened:
/// in `*.rs is *never* read` the span is `*never*`. A quoted span on the way
/// is skipped, stars and all.
fn closing(line: &str, from: usize, stars: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'`' => at += line[at + 1..].find('`').map_or(1, |len| len + 2),
            b'*' => {
                let found = run(bytes, at);
                let before = line[..at].chars().next_back();
                let after = line[at + found..].chars().next();
                if found == stars {
                    let closes = before.is_some_and(|c| !c.is_whitespace() && c != '\\')
                        && after.is_none_or(|c| !c.is_alphanumeric());
                    return closes.then_some(at + found);
                }
                at += found;
            }
            _ => at += 1,
        }
    }
    None
}

/// A titled two-column list, after what `--help` says under the title.
fn section(
    out: &mut String,
    title: &str,
    prose: Option<&str>,
    rows: &[(Cell, String)],
    layout: Layout,
) {
    if rows.is_empty() {
        return;
    }
    heading(out, &format!("{title}:"), layout.style);
    out.push('\n');
    if let Some(prose) = prose.filter(|_| layout.long) {
        indented(out, prose, 2, layout);
        out.push('\n');
    }
    list(out, rows, layout);
    if !layout.long {
        out.push('\n');
    }
}

/// A two-column list; long help puts each description under its item.
///
/// Descriptions wrap to the width. The column is at most two fifths of the
/// page, so one long spelling does not squeeze every
/// description; an item wider than that has its description on the next line.
fn list(out: &mut String, rows: &[(Cell, String)], layout: Layout) {
    let Layout { long, width, .. } = layout;
    let longest = rows.iter().map(|(left, _)| left.width).max().unwrap_or(0);
    let available = width.saturating_sub(4);
    let column = longest.min(available / 5 * 2 + available % 5 * 2 / 5);
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
}

/// `text` painted and wrapped, each line after `indent` spaces; a blank line
/// between two paragraphs stays empty.
fn indented(out: &mut String, text: &str, indent: usize, layout: Layout) {
    let text = painted_marks(text, layout.style);
    let room = layout.width.saturating_sub(indent).max(20);
    for line in text.lines() {
        if line.trim().is_empty() {
            out.push('\n');
        }
        for piece in Wrap::new(line, room) {
            let _ = writeln!(out, "{:indent$}{piece}", "");
        }
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
