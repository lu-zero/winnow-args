//! man pages for a stitched command.
//!
//! Pages are [roff](https://docs.rs/roff) source. One page per command, with
//! the same sections as the Markdown renderer.

#![warn(missing_docs)]

use roff::{Inline, Roff};
use winnow_args_spec::{Command, Item, Page, Sub, pages};

/// Header fields of a manual page. They are not part of the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manual {
    /// Section number, usually `"1"`.
    pub section: String,
    /// The date in the center of the header. Empty leaves it out.
    pub date: String,
    /// The source in the left of the footer.
    pub source: String,
    /// The manual title in the center of the footer.
    pub manual: String,
}

impl Default for Manual {
    fn default() -> Self {
        Self {
            section: "1".to_owned(),
            date: String::new(),
            source: String::new(),
            manual: String::new(),
        }
    }
}

impl Manual {
    /// The day `seconds` after the Unix epoch falls on, in UTC, as a header
    /// shows it: `2026-10-10`. A reproducible build passes
    /// `SOURCE_DATE_EPOCH`.
    pub fn date_of(seconds: u64) -> String {
        // Days to a civil date, counted from a March that starts the year.
        let days = seconds / 86_400 + 719_468;
        let (era, day_of_era) = (days / 146_097, days % 146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month + 2) / 5 + 1;
        let (month, year) = if month < 10 {
            (month + 3, year_of_era + era * 400)
        } else {
            (month - 9, year_of_era + era * 400 + 1)
        };
        format!("{year:04}-{month:02}-{day:02}")
    }

    /// The root page and one page per visible subcommand.
    ///
    /// Each pair is `(file name, roff)`. `tool use` in section 1 is
    /// `tool-use.1`. Two paths that join to one name are an error.
    pub fn render_pages(
        &self,
        root: &Command,
        bin: &str,
    ) -> Result<Vec<(String, String)>, winnow_args_spec::Error> {
        Ok(pages(root, bin)?
            .iter()
            .map(|page| {
                let file = format!("{}.{}", page.stem(), self.section);
                (file, self.render(page))
            })
            .collect())
    }

    /// One page.
    pub fn render(&self, page: &Page<'_>) -> String {
        let command = page.command;
        let mut doc = Roff::new();
        let name = page.stem();
        // The title is in capitals by convention; NAME keeps the word as typed.
        let mut header = vec![arg(&name.to_uppercase()), arg(&self.section)];
        if !self.date.is_empty() || !self.source.is_empty() || !self.manual.is_empty() {
            header.extend([arg(&self.date), arg(&self.source), arg(&self.manual)]);
        }
        doc.control("TH", header.iter().map(String::as_str));
        doc.control("SH", ["NAME"]);
        let about = one_line(if command.about.is_empty() {
            command.long_about.as_str()
        } else {
            command.about.as_str()
        });
        if about.is_empty() {
            doc.text([bold(&name)]);
        } else {
            doc.text([bold(&name), roman(" - "), roman(&about)]);
        }

        doc.control("SH", ["SYNOPSIS"]);
        doc.text([roman(command.usage(&page.path))]);

        let body = command.prose();
        let notes = command.notes();
        if !body.is_empty() || !notes.is_empty() {
            doc.control("SH", ["DESCRIPTION"]);
            if !body.is_empty() {
                doc.text([roman(body)]);
            }
            for (index, note) in notes.into_iter().enumerate() {
                if index > 0 || !body.is_empty() {
                    doc.control("PP", std::iter::empty::<&str>());
                }
                doc.text([roman(note)]);
            }
        }

        let subs: Vec<&Sub> = command.subcommands.iter().filter(|sub| !sub.hide).collect();
        if !subs.is_empty() {
            doc.control("SH", ["COMMANDS"]);
            for sub in subs {
                let mut tag = vec![bold(&sub.name)];
                for alias in &sub.aliases {
                    tag.push(roman(", "));
                    tag.push(bold(alias));
                }
                term(&mut doc, tag, &sub.about);
            }
        }

        let grouped = command.grouped();
        if !grouped.arguments.is_empty() {
            doc.control("SH", ["ARGUMENTS"]);
            for item in grouped.arguments {
                item_tp(&mut doc, item);
            }
        }
        let builtins = command.builtins();
        if !grouped.options.is_empty() || !builtins.is_empty() {
            doc.control("SH", ["OPTIONS"]);
            for item in grouped.options {
                item_tp(&mut doc, item);
            }
            for (synopsis, blurb) in builtins {
                term(&mut doc, vec![bold(synopsis)], blurb);
            }
        }
        for (title, items) in grouped.headings {
            doc.control("SH", [arg(title).as_str()]);
            for item in items {
                item_tp(&mut doc, item);
            }
        }
        if !page.globals.is_empty() {
            doc.control("SH", ["GLOBAL OPTIONS"]);
            for item in &page.globals {
                item_tp(&mut doc, item);
            }
        }
        let after = command.after();
        // A .TP keeps following lines in that tagged paragraph until a paragraph macro.
        if !after.is_empty() {
            doc.control("PP", std::iter::empty::<&str>());
            doc.text([roman(after)]);
        }
        doc.render()
    }
}

/// An argument of a request. An empty one still takes its place, and a
/// quote would end the quoting `roff` adds around spaces.
fn arg(text: &str) -> String {
    if text.is_empty() {
        "\"\"".to_owned()
    } else {
        printable(text)
            .replace(['\n', '\t'], " ")
            .replace('\\', "\\e")
            .replace('"', "\\(dq")
    }
}

/// Text without the control characters a formatter refuses or a terminal
/// would act on: an escape or a bell in a default value. A tab is a space.
fn printable(text: &str) -> String {
    text.chars()
        .filter_map(|char| match char {
            '\n' => Some('\n'),
            '\t' => Some(' '),
            char if char.is_control() => None,
            char => Some(char),
        })
        .collect()
}

fn roman(text: impl AsRef<str>) -> Inline {
    roff::roman(printable(text.as_ref()))
}

fn bold(text: impl AsRef<str>) -> Inline {
    roff::bold(printable(text.as_ref()))
}

fn italic(text: impl AsRef<str>) -> Inline {
    roff::italic(printable(text.as_ref()))
}

fn one_line(text: &str) -> String {
    text.replace('\n', " ")
}

fn term(doc: &mut Roff, tag: Vec<Inline>, body: &str) {
    doc.control("TP", std::iter::empty::<&str>());
    doc.text(tag);
    if !body.is_empty() {
        doc.text([roman(body)]);
    }
}

fn item_tp(doc: &mut Roff, item: &Item) {
    term(doc, tag(item), &item.description());
    if let Some(vocabulary) = &item.vocabulary {
        // Indented, so that the next flag is not read as one more keyword.
        doc.control("RS", std::iter::empty::<&str>());
        doc.text([roman("Vocabulary:")]);
        for child in vocabulary.items.iter().filter(|item| !item.hide) {
            item_tp(doc, child);
        }
        doc.control("RE", std::iter::empty::<&str>());
    }
}

fn tag(item: &Item) -> Vec<Inline> {
    let mut tag = vec![bold(item.names())];
    if !item.positional {
        let suffix = item.value_suffix();
        if !suffix.is_empty() {
            tag.push(italic(suffix));
        }
    }
    tag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leading_dot_is_not_a_request_and_hyphens_are_escaped() {
        let file = Item {
            long: Some("file".to_owned()),
            help: ".so evil".to_owned(),
            ..Item::default()
        };
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            items: vec![file],
            ..Command::default()
        };
        let page = Manual::default().render(&pages(&command, "tool").unwrap()[0]);
        assert!(page.contains(".TH TOOL 1\n"), "{page}");
        assert!(
            page.contains("\\-\\-file") || page.contains("\\-file"),
            "{page}"
        );
        assert!(page.lines().all(|line| !line.starts_with(".so")), "{page}");
        assert!(page.contains("evil"), "{page}");
    }

    #[test]
    fn an_empty_header_field_keeps_its_place_and_a_quote_is_escaped() {
        let flag = Item {
            long: Some("file".to_owned()),
            heading: Some("Say \"hi\"".to_owned()),
            ..Item::default()
        };
        let command = Command {
            items: vec![flag],
            ..Command::default()
        };
        let manual = Manual {
            source: "tool 1.0".to_owned(),
            manual: "User Commands".to_owned(),
            ..Manual::default()
        };
        let page = manual.render(&pages(&command, "tool").unwrap()[0]);
        assert!(
            page.contains(".TH TOOL 1 \"\" \"tool 1.0\" \"User Commands\"\n"),
            "{page}"
        );
        assert!(page.contains(".SH \"Say \\(dqhi\\(dq\"\n"), "{page}");
    }

    #[test]
    fn a_date_is_the_utc_day_and_control_characters_are_dropped() {
        assert_eq!(Manual::date_of(0), "1970-01-01");
        assert_eq!(Manual::date_of(951_782_400), "2000-02-29");
        assert_eq!(Manual::date_of(951_868_799), "2000-02-29");
        assert_eq!(Manual::date_of(1_798_761_600), "2027-01-01");
        let flag = Item {
            long: Some("bell".to_owned()),
            help: "Ring\u{7} the\u{1b}[0m bell.\tLoudly.".to_owned(),
            ..Item::default()
        };
        let command = Command {
            items: vec![flag],
            ..Command::default()
        };
        let page = Manual::default().render(&pages(&command, "tool").unwrap()[0]);
        assert!(page.contains("Ring the[0m bell. Loudly.\n"), "{page}");
    }

    #[test]
    fn after_help_closes_the_tagged_paragraph() {
        let file = Item {
            long: Some("file".to_owned()),
            help: "A path.".to_owned(),
            ..Item::default()
        };
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            after_help: "See also.".to_owned(),
            items: vec![file],
            ..Command::default()
        };
        let page = Manual::default().render(&pages(&command, "tool").unwrap()[0]);
        assert!(
            page.contains(".TP\n\\fB\\-\\-file\\fR\nA path.\n.PP\nSee also.\n"),
            "{page}"
        );
    }
}
