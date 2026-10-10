//! man pages for a stitched command.
//!
//! Pages are [roff](https://docs.rs/roff) source. One page per command, with
//! the same sections as the Markdown renderer.

#![warn(missing_docs)]

use roff::{Inline, Roff, bold, italic, roman};
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
        if !body.is_empty() || command.unknown_flags_note().is_some() {
            doc.control("SH", ["DESCRIPTION"]);
            if !body.is_empty() {
                doc.text([roman(body)]);
            }
            if let Some(note) = command.unknown_flags_note() {
                if !body.is_empty() {
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
                item_tp(&mut doc, item, command.long_only);
            }
        }
        let builtins = command.builtins();
        if !grouped.options.is_empty() || !builtins.is_empty() {
            doc.control("SH", ["OPTIONS"]);
            for item in grouped.options {
                item_tp(&mut doc, item, command.long_only);
            }
            for (synopsis, blurb) in builtins {
                term(&mut doc, vec![bold(synopsis)], blurb);
            }
        }
        for (title, items) in grouped.headings {
            doc.control("SH", [arg(title).as_str()]);
            for item in items {
                item_tp(&mut doc, item, command.long_only);
            }
        }
        if !page.globals.is_empty() {
            doc.control("SH", ["GLOBAL OPTIONS"]);
            for global in &page.globals {
                item_tp(&mut doc, global.item, global.long_only);
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
        text.replace('"', "\\(dq")
    }
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

fn item_tp(doc: &mut Roff, item: &Item, long_only: bool) {
    term(doc, tag(item, long_only), &item.description());
    if let Some(vocabulary) = &item.vocabulary {
        // Indented, so that the next flag is not read as one more keyword.
        doc.control("RS", std::iter::empty::<&str>());
        doc.text([roman("Vocabulary:")]);
        for child in vocabulary.items.iter().filter(|item| !item.hide) {
            item_tp(doc, child, vocabulary.long_only);
        }
        doc.control("RE", std::iter::empty::<&str>());
    }
}

fn tag(item: &Item, long_only: bool) -> Vec<Inline> {
    let mut tag = vec![bold(item.names(long_only))];
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
