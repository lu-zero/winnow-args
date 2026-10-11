//! One TOML fragment per derived type, when `WINNOW_ARGS_SPEC` names a directory.
//!
//! The derives expand to a constant that reads the variable, so Cargo
//! compiles the crate again when its value changes. The directory is an
//! absolute path, and one Cargo does not watch (`target/spec`).

use std::collections::HashMap;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::PoisonError;

use proc_macro2::Span;

/// Whether this compilation should write fragments.
pub(crate) fn enabled() -> bool {
    dir().is_some()
}

fn dir() -> Option<PathBuf> {
    let dir = std::env::var_os("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty())?;
    // An editor's macro server expands a type again at each edit, for no
    // build: with the variable in its environment it writes nothing.
    let editor = std::env::current_exe().is_ok_and(|exe| {
        exe.file_stem()
            .is_some_and(|name| name.to_string_lossy().starts_with("rust-analyzer"))
    });
    (!editor).then(|| PathBuf::from(dir))
}

#[derive(Default)]
pub(crate) struct ArgsDoc {
    pub ident: String,
    pub name: String,
    pub about: String,
    pub long_about: String,
    pub after_help: String,
    pub after_long_help: String,
    pub package_version: Option<String>,
    pub items: Vec<ItemDoc>,
    pub flatten: Vec<String>,
    pub sequence: Option<String>,
    pub subcommand: Option<String>,
    pub subcommand_required: bool,
    pub help_flag: bool,
    pub help_short: bool,
    pub long_only: bool,
    pub unknown_flags_value: bool,
}

#[derive(Default)]
pub(crate) struct VariantDoc {
    pub name: String,
    pub aliases: Vec<String>,
    pub about: String,
    /// The whole description of a variant with no type of its own.
    pub long_about: String,
    pub hide: bool,
    pub ty: Option<String>,
}

#[derive(Default)]
pub(crate) struct ItemDoc {
    pub short: Option<char>,
    pub more_shorts: Vec<char>,
    pub plus: Option<char>,
    pub long: Option<String>,
    pub aliases: Vec<String>,
    pub negate: Option<String>,
    pub value_name: Option<String>,
    pub help: String,
    pub long_help: String,
    pub heading: Option<String>,
    pub hide: bool,
    pub positional: bool,
    pub required: bool,
    pub multiple: bool,
    pub trailing: bool,
    pub default: Option<String>,
    pub env: Option<String>,
    pub choices: Vec<String>,
    pub choices_ty: Option<String>,
    pub require_equals: bool,
    pub global: bool,
    pub optional_value: bool,
    /// Zero and one are both one word, and are not written.
    pub values: usize,
    pub more_values: bool,
    pub two_dashes: bool,
    pub prefix: bool,
    pub stop_flags: bool,
    pub keywords: Option<String>,
}

pub(crate) fn write_args(span: Span, doc: &ArgsDoc) -> syn::Result<()> {
    write_at(span, &doc.ident, &args_toml(doc))
}

pub(crate) fn write_subcommands(
    span: Span,
    ident: &str,
    about: &str,
    long_about: &str,
    variants: &[VariantDoc],
) -> syn::Result<()> {
    let mut out = String::new();
    header(&mut out, "subcommands", ident);
    field(&mut out, "about", about);
    field(&mut out, "long_about", long_about);
    for variant in variants {
        out.push('\n');
        out.push_str("[[variant]]\n");
        field(&mut out, "name", &variant.name);
        strings(&mut out, "aliases", &variant.aliases);
        field(&mut out, "about", &variant.about);
        field(&mut out, "long_about", &variant.long_about);
        flag(&mut out, "hide", variant.hide);
        opt(&mut out, "ty", &variant.ty);
    }
    write_at(span, ident, &out)
}

pub(crate) fn write_occurrence(span: Span, ident: &str, items: &[ItemDoc]) -> syn::Result<()> {
    let mut out = String::new();
    header(&mut out, "occurrence", ident);
    for item in items {
        out.push('\n');
        out.push_str(&item_toml(item));
    }
    write_at(span, ident, &out)
}

pub(crate) fn write_choices(span: Span, ident: &str, choices: &[String]) -> syn::Result<()> {
    let mut out = String::new();
    header(&mut out, "choices", ident);
    strings(&mut out, "choices", choices);
    write_at(span, ident, &out)
}

fn args_toml(doc: &ArgsDoc) -> String {
    let mut out = String::new();
    header(&mut out, "args", &doc.ident);
    field(&mut out, "name", &doc.name);
    field(&mut out, "about", &doc.about);
    field(&mut out, "long_about", &doc.long_about);
    field(&mut out, "after_help", &doc.after_help);
    field(&mut out, "after_long_help", &doc.after_long_help);
    opt(&mut out, "package_version", &doc.package_version);
    strings(&mut out, "flatten", &doc.flatten);
    opt(&mut out, "sequence", &doc.sequence);
    opt(&mut out, "subcommand", &doc.subcommand);
    flag(&mut out, "subcommand_required", doc.subcommand_required);
    flag(&mut out, "help_flag", doc.help_flag);
    flag(&mut out, "help_short", doc.help_short);
    flag(&mut out, "long_only", doc.long_only);
    flag(&mut out, "unknown_flags_value", doc.unknown_flags_value);
    for item in &doc.items {
        out.push('\n');
        out.push_str(&item_toml(item));
    }
    out
}

fn item_toml(item: &ItemDoc) -> String {
    let mut out = String::from("[[item]]\n");
    opt_char(&mut out, "short", item.short);
    chars(&mut out, "more_shorts", &item.more_shorts);
    opt_char(&mut out, "plus", item.plus);
    opt(&mut out, "long", &item.long);
    strings(&mut out, "aliases", &item.aliases);
    opt(&mut out, "negate", &item.negate);
    opt(&mut out, "value_name", &item.value_name);
    field(&mut out, "help", &item.help);
    field(&mut out, "long_help", &item.long_help);
    opt(&mut out, "heading", &item.heading);
    flag(&mut out, "hide", item.hide);
    flag(&mut out, "positional", item.positional);
    flag(&mut out, "required", item.required);
    flag(&mut out, "multiple", item.multiple);
    flag(&mut out, "trailing", item.trailing);
    opt(&mut out, "default", &item.default);
    opt(&mut out, "env", &item.env);
    strings(&mut out, "choices", &item.choices);
    opt(&mut out, "choices_ty", &item.choices_ty);
    flag(&mut out, "require_equals", item.require_equals);
    flag(&mut out, "global", item.global);
    flag(&mut out, "optional_value", item.optional_value);
    if item.values > 1 {
        out.push_str("values = ");
        out.push_str(&item.values.to_string());
        out.push('\n');
    }
    flag(&mut out, "more_values", item.more_values);
    flag(&mut out, "two_dashes", item.two_dashes);
    flag(&mut out, "prefix", item.prefix);
    flag(&mut out, "stop_flags", item.stop_flags);
    opt(&mut out, "keywords", &item.keywords);
    out
}

fn header(out: &mut String, kind: &str, ident: &str) {
    out.push_str("version = 1\n");
    field(out, "kind", kind);
    field(out, "ident", ident);
}

/// Text that is absent when empty.
fn field(out: &mut String, key: &str, value: &str) {
    if !value.is_empty() {
        pair(out, key, value);
    }
}

/// Text that may be present and empty: `default = ""` is a default.
fn opt(out: &mut String, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        pair(out, key, value);
    }
}

fn pair(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&quote_basic(value));
    out.push('\n');
}

fn flag(out: &mut String, key: &str, value: bool) {
    if value {
        out.push_str(key);
        out.push_str(" = true\n");
    }
}

fn opt_char(out: &mut String, key: &str, value: Option<char>) {
    if let Some(value) = value {
        pair(out, key, &value.to_string());
    }
}

fn chars(out: &mut String, key: &str, values: &[char]) {
    let owned: Vec<String> = values.iter().map(char::to_string).collect();
    strings(out, key, &owned);
}

fn strings(out: &mut String, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    out.push_str(key);
    out.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&quote_basic(value));
    }
    out.push_str("]\n");
}

/// A TOML basic string, so `true`, `no` and `4` stay text.
fn quote_basic(text: &str) -> String {
    let mut out = String::from("\"");
    for char in text.chars() {
        match char {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            char if char.is_control() => {
                out.push_str(&format!("\\u{:04X}", u32::from(char)));
            }
            char => out.push(char),
        }
    }
    out.push('"');
    out
}

/// Fragments this process has written, by directory, source file and name.
/// A second body under one of them is a second type filed there.
static WRITTEN: Mutex<Option<HashMap<Filed, String>>> = Mutex::new(None);

/// A fragment's directory, source file and name.
type Filed = (String, String, String);

/// The source file of `span`, from the package's directory, with `/` between
/// its parts: two types of one name are told apart by it.
fn source(span: Span) -> String {
    let file = span.unwrap().local_file().unwrap_or_default();
    let file = normal(&std::path::absolute(&file).unwrap_or(file));
    let package = std::env::var_os("CARGO_MANIFEST_DIR").map(|dir| normal(Path::new(&dir)));
    // A file outside the package (`path = "../examples/x.rs"`) is told from
    // where the two part ways.
    let shared = package.map_or(0, |package| {
        let same = file.iter().zip(&package).take_while(|(a, b)| a == b);
        same.count().min(file.len().saturating_sub(1))
    });
    file[shared..].join("/")
}

/// The names in a path, `..` followed.
fn normal(path: &Path) -> Vec<String> {
    let mut parts = Vec::new();
    for part in path.components() {
        match part {
            Component::Normal(name) => parts.push(name.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts.pop();
            }
            _ => {}
        }
    }
    parts
}

fn write_at(span: Span, ident: &str, body: &str) -> syn::Result<()> {
    let Some(root) = dir() else {
        return Ok(());
    };
    let error = |message: String| syn::Error::new(span, message);
    if !root.is_absolute() {
        return Err(error(format!(
            "`WINNOW_ARGS_SPEC` is `{}`, which is not an absolute path",
            root.display()
        )));
    }
    if ident.is_empty() || ident.contains(['/', '\\']) {
        return Err(error(format!("`{ident}` is not a file name")));
    }
    // A library and a binary of one package have one crate name, and may each
    // have a type of one name: the binary's directory says which it is.
    let mut target = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "_".to_owned());
    if std::env::var_os("CARGO_BIN_NAME").is_some() {
        target.push_str("-bin");
    }
    let source = source(span);
    // With the other keys of the fragment, before its first table.
    let (version, rest) = body.split_once('\n').unwrap_or((body, ""));
    let mut body = format!("{version}\n");
    field(&mut body, "file", &source);
    body.push_str(rest);
    let key = (target.clone(), source.clone(), ident.to_owned());
    let mut written = WRITTEN.lock().unwrap_or_else(PoisonError::into_inner);
    match written.get_or_insert_default().insert(key, body.clone()) {
        Some(earlier) if earlier == body => return Ok(()),
        Some(_) => {
            return Err(error(format!(
                "two types of this file are filed under `{ident}`: give one another name for the documentation, as `#[arg(spec = \"Name\")]`, and state it again where a field or a variant holds that type"
            )));
        }
        None => {}
    }
    // One directory a source file: a type of the same name in another file
    // has its own.
    let dir = root.join(target).join(&source);
    std::fs::create_dir_all(&dir)
        .map_err(|source| error(format!("creating {}: {source}", dir.display())))?;
    // Another target of this crate may be writing the same file. A rename
    // replaces it whole, so a reader never sees half of one.
    let file = dir.join(format!("{ident}.toml"));
    let partial = dir.join(format!("{ident}.toml.{}", std::process::id()));
    std::fs::write(&partial, body)
        .and_then(|()| std::fs::rename(&partial, &file))
        .map_err(|source| {
            let _ = std::fs::remove_file(&partial);
            error(format!("writing {}: {source}", file.display()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_stay_text_and_empty_fields_are_omitted() {
        let item = ItemDoc {
            long: Some("ok".to_owned()),
            help: "true".to_owned(),
            choices: vec!["no".to_owned(), "4".to_owned()],
            ..ItemDoc::default()
        };
        let doc = ArgsDoc {
            ident: "Cli".to_owned(),
            about: "say \"hi\"\nnext".to_owned(),
            items: vec![item],
            ..ArgsDoc::default()
        };
        let text = args_toml(&doc);
        assert!(
            text.contains("about = \"say \\\"hi\\\"\\nnext\"\n"),
            "{text}"
        );
        assert!(text.contains("help = \"true\"\n"), "{text}");
        assert!(text.contains("choices = [\"no\", \"4\"]\n"), "{text}");
        assert!(!text.contains("help_flag"), "{text}");
        assert!(!text.contains("heading"), "{text}");
        assert!(!text.contains("name ="), "{text}");
        assert!(!text.contains("values ="), "{text}");
    }
}
