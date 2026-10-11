//! `examples/{brush_builtins,ld}.rs`: what the two ports declare,
//! in programs that only print what they parse.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use regex::Captures;

use crate::{Result, index_of, regex, root, rustfmt};

const BRUSH: &str = include_str!("../templates/brush_builtins.rs");
const LD: &str = include_str!("../templates/ld.rs");

/// The brush builtins shown, by source file.
const BUILTINS: &[&str] = &[
    "alias", "cd", "declare", "echo", "getopts", "hash", "history", "kill", "printf", "pwd",
    "read", "set", "shopt", "test", "type_", "umask", "unset",
];

pub(crate) fn generate(brush: &Path, mold: &Path) -> Result<()> {
    let examples = root().join("examples");
    let (builtins, ld) = (examples.join("brush_builtins.rs"), examples.join("ld.rs"));
    fs::write(&builtins, brush_builtins(brush)?)?;
    fs::write(&ld, self::ld(mold)?)?;
    rustfmt(&[&builtins, &ld])
}

fn brush_builtins(brush: &Path) -> Result<String> {
    let derive = regex(r"#\[derive\((Default, )?winnow_args::Args\)\]");
    let name_of = regex(r"struct (\w+)");
    let (mut structs, mut variants) = (Vec::new(), Vec::new());
    for name in BUILTINS {
        let source = fs::read_to_string(brush.join(format!("brush-builtins/src/{name}.rs")))?;
        for item in derived(&source) {
            let item = item.replace("pub(crate) struct", "struct");
            let item = derive.replace_all(&item, "#[derive(Debug, Args)]");
            // Operands brush fills in itself are plain words here.
            let item = item.replace(
                "    #[arg(skip)]\n    declarations: Vec<brush_core::CommandArg>,",
                "    /// Names and assignments.\n    #[arg(positional, stop_flags)]\n    \
                 declarations: Vec<String>,",
            );
            let ty = &name_of.captures(&item).ok_or("a struct without a name")?[1];
            if let Some(variant) = ty.strip_suffix("Command") {
                let about = item.lines().find_map(|line| line.strip_prefix("/// "));
                let doc = about.map_or(String::new(), |about| format!("    /// {about}\n"));
                let command = name.trim_end_matches('_');
                variants.push(format!(
                    "{doc}    #[arg(name = \"{command}\")]\n    {variant}({ty}),"
                ));
            }
            structs.push(item);
        }
    }
    Ok(BRUSH
        .replace("@VARIANTS@", &variants.join("\n"))
        .replace("@STRUCTS@", &structs.join("\n\n")))
}

/// The structs deriving `winnow_args::Args`, with their docs and attributes.
fn derived(source: &str) -> Vec<String> {
    let empty = regex(r"^(pub(\(crate\))? )?struct \w+ \{\}$");
    let lines: Vec<&str> = source.split('\n').collect();
    let mut items = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if line.starts_with("#[derive(") && line.contains("winnow_args::Args") {
            let docs = lines[..at]
                .iter()
                .rev()
                .take_while(|l| l.starts_with("///"));
            let first = at - docs.count();
            let body = lines[at..]
                .iter()
                .take_while(|l| **l != "}" && !empty.is_match(l));
            let last = at + body.count();
            items.push(lines[first..=last].join("\n"));
        }
    }
    items
}

fn ld(mold: &Path) -> Result<String> {
    let source = fs::read_to_string(mold.join("src/cmdline_winnow.rs"))?;
    let start = index_of(&source, "#[derive(Occurrence)]", "option enum")?;
    let end = index_of(
        &source,
        "/// The whole command line",
        "end of the option enum",
    )?;
    let item = format!("{}\n", source[start..end].trim_end());
    let item = regex(r#"\n    /// `--help`\.\n    #\[arg\(long = "help"\)\]\n    Help,"#)
        .replace_all(&item, "");
    let item = item
        .replace("#[derive(Occurrence)]", "#[derive(Occurrence, Debug)]")
        .replace("pub(crate) enum", "enum");

    let start = index_of(
        &source,
        "/// The option a `-z` keyword stands for",
        "`z_opt`",
    )?;
    let end = index_of(&source, "/// An option's value as UTF-8", "end of `z_opt`")?;
    let z_opt = source[start..end].trim_end().replace("pub(crate) fn", "fn") + "\n";

    let cmdline = fs::read_to_string(mold.join("src/cmdline.rs"))?;
    let helps = helps(&cmdline)?;
    let key = regex(r#"(?:short = '(.)'|long = "([^"]+)")"#);
    let mut unnamed = None;
    let item = regex(r"    /// (`-[^\n]*`)\n    #\[arg\(([^\n]*)\)\]").replace_all(
        &item,
        |found: &Captures<'_>| {
            let (spellings, attr) = (&found[1], &found[2]);
            let name = key.captures(attr).and_then(|k| k.get(1).or(k.get(2)));
            if name.is_none() {
                unnamed = Some(attr.to_owned());
            }
            let doc = name.and_then(|name| helps.get(name.as_str()));
            format!(
                "    /// {}\n    #[arg({attr})]",
                doc.map_or(spellings, |doc| doc)
            )
        },
    );
    if let Some(attr) = unnamed {
        return Err(format!("an option with no spelling: `#[arg({attr})]`").into());
    }
    Ok(LD.replace("@ENUM@", &item).replace("@ZOPT@", &z_opt))
}

/// mold's own help text, by option, where it has one.
fn helps(source: &str) -> Result<HashMap<&str, &str>> {
    let text = &source[index_of(source, "const HELP: &str = \"", "help text")?..];
    let text = &text[..index_of(text, "\";", "end of the help text")?];
    let option = regex(r"^  (-\S.*?)(?:\s{2,}(\S.*))?$");
    let continued = regex(r"^ {10,}\S");
    let lines: Vec<&str> = text.split('\n').collect();
    let mut helps = HashMap::new();
    for (at, line) in lines.iter().enumerate() {
        let Some(found) = option.captures(line) else {
            continue;
        };
        // An option too long for its line has its description on the next.
        let next = lines.get(at + 1).filter(|next| continued.is_match(next));
        let Some(doc) = found.get(2).map(|d| d.as_str()).or(next.map(|n| n.trim())) else {
            continue;
        };
        for spelling in found.get(1).expect("a group").as_str().split(", ") {
            let name = spelling.split([' ', '=', '[']).next().unwrap_or(spelling);
            helps.insert(name.trim_start_matches('-'), doc);
        }
    }
    Ok(helps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_structs_keep_their_docs() {
        let source = "use x;\n\n/// Doc.\n#[derive(winnow_args::Args)]\n#[arg(a)]\npub(crate) struct A {\n    x: bool,\n}\n\n#[derive(Debug)]\nstruct B {}\n\n#[derive(Default, winnow_args::Args)]\nstruct C {}\n";
        assert_eq!(
            derived(source),
            [
                "/// Doc.\n#[derive(winnow_args::Args)]\n#[arg(a)]\npub(crate) struct A {\n    x: bool,\n}",
                "#[derive(Default, winnow_args::Args)]\nstruct C {}",
            ]
        );
    }

    #[test]
    fn help_is_read_by_option() {
        let source = "const HELP: &str = \"Options:\n  -o FILE, --output FILE     Set output\n  --very-long-option-name=VALUE\n                              On the next line\n  --undocumented\n\";";
        let helps = helps(source).unwrap();
        assert_eq!(helps.get("o"), Some(&"Set output"));
        assert_eq!(helps.get("output"), Some(&"Set output"));
        assert_eq!(
            helps.get("very-long-option-name"),
            Some(&"On the next line")
        );
        assert_eq!(helps.get("undocumented"), None);
    }
}
