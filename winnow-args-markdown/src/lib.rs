#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use winnow_args_spec::marks::{Mark, pieces};
use winnow_args_spec::{Command, Item, Page, Sub, append_sentence, pages};

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
    let before = command.before();
    if !before.is_empty() {
        out.push_str(&blocks(before));
        out.push_str("\n\n");
    }
    let about = command.prose();
    if !about.is_empty() {
        out.push_str(&blocks(about));
        out.push_str("\n\n");
    }
    if let Some(effect) = command.effect {
        out.push_str(&format!("**Effect:** {}.\n\n", effect.label()));
    }

    heading(&mut out, "Usage");
    out.push_str("```\n");
    out.push_str(&command.usage(&page.path));
    out.push_str("\n```\n\n");

    let subs: Vec<&Sub> = command.subcommands.iter().filter(|sub| !sub.hide).collect();
    if !subs.is_empty() {
        heading(&mut out, "Commands");
        for sub in subs {
            push_subcommand(&mut out, page, sub, command.is_default(sub));
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
        if let Some(help) = command.heading_help(title) {
            out.push_str(&blocks(help));
            out.push_str("\n\n");
        }
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
    if !command.outputs.is_empty() {
        heading(&mut out, "Output formats");
        for output in &command.outputs {
            let mut what = escape(&output.help);
            if output.default {
                append_sentence(&mut what, "The default.");
            }
            if !output.framing.is_empty() {
                append_sentence(&mut what, &format!("Framing: {}.", code(&output.framing)));
            }
            push_term(&mut out, &output.name, &what, "");
        }
        out.push('\n');
    }
    if !command.exit_codes.is_empty() {
        heading(&mut out, "Exit status");
        for exit in &command.exit_codes {
            push_term(&mut out, &exit.code.to_string(), &escape(&exit.help), "");
        }
        out.push('\n');
    }
    if !command.examples.is_empty() {
        heading(&mut out, "Examples");
        for example in &command.examples {
            if !example.header.is_empty() {
                out.push_str(&format!("**{}**\n\n", escape_marked(&example.header, "")));
            }
            if !example.help.is_empty() {
                out.push_str(&blocks(&example.help));
                out.push_str("\n\n");
            }
            // A fence longer than any run of backticks in the line.
            let fence = "`".repeat(backticks(&example.code).max(2) + 1);
            out.push_str(&format!("{fence}\n$ {}\n{fence}\n\n", example.code));
        }
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

fn push_subcommand(out: &mut String, page: &Page<'_>, sub: &Sub, default: bool) {
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
    if default {
        out.push_str(" (default)");
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
    out.push_str("* ");
    out.push_str(&code(synopsis));
    if !description.is_empty() {
        out.push_str(" — ");
        // A blank line between paragraphs stays empty: no trailing spaces.
        for (n, line) in description.split('\n').enumerate() {
            if n > 0 {
                out.push('\n');
                if !line.is_empty() {
                    out.push_str(indent);
                    out.push_str("  ");
                }
            }
            out.push_str(line);
        }
    }
    out.push('\n');
}

/// The longest run of backticks in `text`.
fn backticks(text: &str) -> usize {
    let runs = text.split(|c| c != '`').map(str::len);
    runs.max().unwrap_or(0)
}

/// `text` as a code span: between more backticks than it has in a row, and
/// away from them where it begins or ends with one.
fn code(text: &str) -> String {
    let ticks = "`".repeat(backticks(text) + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{text}{pad}{ticks}")
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

/// Escape Markdown outside the spans that help reads, and a `#` that would
/// start a heading. A `` `code` `` span is kept, and an `*emphasized*` or
/// `**strong**` one is written as Markdown's own.
fn escape(text: &str) -> String {
    escape_marked(text, "**")
}

/// [`escape`], with `strong` around a strong span: empty where the text is
/// bold already.
fn escape_marked(text: &str, strong: &str) -> String {
    let text = &text.replace("\n#", "\n\\#");
    let mut out = String::from(if text.starts_with('#') { "\\" } else { "" });
    for piece in pieces(text) {
        let mark = match piece.mark {
            None => "",
            Some(Mark::Code) => {
                out.push('`');
                out.push_str(piece.text);
                out.push('`');
                continue;
            }
            Some(Mark::Emphasis) => "*",
            Some(Mark::Strong) => strong,
        };
        out.push_str(mark);
        push_escaped(&mut out, piece.text);
        out.push_str(mark);
    }
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
            long_help: "A path.\n.TH is prose\n\nAnother paragraph.".to_owned(),
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
            about: "Use *wild* and `a<b>` for <FILE> & 2*3 co.".to_owned(),
            after_help: "Examples:\n    $ tool use <x>\n# not a heading".to_owned(),
            items: vec![verbose, file],
            subcommands: vec![winnow_args_spec::Sub {
                name: "use".to_owned(),
                aliases: vec!["u".to_owned()],
                about: "Install *now*.".to_owned(),
                command: child,
                ..winnow_args_spec::Sub::default()
            }],
            help_flag: true,
            help_short: true,
            unknown_flags_value: true,
            package_version: Some("1.2.3".to_owned()),
            ..Command::default()
        }
    }

    #[test]
    fn external_commands_and_multicall_are_noted() {
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            external_subcommands: true,
            multicall: true,
            ..Command::default()
        };
        let page = &render_pages(&command, "tool").unwrap()[0].1;
        assert_eq!(command.notes().len(), 2);
        for note in command.notes() {
            assert!(page.contains(note), "{page}");
        }
    }

    #[test]
    fn before_help_leads_and_usage_lines_are_the_usage() {
        let command = Command {
            name: "tool".to_owned(),
            about: "A tool.".to_owned(),
            before_help: "A banner.".to_owned(),
            usage: "tool <SRC> <DST>\ntool --list".to_owned(),
            ..Command::default()
        };
        let page = &render_pages(&command, "tool").unwrap()[0];
        assert!(
            page.1.starts_with("# tool\n\nA banner.\n\nA tool.\n\n"),
            "{}",
            page.1
        );
        assert!(
            page.1.contains("```\ntool <SRC> <DST>\ntool --list\n```"),
            "{}",
            page.1
        );
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
            root.contains("Use *wild* and `a<b>` for \\<FILE\\> \\& 2\\*3 co."),
            "{root}"
        );
        assert!(
            root.contains("[`use`](tool-use.md), `u` — Install *now*."),
            "{root}"
        );
        assert!(
            root.contains("* `--file <PATH>` — A path.\n  .TH is prose\n\n  Another paragraph.\n"),
            "{root}"
        );
        assert!(
            root.contains("* `-h, --help` — Print help (see a summary with '-h')"),
            "{root}"
        );
        assert!(root.contains("* `-V, --version` — Print version"), "{root}");
        for (name, page) in &pages {
            let trailing = page.lines().find(|line| line.ends_with([' ', '\t']));
            assert_eq!(trailing, None, "a line of {name} ends in a space");
        }
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

    /// A command that says all a page says beyond its lists.
    fn telling() -> Command {
        use winnow_args_spec::{Effect, Example, ExitCode, Heading, Output, Sub};
        let wipe = Item {
            long: Some("wipe".to_owned()),
            help: "Start over.".to_owned(),
            heading: Some("Roots".to_owned()),
            warning: "Not for use.".to_owned(),
            effect: Some(Effect::Destructive),
            ..Item::default()
        };
        let sub = |name: &str, about: &str| Sub {
            name: name.to_owned(),
            aliases: Vec::new(),
            about: about.to_owned(),
            command: Command {
                name: name.to_owned(),
                about: about.to_owned(),
                ..Command::default()
            },
            ..Sub::default()
        };
        Command {
            name: "em".to_owned(),
            about: "A package manager.".to_owned(),
            items: vec![wipe],
            subcommands: vec![sub("emerge", "Merge."), sub("query", "Ask.")],
            default_subcommand: Some("emerge".to_owned()),
            effect: Some(Effect::Write),
            examples: vec![
                Example {
                    code: "em -ua @world".to_owned(),
                    header: "Update **all**".to_owned(),
                    help: "Rebuilds what changed.".to_owned(),
                },
                Example {
                    code: "em query".to_owned(),
                    ..Example::default()
                },
            ],
            headings: vec![Heading {
                title: "Roots".to_owned(),
                help: "Which tree is read.".to_owned(),
            }],
            outputs: vec![
                Output {
                    name: "pretty".to_owned(),
                    default: true,
                    help: "For people.".to_owned(),
                    ..Output::default()
                },
                Output {
                    name: "json".to_owned(),
                    framing: "json".to_owned(),
                    ..Output::default()
                },
            ],
            exit_codes: vec![ExitCode {
                code: 64,
                help: "No such root.".to_owned(),
            }],
            ..Command::default()
        }
    }

    #[test]
    fn backticks_in_code_do_not_end_it() {
        let mut command = telling();
        command.examples[0].code = "em run \"```\" `date`".to_owned();
        command.outputs[1].name = "a`b".to_owned();
        command.outputs[1].framing = "`x`".to_owned();
        let pages = render_pages(&command, "em").unwrap();
        let root = &pages[0].1;
        assert!(
            root.contains("````\n$ em run \"```\" `date`\n````\n"),
            "{root}"
        );
        assert!(root.contains("* ``a`b`` — Framing: `` `x` ``.\n"), "{root}");
        // Without any, a page is as it was.
        assert_eq!(code("--file <PATH>"), "`--file <PATH>`");
    }

    #[test]
    fn a_page_says_what_is_beyond_its_lists() {
        let pages = render_pages(&telling(), "em").unwrap();
        let root = &pages[0].1;
        assert!(
            root.starts_with(
                "# em\n\nA package manager.\n\n**Effect:** modifies state.\n\n## Usage\n"
            ),
            "{root}"
        );
        assert!(
            root.contains(
                "* [`emerge`](em-emerge.md) — Merge. (default)\n* [`query`](em-query.md) — Ask.\n"
            ),
            "{root}"
        );
        assert!(
            root.contains(
                "## Roots\n\nWhich tree is read.\n\n* `--wipe` — Start over. \
                 Effect: destructive (may delete or irreversibly overwrite). Warning: Not for use.\n"
            ),
            "{root}"
        );
        assert!(
            root.contains(
                "## Output formats\n\n* `pretty` — For people. The default.\n* `json` — Framing: `json`.\n"
            ),
            "{root}"
        );
        assert!(
            root.contains("## Exit status\n\n* `64` — No such root.\n"),
            "{root}"
        );
        assert!(
            root.ends_with(
                "## Examples\n\n**Update all**\n\nRebuilds what changed.\n\n```\n$ em -ua @world\n```\n\n```\n$ em query\n```\n"
            ),
            "{root}"
        );
    }
}
