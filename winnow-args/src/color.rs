//! Colors for help and errors, for the terminal at hand.
//!
//! Terminals show 16, 256 or 16 million colors. [`Depth::detect`] works out
//! which from the environment, the way `supports-color` and `anstyle-query`
//! do, and a [`Theme`] offers a [`Palette`] for each depth: the richest one
//! the terminal can show is used, and a color deeper than the terminal is
//! mapped to the nearest one it has. The default theme uses only the 16 basic
//! colors, which the terminal's own theme defines — so they suit light and
//! dark backgrounds alike.

use std::fmt::Write as _;

/// How many colors a terminal shows. Ordered: a deeper terminal shows
/// everything a shallower one does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Depth {
    /// No color and no attributes: a pipe, a file, `NO_COLOR`.
    None,
    /// The 16 basic colors, bold, underline: every color terminal.
    Ansi16,
    /// The xterm 256-color palette: `TERM=*-256color`.
    Ansi256,
    /// 24-bit color: `COLORTERM=truecolor`.
    TrueColor,
}

impl Depth {
    /// The depth for a stream that is (or is not) a terminal, from the
    /// environment:
    ///
    /// - a non-empty `NO_COLOR` means none, whatever else is set;
    /// - `FORCE_COLOR` (`1`–`3`, or empty/`true` for 1) or `CLICOLOR_FORCE`
    ///   (not `0`) give color even to a pipe;
    /// - otherwise none for a pipe, `CLICOLOR=0`, `TERM=dumb`, or no `TERM`
    ///   at all;
    /// - then 24-bit for `COLORTERM=truecolor`/`24bit`, a `TERM` ending in
    ///   `direct` or `truecolor`, or iTerm2; 256 for a `TERM` ending in
    ///   `256`/`256color` or Apple's Terminal; else 16.
    ///
    /// Under [`with_env`](crate::with_env) the stream never counts as a
    /// terminal, so the answer depends only on the variables given.
    pub fn detect(is_terminal: bool) -> Depth {
        let var = |name| crate::env::var(name);
        let text = |name| var(name).and_then(|v| v.into_string().ok());
        if var("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return Depth::None;
        }
        let forced = match text("FORCE_COLOR").as_deref() {
            Some("" | "true") => Depth::Ansi16,
            Some("false" | "0") => Depth::None,
            Some(level) => match level.parse::<u8>() {
                Ok(0) => Depth::None,
                Ok(1) | Err(_) => Depth::Ansi16,
                Ok(2) => Depth::Ansi256,
                Ok(_) => Depth::TrueColor,
            },
            None if var("CLICOLOR_FORCE").is_some_and(|v| !v.is_empty() && v != "0") => {
                Depth::Ansi16
            }
            None => Depth::None,
        };
        let term = text("TERM");
        if forced == Depth::None {
            let terminal = is_terminal && !crate::env::overridden();
            let refused = var("CLICOLOR").is_some_and(|v| v == "0");
            let dumb = term.as_deref().is_none_or(|term| term == "dumb");
            if !terminal || refused || dumb {
                return Depth::None;
            }
        }
        let term = term.as_deref().unwrap_or_default();
        let program = text("TERM_PROGRAM");
        let detected = if matches!(text("COLORTERM").as_deref(), Some("truecolor" | "24bit"))
            || term.ends_with("direct")
            || term.ends_with("truecolor")
            || program.as_deref() == Some("iTerm.app")
        {
            Depth::TrueColor
        } else if term.ends_with("256")
            || term.ends_with("256color")
            || program.as_deref() == Some("Apple_Terminal")
        {
            Depth::Ansi256
        } else {
            Depth::Ansi16
        };
        detected.max(forced)
    }
}

/// The 16 basic colors. What they look like is the terminal theme's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs, reason = "the names are the colors")]
pub enum Ansi {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Ansi {
    const ALL: [Ansi; 16] = [
        Ansi::Black,
        Ansi::Red,
        Ansi::Green,
        Ansi::Yellow,
        Ansi::Blue,
        Ansi::Magenta,
        Ansi::Cyan,
        Ansi::White,
        Ansi::BrightBlack,
        Ansi::BrightRed,
        Ansi::BrightGreen,
        Ansi::BrightYellow,
        Ansi::BrightBlue,
        Ansi::BrightMagenta,
        Ansi::BrightCyan,
        Ansi::BrightWhite,
    ];

    /// Its index, 0–15, as in the 256-color palette.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The SGR foreground code: 30–37, 90–97.
    const fn code(self) -> u8 {
        let i = self as u8;
        if i < 8 { 30 + i } else { 90 + i - 8 }
    }

    /// xterm's default RGB for it, to measure other colors against.
    const fn rgb(self) -> (u8, u8, u8) {
        match self {
            Ansi::Black => (0, 0, 0),
            Ansi::Red => (205, 0, 0),
            Ansi::Green => (0, 205, 0),
            Ansi::Yellow => (205, 205, 0),
            Ansi::Blue => (0, 0, 238),
            Ansi::Magenta => (205, 0, 205),
            Ansi::Cyan => (0, 205, 205),
            Ansi::White => (229, 229, 229),
            Ansi::BrightBlack => (127, 127, 127),
            Ansi::BrightRed => (255, 0, 0),
            Ansi::BrightGreen => (0, 255, 0),
            Ansi::BrightYellow => (255, 255, 0),
            Ansi::BrightBlue => (92, 92, 255),
            Ansi::BrightMagenta => (255, 0, 255),
            Ansi::BrightCyan => (0, 255, 255),
            Ansi::BrightWhite => (255, 255, 255),
        }
    }
}

/// A foreground color at any depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// One of the 16 basic colors.
    Ansi(Ansi),
    /// An entry of the xterm 256-color palette.
    Ansi256(u8),
    /// 24-bit color.
    Rgb(u8, u8, u8),
}

/// The six levels of each channel in the 256-color cube (entries 16–231).
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

impl Color {
    /// The depth it needs.
    pub const fn depth(self) -> Depth {
        match self {
            Color::Ansi(_) => Depth::Ansi16,
            Color::Ansi256(_) => Depth::Ansi256,
            Color::Rgb(..) => Depth::TrueColor,
        }
    }

    /// Its approximate RGB (xterm's defaults for the palette colors).
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Ansi(a) => a.rgb(),
            Color::Ansi256(n @ 0..=15) => Ansi::ALL[usize::from(n)].rgb(),
            Color::Ansi256(n @ 16..=231) => {
                let n = n - 16;
                let level = |i: u8| CUBE[usize::from(i)];
                (level(n / 36), level(n / 6 % 6), level(n % 6))
            }
            Color::Ansi256(n) => {
                let gray = 8 + (n - 232) * 10;
                (gray, gray, gray)
            }
            Color::Rgb(r, g, b) => (r, g, b),
        }
    }

    /// The nearest color a terminal of `depth` can show; `None` for
    /// [`Depth::None`].
    pub fn at(self, depth: Depth) -> Option<Color> {
        match depth {
            Depth::None => None,
            _ if self.depth() <= depth => Some(self),
            Depth::Ansi256 => Some(Color::Ansi256(to_256(self.rgb()))),
            Depth::Ansi16 | Depth::TrueColor => Some(Color::Ansi(to_16(self.rgb()))),
        }
    }

    /// Append the SGR parameters that select it as the foreground.
    fn write_sgr(self, out: &mut String) {
        let _ = match self {
            Color::Ansi(a) => write!(out, "{}", a.code()),
            Color::Ansi256(n) => write!(out, "38;5;{n}"),
            Color::Rgb(r, g, b) => write!(out, "38;2;{r};{g};{b}"),
        };
    }
}

/// Squared distance between two colors, weighted for how the eye sees them.
fn distance((r1, g1, b1): (u8, u8, u8), (r2, g2, b2): (u8, u8, u8)) -> u32 {
    let d = |a: u8, b: u8| u32::from(a.abs_diff(b)).pow(2);
    2 * d(r1, r2) + 4 * d(g1, g2) + 3 * d(b1, b2)
}

/// The nearest basic color.
fn to_16(rgb: (u8, u8, u8)) -> Ansi {
    Ansi::ALL
        .into_iter()
        .min_by_key(|a| distance(a.rgb(), rgb))
        .unwrap_or(Ansi::White)
}

/// The nearest entry of the cube or the gray ramp (16–255): the basic colors
/// are left out, since their look is the terminal theme's.
fn to_256(rgb: (u8, u8, u8)) -> u8 {
    let level = |c: u8| {
        CUBE.iter()
            .enumerate()
            .min_by_key(|(_, l)| l.abs_diff(c))
            .map_or(0, |(i, _)| i as u8)
    };
    let (r, g, b) = (level(rgb.0), level(rgb.1), level(rgb.2));
    let cube = 16 + 36 * r + 6 * g + b;
    let average = (u16::from(rgb.0) + u16::from(rgb.1) + u16::from(rgb.2)) / 3;
    let step = (average.saturating_sub(3) / 10).min(23) as u8;
    let gray = 232 + step;
    if distance(Color::Ansi256(gray).rgb(), rgb) < distance(Color::Ansi256(cube).rgb(), rgb) {
        gray
    } else {
        cube
    }
}

/// How to paint one kind of text: a color and attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Paint {
    /// The foreground color, if any.
    pub fg: Option<Color>,
    /// Bold (or bright, on some terminals).
    pub bold: bool,
    /// Dim.
    pub dim: bool,
    /// Italic.
    pub italic: bool,
    /// Underlined.
    pub underline: bool,
}

impl Paint {
    /// Nothing: the text as the terminal shows it.
    pub const NONE: Paint = Paint {
        fg: None,
        bold: false,
        dim: false,
        italic: false,
        underline: false,
    };

    /// In `color`.
    pub const fn fg(color: Color) -> Paint {
        Paint {
            fg: Some(color),
            ..Paint::NONE
        }
    }

    /// In one of the basic colors.
    pub const fn ansi(color: Ansi) -> Paint {
        Paint::fg(Color::Ansi(color))
    }

    /// And bold.
    pub const fn bold(self) -> Paint {
        Paint { bold: true, ..self }
    }

    /// And dim.
    pub const fn dim(self) -> Paint {
        Paint { dim: true, ..self }
    }

    /// And italic.
    pub const fn italic(self) -> Paint {
        Paint {
            italic: true,
            ..self
        }
    }

    /// And underlined.
    pub const fn underline(self) -> Paint {
        Paint {
            underline: true,
            ..self
        }
    }

    /// The deepest color it uses.
    pub const fn depth(self) -> Depth {
        match self.fg {
            Some(color) => color.depth(),
            None => Depth::Ansi16,
        }
    }

    /// Append `text` painted for a terminal of `depth`: one SGR sequence,
    /// the text, a reset. Plain when `depth` is [`Depth::None`] or there is
    /// nothing to paint.
    pub fn write(self, out: &mut String, depth: Depth, text: &str) {
        if depth == Depth::None || self == Paint::NONE {
            out.push_str(text);
            return;
        }
        out.push_str("\u{1b}[");
        let mut first = true;
        let mut sep = |out: &mut String| {
            if !first {
                out.push(';');
            }
            first = false;
        };
        for (on, code) in [
            (self.bold, '1'),
            (self.dim, '2'),
            (self.italic, '3'),
            (self.underline, '4'),
        ] {
            if on {
                sep(out);
                out.push(code);
            }
        }
        if let Some(color) = self.fg.and_then(|c| c.at(depth)) {
            sep(out);
            color.write_sgr(out);
        }
        out.push('m');
        out.push_str(text);
        out.push_str("\u{1b}[0m");
    }
}

/// A paint for each kind of text in help and errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette {
    /// `Usage:`, `Options:` and the other headings.
    pub header: Paint,
    /// The program's name in the usage line.
    pub program: Paint,
    /// What is typed as is: flags, subcommand names.
    pub literal: Paint,
    /// Value names: all of `<PATH>`, the name in `[FILE]`.
    pub placeholder: Paint,
    /// `error:`.
    pub error: Paint,
    /// The part of the command line an error is about.
    pub invalid: Paint,
    /// What an error says is missing: something to type.
    pub valid: Paint,
    /// The environment variable in `[env: …]`.
    pub env: Paint,
    /// The value in `[default: …]`.
    pub default: Paint,
    /// Each value in `[possible values: …]`.
    pub choice: Paint,
}

impl Palette {
    /// Nothing painted.
    pub const PLAIN: Palette = Palette {
        header: Paint::NONE,
        program: Paint::NONE,
        literal: Paint::NONE,
        placeholder: Paint::NONE,
        error: Paint::NONE,
        invalid: Paint::NONE,
        valid: Paint::NONE,
        env: Paint::NONE,
        default: Paint::NONE,
        choice: Paint::NONE,
    };

    /// The default in the 16 basic colors: greens for what is typed, cyans
    /// for structure and values, no yellow. Bold cyan headings, bold green
    /// flags and subcommands, cyan value names, the program plain; cyan
    /// environment variables, green defaults, bright green possible values;
    /// bold red `error:`, red for what was typed wrong, green for what is
    /// missing.
    pub const DEFAULT: Palette = Palette {
        header: Paint::ansi(Ansi::Cyan).bold(),
        program: Paint::NONE,
        literal: Paint::ansi(Ansi::Green).bold(),
        placeholder: Paint::ansi(Ansi::Cyan),
        error: Paint::ansi(Ansi::Red).bold(),
        invalid: Paint::ansi(Ansi::Red),
        valid: Paint::ansi(Ansi::Green),
        env: Paint::ansi(Ansi::Cyan),
        default: Paint::ansi(Ansi::Green),
        choice: Paint::ansi(Ansi::BrightGreen),
    };

    /// [`Palette::DEFAULT`] in tamer tints of the 256-color palette, readable
    /// on dark and light backgrounds: bold teal 73 (`#5fafaf`) headings, bold
    /// green 71 (`#5faf5f`) flags and subcommands, slate teal 109 (`#87afaf`)
    /// value names; teal 37 (`#00afaf`) environment variables, sea green 72
    /// (`#5faf87`) defaults, green 71 possible values; bold red 167
    /// (`#d75f5f`) `error:`, rose 174 (`#d78787`) for what was typed wrong,
    /// sage 108 (`#87af87`) for what is missing.
    pub const DEFAULT_256: Palette = Palette {
        header: Paint::fg(Color::Ansi256(73)).bold(),
        program: Paint::NONE,
        literal: Paint::fg(Color::Ansi256(71)).bold(),
        placeholder: Paint::fg(Color::Ansi256(109)),
        error: Paint::fg(Color::Ansi256(167)).bold(),
        invalid: Paint::fg(Color::Ansi256(174)),
        valid: Paint::fg(Color::Ansi256(108)),
        env: Paint::fg(Color::Ansi256(37)),
        default: Paint::fg(Color::Ansi256(72)),
        choice: Paint::fg(Color::Ansi256(71)),
    };

    /// clap 4's colors: bold underlined headings, bold program and literals,
    /// plain value names and annotations; bold red
    /// `error:`, yellow for what was typed wrong, green for what is missing.
    pub const CLAP: Palette = Palette {
        header: Paint::NONE.bold().underline(),
        program: Paint::NONE.bold(),
        literal: Paint::NONE.bold(),
        placeholder: Paint::NONE,
        error: Paint::ansi(Ansi::Red).bold(),
        invalid: Paint::ansi(Ansi::Yellow),
        valid: Paint::ansi(Ansi::Green),
        env: Paint::NONE,
        default: Paint::NONE,
        choice: Paint::NONE,
    };

    /// The deepest color any role uses.
    pub const fn depth(&self) -> Depth {
        let roles = [
            self.header,
            self.program,
            self.literal,
            self.placeholder,
            self.error,
            self.invalid,
            self.valid,
            self.env,
            self.default,
            self.choice,
        ];
        let mut deepest = Depth::Ansi16;
        let mut i = 0;
        while i < roles.len() {
            let depth = roles[i].depth();
            if depth as u8 > deepest as u8 {
                deepest = depth;
            }
            i += 1;
        }
        deepest
    }
}

/// Palettes for each depth: the richest one a terminal can show is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Theme {
    /// For every color terminal; the fallback for the others.
    pub ansi16: Palette,
    /// For 256-color terminals, if different.
    pub ansi256: Option<Palette>,
    /// For 24-bit terminals, if different.
    pub truecolor: Option<Palette>,
}

impl Theme {
    /// [`Palette::DEFAULT`] on 16-color terminals, [`Palette::DEFAULT_256`] on
    /// 256-color and 24-bit ones.
    pub const DEFAULT: Theme = Theme {
        ansi16: Palette::DEFAULT,
        ansi256: Some(Palette::DEFAULT_256),
        truecolor: None,
    };

    /// One palette at every depth (its colors mapped down where deeper than
    /// the terminal).
    pub const fn uniform(palette: Palette) -> Theme {
        Theme {
            ansi16: palette,
            ansi256: None,
            truecolor: None,
        }
    }

    /// The palette for a terminal of `depth`.
    pub fn palette(&self, depth: Depth) -> Palette {
        match depth {
            Depth::None => Palette::PLAIN,
            Depth::Ansi16 => self.ansi16,
            Depth::Ansi256 => self.ansi256.unwrap_or(self.ansi16),
            Depth::TrueColor => self.truecolor.or(self.ansi256).unwrap_or(self.ansi16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_map_to_the_nearest_the_terminal_has() {
        let orange = Color::Rgb(255, 135, 0);
        assert_eq!(orange.at(Depth::TrueColor), Some(orange));
        assert_eq!(orange.at(Depth::Ansi256), Some(Color::Ansi256(208)));
        assert_eq!(orange.at(Depth::Ansi16), Some(Color::Ansi(Ansi::Yellow)));
        assert_eq!(orange.at(Depth::None), None);
        // Pure hues and grays land on their exact entries.
        assert_eq!(
            Color::Rgb(0, 255, 255).at(Depth::Ansi256),
            Some(Color::Ansi256(51))
        );
        assert_eq!(
            Color::Rgb(128, 128, 128).at(Depth::Ansi256),
            Some(Color::Ansi256(244))
        );
        assert_eq!(
            Color::Ansi256(196).at(Depth::Ansi16),
            Some(Color::Ansi(Ansi::BrightRed))
        );
        // A basic color is shown as itself everywhere.
        let cyan = Color::Ansi(Ansi::Cyan);
        assert_eq!(cyan.at(Depth::Ansi256), Some(cyan));
    }

    #[test]
    fn a_paint_is_one_sequence() {
        let mut out = String::new();
        Paint::ansi(Ansi::Yellow)
            .bold()
            .write(&mut out, Depth::Ansi16, "Usage:");
        assert_eq!(out, "\u{1b}[1;33mUsage:\u{1b}[0m");
        out.clear();
        Paint::fg(Color::Rgb(1, 2, 3))
            .underline()
            .write(&mut out, Depth::TrueColor, "x");
        assert_eq!(out, "\u{1b}[4;38;2;1;2;3mx\u{1b}[0m");
        out.clear();
        Paint::ansi(Ansi::BrightCyan).write(&mut out, Depth::Ansi16, "x");
        assert_eq!(out, "\u{1b}[96mx\u{1b}[0m");
        out.clear();
        Paint::ansi(Ansi::Red).write(&mut out, Depth::None, "x");
        assert_eq!(out, "x");
    }

    #[test]
    fn a_theme_uses_the_richest_palette_the_terminal_shows() {
        let rich = Palette {
            header: Paint::fg(Color::Rgb(255, 135, 0)),
            ..Palette::DEFAULT
        };
        let theme = Theme {
            truecolor: Some(rich),
            ..Theme::DEFAULT
        };
        assert_eq!(theme.palette(Depth::TrueColor), rich);
        assert_eq!(theme.palette(Depth::Ansi256), Palette::DEFAULT_256);
        assert_eq!(
            Theme::DEFAULT.palette(Depth::TrueColor),
            Palette::DEFAULT_256
        );
        assert_eq!(Theme::DEFAULT.palette(Depth::Ansi16), Palette::DEFAULT);
        assert_eq!(theme.palette(Depth::None), Palette::PLAIN);
        assert_eq!(rich.depth(), Depth::TrueColor);
        assert_eq!(Palette::DEFAULT.depth(), Depth::Ansi16);
        assert_eq!(Palette::DEFAULT_256.depth(), Depth::Ansi256);
    }
}
