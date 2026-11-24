//! The marks of prose: `` `code` ``, `*emphasis*` and `**strong**`, read as
//! `winnow_args::help` reads them, for the pages to render.
//!
//! A span does not cross a line. Stars open after a boundary and before a
//! non-space; the first later run of as many, after a word and before no word,
//! closes it. Quoted spans and runs of other lengths are skipped on the way.
//! A line indented four spaces or a tab stays as written.

/// What a span is marked as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Mark {
    /// `` `code` ``.
    Code,
    /// `*emphasis*`.
    Emphasis,
    /// `**strong**`.
    Strong,
}

/// A piece of prose: plain text, or a marked span's text without its marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece<'a> {
    /// The span's mark; `None` for plain text.
    pub mark: Option<Mark>,
    /// The text, without the marks.
    pub text: &'a str,
}

/// `text` cut into plain text and marked spans, in order.
pub fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let mut at = 0;
        // An indented line is a code block, not prose with markup.
        if !line.starts_with("    ") && !line.starts_with('\t') {
            while let Some((open, end, mark, len)) = span(line, at) {
                if start + open > plain {
                    out.push(Piece {
                        mark: None,
                        text: &text[plain..start + open],
                    });
                }
                out.push(Piece {
                    mark: Some(mark),
                    text: &line[open + len..end - len],
                });
                at = end;
                plain = start + end;
            }
        }
        start += line.len();
    }
    if plain < text.len() || out.is_empty() {
        out.push(Piece {
            mark: None,
            text: &text[plain..],
        });
    }
    out
}

/// The first span of `line` at or after `from`: where it opens and ends, its
/// mark, and how long the mark is on each side.
fn span(line: &str, from: usize) -> Option<(usize, usize, Mark, usize)> {
    let bytes = line.as_bytes();
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'`' => {
                if let Some(len) = line[at + 1..].find('`') {
                    return Some((at, at + len + 2, Mark::Code, 1));
                }
                at += 1;
            }
            b'*' => {
                let stars = run(bytes, at);
                let before = line[..at].chars().next_back();
                let after = line[at + stars..].chars().next();
                if stars <= 2
                    && can_open(before)
                    && after.is_some_and(|c| !c.is_whitespace() && c != '*')
                    && let Some(end) = closing(line, at + stars, stars)
                {
                    let mark = if stars == 2 {
                        Mark::Strong
                    } else {
                        Mark::Emphasis
                    };
                    return Some((at, end, mark, stars));
                }
                at += stars;
            }
            _ => at += 1,
        }
    }
    None
}

/// A star opens a span only after a boundary: shell globs and quoted ones
/// (`--include=*.rs`, `src/**/*.rs`, `'*.toml'`) are text, not markup.
fn can_open(before: Option<char>) -> bool {
    before.is_none_or(|c| c.is_whitespace() || matches!(c, '(' | '[' | '{' | '<' | '"'))
}

/// A star closes a span only after a word and before a boundary; a slash
/// after it starts a path, so `*/` is text too.
fn can_close(after: Option<char>) -> bool {
    after.is_none_or(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '>' | '"' | '\'' | '-'
            )
    })
}

/// How many stars are in a row at `at`.
fn run(bytes: &[u8], at: usize) -> usize {
    bytes[at..].iter().take_while(|b| **b == b'*').count()
}

/// The end of the span that `stars` stars opened before `from`: past the
/// next run of as many, if it closes one.
fn closing(line: &str, from: usize, stars: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'`' => at += line[at + 1..].find('`').map_or(1, |len| len + 2),
            b'*' => {
                let found = run(bytes, at);
                if found == stars {
                    let before = line[..at].chars().next_back();
                    let after = line[at + found..].chars().next();
                    let closes =
                        before.is_some_and(|c| !c.is_whitespace() && c != '\\') && can_close(after);
                    return closes.then_some(at + found);
                }
                at += found;
            }
            _ => at += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Mark, Piece, pieces};

    fn marks(text: &str) -> Vec<(Option<Mark>, &str)> {
        pieces(text).into_iter().map(|p| (p.mark, p.text)).collect()
    }

    #[test]
    fn spans_lose_their_marks() {
        assert_eq!(
            marks("Use `--force` to *really* do it, **now**."),
            [
                (None, "Use "),
                (Some(Mark::Code), "--force"),
                (None, " to "),
                (Some(Mark::Emphasis), "really"),
                (None, " do it, "),
                (Some(Mark::Strong), "now"),
                (None, "."),
            ]
        );
    }

    #[test]
    fn stars_that_open_no_span_are_text() {
        for text in [
            "2*3*4",
            "*.rs or *.toml",
            "--include=*.rs --exclude=*.toml",
            "src/**/*.rs",
            "'*.toml'",
            "* a bullet",
            "\\*not\\*",
            "***three***",
            "",
        ] {
            assert_eq!(marks(text), [(None, text)], "{text:?}");
        }
        assert_eq!(
            marks("*.rs is *never* read"),
            [
                (None, "*.rs is "),
                (Some(Mark::Emphasis), "never"),
                (None, " read")
            ]
        );
        assert_eq!(marks("*a `b*` c*"), [(Some(Mark::Emphasis), "a `b*` c")]);
    }

    #[test]
    fn agrees_with_the_terminal_help() {
        let shown =
            |text: &str| -> String { pieces(text).iter().map(|p: &Piece| p.text).collect() };
        for (text, expected) in [
            ("plain *emphasis* here", "plain emphasis here"),
            ("*two words* x", "two words x"),
            ("**strong**: x", "strong: x"),
            ("(*x*) y", "(x) y"),
            ("\"*quoted*\" x", "\"quoted\" x"),
            ("*word*-based", "word-based"),
            ("*this*/that", "*this*/that"),
            ("*note.* x", "note. x"),
            ("*a **b** c*", "a **b** c"),
            ("*.rs is *never* read", "*.rs is never read"),
            ("`*.rs and *.toml`", "*.rs and *.toml"),
        ] {
            assert_eq!(shown(text), expected, "{text:?}");
        }
        for text in [
            "2*3*4",
            "a*b*",
            "\\*not*",
            "***no***",
            "* a*",
            "*a *",
            "*open",
            "    *indented*",
            "\t*tabbed*",
        ] {
            assert_eq!(marks(text), [(None, text)], "{text:?}");
        }
    }

    #[test]
    fn a_span_does_not_cross_a_line() {
        assert_eq!(
            marks("*a\nb* `c`\n"),
            [(None, "*a\nb* "), (Some(Mark::Code), "c"), (None, "\n")]
        );
        let all: String = pieces("x *y*\nz").iter().map(|p: &Piece| p.text).collect();
        assert_eq!(all, "x y\nz");
    }
}
