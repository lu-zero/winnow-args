//! Markdown pages for a stitched command.
//!
//! One page per command. A subcommand is a link to its own page, and a
//! flattened command's items are already in the tree.

#![warn(missing_docs)]

use winnow_args_spec::{Command, Item, Page, Sub, pages};

/// The root page and one page per visible subcommand.
///
/// Each pair is `(file name, markdown)`. The file name joins the path with
/// `-`, so `tool use add` is `tool-use-add.md`. Two paths that join to one
/// name are an error.
pub fn render_pages(
    root: &Command,
    bin: &str,
) -> Result<Vec<(String, String)>, winnow_args_spec::Error> {
    Ok(pages(root, bin)?
        .iter()
        .map(|page| (format!("{}.md", page.stem()), render(page)))
        .collect())
}

/// One page.
pub fn render(page: &Page<'_>) -> String {
    let command = page.command;
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&page.path.join(" "));
    out.push_str("\n\n");
    let about = command.prose();
    if !about.is_empty() {
        out.push_str(&blocks(about));
        out.push_str("\n\n");
    }

    heading(&mut out, "Usage");
    out.push_str("```\n");
    out.push_str(&command.usage(&page.path));
    out.push_str("\n```\n\n");

    let subs: Vec<&Sub> = command.subcommands.iter().filter(|sub| !sub.hide).collect();
    if !subs.is_empty() {
        heading(&mut out, "Commands");
        for sub in subs {
            push_subcommand(&mut out, page, sub);
        }
        out.push('\n');
    }

    let grouped = command.grouped();
    if !grouped.arguments.is_empty() {
        heading(&mut out, "Arguments");
        for item in grouped.arguments {
            push_item(&mut out, item, "");
        }
        out.push('\n');
    }
    let builtins = command.builtins();
    if !grouped.options.is_empty() || !builtins.is_empty() {
        heading(&mut out, "Options");
        for item in grouped.options {
            push_item(&mut out, item, "");
        }
        for (synopsis, blurb) in builtins {
            push_term(&mut out, synopsis, blurb, "");
        }
        out.push('\n');
    }
    for (title, items) in grouped.headings {
        heading(&mut out, title);
        for item in items {
            push_item(&mut out, item, "");
        }
        out.push('\n');
    }
    if !page.globals.is_empty() {
        heading(&mut out, "Global options");
        for item in &page.globals {
            push_item(&mut out, item, "");
        }
        out.push('\n');
    }
    for note in command.notes() {
        out.push_str(note);
        out.push_str("\n\n");
    }
    let after = command.after();
    if !after.is_empty() {
        out.push_str(&blocks(after));
        out.push_str("\n\n");
    }
    finish(out)
}

fn heading(out: &mut String, title: &str) {
    out.push_str("## ");
    out.push_str(&escape(title));
    out.push_str("\n\n");
}

fn push_subcommand(out: &mut String, page: &Page<'_>, sub: &Sub) {
    out.push_str(&format!(
        "* [`{name}`]({stem}-{name}.md)",
        name = sub.name,
        stem = page.stem()
    ));
    for alias in &sub.aliases {
        out.push_str(", `");
        out.push_str(alias);
        out.push('`');
    }
    if !sub.about.is_empty() {
        out.push_str(" — ");
        out.push_str(&escape(&sub.about));
    }
    out.push('\n');
}

fn push_item(out: &mut String, item: &Item, indent: &str) {
    push_term(out, &item.synopsis(), &escape(&item.description()), indent);
    if let Some(vocabulary) = &item.vocabulary {
        out.push_str(indent);
        out.push_str("  Vocabulary:\n");
        let nested = format!("{indent}  ");
        for child in vocabulary.items.iter().filter(|item| !item.hide) {
            push_item(out, child, &nested);
        }
    }
}

fn push_term(out: &mut String, synopsis: &str, description: &str, indent: &str) {
    out.push_str(indent);
    out.push_str("* `");
    out.push_str(synopsis);
    out.push('`');
    if !description.is_empty() {
        out.push_str(" — ");
        out.push_str(&description.replace('\n', &format!("\n{indent}  ")));
    }
    out.push('\n');
}

/// The text of a page, outside its lists. A run of lines indented by four
/// spaces or a tab is kept as written, in a code fence: that is how an
/// example in after-help is laid out, and a paragraph would fold it into one
/// line. The other lines are escaped.
fn blocks(text: &str) -> String {
    let mut out = String::new();
    let mut fenced = false;
    for line in text.split('\n') {
        let kept = (line.starts_with("    ") || line.starts_with('\t')) && !line.trim().is_empty();
        if kept != fenced {
            out.push_str("```\n");
            fenced = kept;
        }
        if kept {
            out.push_str(line);
        } else {
            out.push_str(&escape(line));
        }
        out.push('\n');
    }
    if fenced {
        out.push_str("```\n");
    }
    out.pop();
    out
}

/// Escape Markdown outside paired backtick spans, and a `#` that would start
/// a heading.
fn escape(text: &str) -> String {
    let text = &text.replace("\n#", "\n\\#");
    let mut out = String::from(if text.starts_with('#') { "\\" } else { "" });
    let mut rest = text.as_str();
    while let Some(open) = rest.find('`') {
        let Some(len) = rest[open + 1..].find('`') else {
            break;
        };
        let close = open + 1 + len;
        push_escaped(&mut out, &rest[..open]);
        out.push_str(&rest[open..=close]);
        rest = &rest[close + 1..];
    }
    push_escaped(&mut out, rest);
    out
}

fn push_escaped(out: &mut String, text: &str) {
    for char in text.chars() {
        if matches!(char, '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '&') {
            out.push('\\');
        }
        out.push(char);
    }
}

fn finish(mut out: String) -> String {
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> Command {
        let verbose = Item {
            short: Some('v'),
            long: Some("verbose".to_owned()),
            help: "Say more.".to_owned(),
            global: true,
            ..Item::default()
        };
        let file = Item {
            long: Some("file".to_owned()),
            value_name: Some("PATH".to_owned()),
            help: "A path.".to_owned(),
            long_help: "A path.\n.TH is prose".to_owned(),
            ..Item::default()
        };
        let child = Command {
            name: "use".to_owned(),
            about: "Install *now*.".to_owned(),
            help_flag: true,
            ..Command::default()
        };
        Command {
            name: "tool".to_owned(),
            about: "Use *wild* and `a<b>` for <FILE> & co.".to_owned(),
            after_help: "Examples:\n    $ tool use <x>\n# not a heading".to_owned(),
            items: vec![verbose, file],
            subcommands: vec![winnow_args_spec::Sub {
                name: "use".to_owned(),
                aliases: vec!["u".to_owned()],
                about: "Install *now*.".to_owned(),
                hide: false,
                command: child,
            }],
            help_flag: true,
            help_short: true,
            unknown_flags_value: true,
            package_version: Some("1.2.3".to_owned()),
            ..Command::default()
        }
    }

    #[test]
    fn pages_escape_prose_and_link_subcommands() {
        let pages = render_pages(&command(), "tool").unwrap();
        assert_eq!(
            pages
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["tool.md", "tool-use.md"]
        );
        let root = &pages[0].1;
        assert!(
            root.contains("Use \\*wild\\* and `a<b>` for \\<FILE\\> \\& co."),
            "{root}"
        );
        assert!(
            root.contains("[`use`](tool-use.md), `u` — Install \\*now\\*."),
            "{root}"
        );
        assert!(
            root.contains("* `--file <PATH>` — A path.\n  .TH is prose"),
            "{root}"
        );
        assert!(
            root.contains("* `-h, --help` — Print help (see a summary with '-h')"),
            "{root}"
        );
        assert!(root.contains("* `-V, --version` — Print version"), "{root}");
        assert!(root.contains("names none is a positional value"), "{root}");
        assert!(
            root.ends_with("Examples:\n```\n    $ tool use <x>\n```\n\\# not a heading\n"),
            "{root}"
        );
        let child = &pages[1].1;
        assert!(
            child.contains("## Global options\n\n* `-v, --verbose` — Say more."),
            "{child}"
        );
        assert!(child.contains("* `--help` — Print help\n"), "{child}");
    }
}
