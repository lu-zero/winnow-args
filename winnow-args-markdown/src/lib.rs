//! Markdown pages for a stitched command.
//!
//! One page per command. A subcommand is a link to its own page, and a
//! flattened command's items are already in the tree.

#![warn(missing_docs)]

use winnow_args_spec::{Command, Global, Item, Page, Sub, pages};

/// The root page and one page per visible subcommand.
///
/// Each pair is `(file name, markdown)`. The file name joins the path with
/// `-`, so `tool use add` is `tool-use-add.md`. Two paths that join to one
/// name are an error.
pub fn render_pages(
    root: &Command,
    bin: &str,
) -> Result<Vec<(String, String)>, winnow_args_spec::Error> {
    let pages = pages(root, bin);
    winnow_args_spec::distinct_page_stems(&pages)?;
    Ok(pages
        .iter()
        .map(|page| (file_name(&page.path), render(page)))
        .collect())
}

/// One page.
pub fn render(page: &Page<'_>) -> String {
    let command = page.command;
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&page.path.join(" "));
    out.push_str("\n\n");
    let about = long(command);
    if !about.is_empty() {
        out.push_str(&escape(about));
        out.push_str("\n\n");
    }

    heading(&mut out, "Usage");
    out.push_str("```\n");
    let words: Vec<&str> = page.path.iter().map(String::as_str).collect();
    out.push_str(&command.usage(&words));
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
            push_item(&mut out, item, command.long_only, "");
        }
        out.push('\n');
    }
    if !grouped.options.is_empty() || command.help_flag || command.package_version.is_some() {
        heading(&mut out, "Options");
        for item in grouped.options {
            push_item(&mut out, item, command.long_only, "");
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
            push_term(&mut out, synopsis, blurb, "");
        }
        if command.package_version.is_some() {
            push_term(&mut out, "-V, --version", "Print version", "");
        }
        out.push('\n');
    }
    for (title, items) in grouped.headings {
        heading(&mut out, title);
        for item in items {
            push_item(&mut out, item, command.long_only, "");
        }
        out.push('\n');
    }
    if !page.globals.is_empty() {
        heading(&mut out, "Global options");
        for global in &page.globals {
            push_global(&mut out, global);
        }
        out.push('\n');
    }
    if let Some(note) = command.unknown_flags_note() {
        out.push_str(note);
        out.push_str("\n\n");
    }
    let after = if command.after_long_help.is_empty() {
        command.after_help.as_str()
    } else {
        command.after_long_help.as_str()
    };
    if !after.is_empty() {
        out.push_str(&escape(after));
        out.push_str("\n\n");
    }
    finish(out)
}

fn long(command: &Command) -> &str {
    if command.long_about.is_empty() {
        command.about.as_str()
    } else {
        command.long_about.as_str()
    }
}

fn heading(out: &mut String, title: &str) {
    out.push_str("## ");
    out.push_str(&escape(title));
    out.push_str("\n\n");
}

fn push_subcommand(out: &mut String, page: &Page<'_>, sub: &Sub) {
    let mut path = page.path.clone();
    path.push(sub.name.clone());
    out.push_str("* [`");
    out.push_str(&sub.name);
    out.push_str("`](");
    out.push_str(&file_name(&path));
    out.push(')');
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

fn push_global(out: &mut String, global: &Global<'_>) {
    push_item(out, global.item, global.long_only, "");
}

fn push_item(out: &mut String, item: &Item, long_only: bool, indent: &str) {
    push_term(
        out,
        &item.synopsis(long_only),
        &escape(&item.description()),
        indent,
    );
    if let Some(vocabulary) = &item.vocabulary {
        out.push_str(indent);
        out.push_str("  Vocabulary:\n");
        let nested = format!("{indent}  ");
        for child in vocabulary.items.iter().filter(|item| !item.hide) {
            push_item(out, child, vocabulary.long_only, &nested);
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
        let continuation = format!("{indent}  ");
        for (index, line) in description.split('\n').enumerate() {
            if index > 0 {
                out.push('\n');
                out.push_str(&continuation);
            }
            out.push_str(line);
        }
    }
    out.push('\n');
}

fn file_name(path: &[String]) -> String {
    format!("{}.md", path.join("-"))
}

/// Escape Markdown outside paired backtick spans.
fn escape(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
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
    use winnow_args_spec::Item;

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

    fn command() -> Command {
        let mut verbose = blank();
        verbose.short = Some('v');
        verbose.long = Some("verbose".to_owned());
        verbose.help = "Say more.".to_owned();
        verbose.global = true;
        let mut file = blank();
        file.long = Some("file".to_owned());
        file.value_name = Some("PATH".to_owned());
        file.help = "A path.".to_owned();
        file.long_help = "A path.\n.TH is prose".to_owned();
        let child = Command {
            name: "use".to_owned(),
            about: "Install *now*.".to_owned(),
            long_about: String::new(),
            after_help: String::new(),
            after_long_help: String::new(),
            items: Vec::new(),
            subcommands: Vec::new(),
            subcommand_required: false,
            help_flag: true,
            help_short: false,
            long_only: false,
            unknown_flags_value: false,
            package_version: None,
        };
        Command {
            name: "tool".to_owned(),
            about: "Use *wild* and `a<b>` for <FILE> & co.".to_owned(),
            long_about: String::new(),
            after_help: String::new(),
            after_long_help: String::new(),
            items: vec![verbose, file],
            subcommands: vec![winnow_args_spec::Sub {
                name: "use".to_owned(),
                aliases: vec!["u".to_owned()],
                about: "Install *now*.".to_owned(),
                hide: false,
                command: child,
            }],
            subcommand_required: false,
            help_flag: true,
            help_short: true,
            long_only: false,
            unknown_flags_value: true,
            package_version: Some("1.2.3".to_owned()),
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
        let child = &pages[1].1;
        assert!(
            child.contains("## Global options\n\n* `-v, --verbose` — Say more."),
            "{child}"
        );
        assert!(child.contains("* `--help` — Print help\n"), "{child}");
    }
}
