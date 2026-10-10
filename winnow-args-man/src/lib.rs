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
        let pages = pages(root, bin);
        winnow_args_spec::distinct_page_stems(&pages)?;
        Ok(pages
            .iter()
            .map(|page| {
                let file = format!("{}.{}", page.path.join("-"), self.section);
                (file, self.render(page))
            })
            .collect())
    }

    /// One page.
    pub fn render(&self, page: &Page<'_>) -> String {
        let command = page.command;
        let mut doc = Roff::new();
        let name = page.path.join("-");
        let mut header = vec![name.as_str(), self.section.as_str()];
        if !self.date.is_empty() || !self.source.is_empty() || !self.manual.is_empty() {
            header.push(self.date.as_str());
            header.push(self.source.as_str());
            header.push(self.manual.as_str());
        }
        doc.control("TH", header);
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
        let words: Vec<&str> = page.path.iter().map(String::as_str).collect();
        doc.text([roman(command.usage(&words))]);

        let body = long_about(command);
        if !body.is_empty() || command.unknown_flags_note().is_some() {
            doc.control("SH", ["DESCRIPTION"]);
            if !body.is_empty() {
                lines(&mut doc, body);
            }
            if let Some(note) = command.unknown_flags_note() {
                doc.text([roman(note)]);
            }
        }

        let subs: Vec<&Sub> = command.subcommands.iter().filter(|sub| !sub.hide).collect();
        if !subs.is_empty() {
            doc.control("SH", ["COMMANDS"]);
            for sub in subs {
                doc.control("TP", std::iter::empty::<&str>());
                let mut tag = vec![bold(&sub.name)];
                for alias in &sub.aliases {
                    tag.push(roman(", "));
                    tag.push(bold(alias));
                }
                doc.text(tag);
                if !sub.about.is_empty() {
                    doc.text([roman(&sub.about)]);
                }
            }
        }

        let grouped = command.grouped();
        if !grouped.arguments.is_empty() {
            doc.control("SH", ["ARGUMENTS"]);
            for item in grouped.arguments {
                item_tp(&mut doc, item, command.long_only);
            }
        }
        if !grouped.options.is_empty() || command.help_flag || command.package_version.is_some() {
            doc.control("SH", ["OPTIONS"]);
            for item in grouped.options {
                item_tp(&mut doc, item, command.long_only);
            }
            if command.help_flag {
                let synopsis = if command.help_short {
                    "-h, --help"
                } else {
                    "--help"
                };
                let blurb = if command.help_short {
                    "Print help (see a summary with '-h')"
                } else {
                    "Print help"
                };
                term(&mut doc, &[bold(synopsis)], blurb);
            }
            if command.package_version.is_some() {
                term(&mut doc, &[bold("-V, --version")], "Print version");
            }
        }
        for (title, items) in grouped.headings {
            doc.control("SH", [title]);
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
        let after = if command.after_long_help.is_empty() {
            command.after_help.as_str()
        } else {
            command.after_long_help.as_str()
        };
        // A .TP keeps following lines in that tagged paragraph until a paragraph macro.
        if !after.is_empty() {
            doc.control("PP", std::iter::empty::<&str>());
            lines(&mut doc, after);
        }
        doc.render()
    }
}

fn long_about(command: &Command) -> &str {
    if command.long_about.is_empty() {
        command.about.as_str()
    } else {
        command.long_about.as_str()
    }
}

fn one_line(text: &str) -> String {
    text.replace('\n', " ")
}

fn lines(doc: &mut Roff, text: &str) {
    for line in text.split('\n') {
        doc.text([roman(line)]);
    }
}

fn term(doc: &mut Roff, tag: &[Inline], body: &str) {
    doc.control("TP", std::iter::empty::<&str>());
    doc.text(tag.to_vec());
    if !body.is_empty() {
        doc.text([roman(body)]);
    }
}

fn item_tp(doc: &mut Roff, item: &Item, long_only: bool) {
    term(doc, &tag(item, long_only), &item.description());
    if let Some(vocabulary) = &item.vocabulary {
        doc.text([roman("Vocabulary:")]);
        for child in vocabulary.items.iter().filter(|item| !item.hide) {
            item_tp(doc, child, vocabulary.long_only);
        }
    }
}

fn tag(item: &Item, long_only: bool) -> Vec<Inline> {
    let synopsis = item.synopsis(long_only);
    if item.positional {
        return vec![bold(synopsis)];
    }
    let suffix = item.value_suffix();
    if suffix.is_empty() {
        return vec![bold(synopsis)];
    }
    let head = synopsis
        .strip_suffix(&suffix)
        .unwrap_or(&synopsis)
        .to_owned();
    vec![bold(head), italic(suffix)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> Item {
        Item {
            short: None,
            more_shorts: Vec::new(),
            plus: None,
            long: None,
            aliases: Vec::new(),
            negate: None,
            value_name: None,
            help: String::new(),
            long_help: String::new(),
            heading: None,
            hide: false,
            positional: false,
            required: false,
            multiple: false,
            trailing: false,
            default: None,
            env: None,
            choices: Vec::new(),
            require_equals: false,
            global: false,
            optional_value: false,
            values: 1,
            more_values: false,
            two_dashes: false,
            prefix: false,
            stop_flags: false,
            vocabulary: None,
        }
    }

    #[test]
    fn a_leading_dot_is_not_a_request_and_hyphens_are_escaped() {
        let mut file = blank();
        file.long = Some("file".to_owned());
        file.help = ".so evil".to_owned();
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            long_about: String::new(),
            after_help: String::new(),
            after_long_help: String::new(),
            items: vec![file],
            subcommands: Vec::new(),
            subcommand_required: false,
            help_flag: false,
            help_short: false,
            long_only: false,
            unknown_flags_value: false,
            package_version: None,
        };
        let page = Manual::default().render(&pages(&command, "tool")[0]);
        assert!(page.contains(".TH tool 1\n"), "{page}");
        assert!(
            page.contains("\\-\\-file") || page.contains("\\-file"),
            "{page}"
        );
        assert!(page.lines().all(|line| !line.starts_with(".so")), "{page}");
        assert!(page.contains("evil"), "{page}");
    }

    #[test]
    fn after_help_closes_the_tagged_paragraph() {
        let mut file = blank();
        file.long = Some("file".to_owned());
        file.help = "A path.".to_owned();
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            long_about: String::new(),
            after_help: "See also.".to_owned(),
            after_long_help: String::new(),
            items: vec![file],
            subcommands: Vec::new(),
            subcommand_required: false,
            help_flag: false,
            help_short: false,
            long_only: false,
            unknown_flags_value: false,
            package_version: None,
        };
        let page = Manual::default().render(&pages(&command, "tool")[0]);
        assert!(
            page.contains(".TP\n\\fB\\-\\-file\\fR\nA path.\n.PP\nSee also.\n"),
            "{page}"
        );
    }
}
