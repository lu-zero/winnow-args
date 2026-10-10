//! One TOML fragment per derived type, when `WINNOW_ARGS_SPEC` names a directory.
//!
//! Cargo does not rebuild a crate because that variable changed, so a
//! documentation build starts from a clean compile. The directory should be
//! one Cargo does not watch (`target/spec`).

use proc_macro2::Span;

/// Whether this compilation should write fragments.
pub(crate) fn enabled() -> bool {
    std::env::var("WINNOW_ARGS_SPEC")
        .ok()
        .is_some_and(|dir| !dir.is_empty())
}

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

pub(crate) struct VariantDoc {
    pub name: String,
    pub aliases: Vec<String>,
    pub about: String,
    pub hide: bool,
    pub ty: Option<String>,
}

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
    pub values: usize,
    pub more_values: bool,
    pub two_dashes: bool,
    pub prefix: bool,
    pub stop_flags: bool,
    pub keywords: Option<String>,
}

impl Default for ItemDoc {
    fn default() -> Self {
        Self {
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
            choices_ty: None,
            require_equals: false,
            global: false,
            optional_value: false,
            values: 1,
            more_values: false,
            two_dashes: false,
            prefix: false,
            stop_flags: false,
            keywords: None,
        }
    }
}

pub(crate) fn write_args(span: Span, doc: &ArgsDoc) -> syn::Result<()> {
    write_at(span, &doc.ident, &args_toml(doc))
}

pub(crate) fn write_subcommands(
    span: Span,
    ident: &str,
    variants: &[VariantDoc],
) -> syn::Result<()> {
    let mut out = String::new();
    header(&mut out, "subcommands", ident);
    for variant in variants {
        out.push('\n');
        out.push_str("[[variant]]\n");
        field(&mut out, "name", &variant.name);
        strings(&mut out, "aliases", &variant.aliases);
        field(&mut out, "about", &variant.about);
        flag(&mut out, "hide", variant.hide);
        if let Some(ty) = &variant.ty {
            field(&mut out, "ty", ty);
        }
    }
    write_at(span, ident, &out)
}

pub(crate) fn write_items(
    span: Span,
    ident: &str,
    kind: &str,
    items: &[ItemDoc],
) -> syn::Result<()> {
    let mut out = String::new();
    header(&mut out, kind, ident);
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
    if let Some(version) = &doc.package_version {
        field(&mut out, "package_version", version);
    }
    strings(&mut out, "flatten", &doc.flatten);
    if let Some(sequence) = &doc.sequence {
        field(&mut out, "sequence", sequence);
    }
    if let Some(subcommand) = &doc.subcommand {
        field(&mut out, "subcommand", subcommand);
    }
    bool_always(&mut out, "subcommand_required", doc.subcommand_required);
    bool_always(&mut out, "help_flag", doc.help_flag);
    bool_always(&mut out, "help_short", doc.help_short);
    bool_always(&mut out, "long_only", doc.long_only);
    bool_always(&mut out, "unknown_flags_value", doc.unknown_flags_value);
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
    if let Some(long) = &item.long {
        field(&mut out, "long", long);
    }
    strings(&mut out, "aliases", &item.aliases);
    if let Some(negate) = &item.negate {
        field(&mut out, "negate", negate);
    }
    if let Some(value_name) = &item.value_name {
        field(&mut out, "value_name", value_name);
    }
    field(&mut out, "help", &item.help);
    field(&mut out, "long_help", &item.long_help);
    if let Some(heading) = &item.heading {
        field(&mut out, "heading", heading);
    }
    flag(&mut out, "hide", item.hide);
    flag(&mut out, "positional", item.positional);
    flag(&mut out, "required", item.required);
    flag(&mut out, "multiple", item.multiple);
    flag(&mut out, "trailing", item.trailing);
    if let Some(default) = &item.default {
        field(&mut out, "default", default);
    }
    if let Some(env) = &item.env {
        field(&mut out, "env", env);
    }
    strings(&mut out, "choices", &item.choices);
    if let Some(choices_ty) = &item.choices_ty {
        field(&mut out, "choices_ty", choices_ty);
    }
    flag(&mut out, "require_equals", item.require_equals);
    flag(&mut out, "global", item.global);
    flag(&mut out, "optional_value", item.optional_value);
    if item.values != 1 {
        out.push_str("values = ");
        out.push_str(&item.values.to_string());
        out.push('\n');
    }
    flag(&mut out, "more_values", item.more_values);
    flag(&mut out, "two_dashes", item.two_dashes);
    flag(&mut out, "prefix", item.prefix);
    flag(&mut out, "stop_flags", item.stop_flags);
    if let Some(keywords) = &item.keywords {
        field(&mut out, "keywords", keywords);
    }
    out
}

fn header(out: &mut String, kind: &str, ident: &str) {
    out.push_str("version = 1\n");
    field(out, "kind", kind);
    field(out, "ident", ident);
}

fn field(out: &mut String, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&quote_basic(value));
    out.push('\n');
}

fn flag(out: &mut String, key: &str, value: bool) {
    if value {
        bool_always(out, key, true);
    }
}

fn bool_always(out: &mut String, key: &str, value: bool) {
    out.push_str(key);
    out.push_str(if value { " = true\n" } else { " = false\n" });
}

fn opt_char(out: &mut String, key: &str, value: Option<char>) {
    if let Some(value) = value {
        field(out, key, &value.to_string());
    }
}

fn chars(out: &mut String, key: &str, values: &[char]) {
    if values.is_empty() {
        return;
    }
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

fn write_at(span: Span, ident: &str, body: &str) -> syn::Result<()> {
    let Ok(root) = std::env::var("WINNOW_ARGS_SPEC") else {
        return Ok(());
    };
    if root.is_empty() {
        return Ok(());
    }
    if ident.is_empty() || ident.contains(['/', '\\']) {
        return Err(syn::Error::new(
            span,
            format!("`{ident}` is not a file name"),
        ));
    }
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "_".to_owned());
    let dir = std::path::Path::new(&root).join(crate_name);
    std::fs::create_dir_all(&dir)
        .map_err(|error| syn::Error::new(span, format!("creating {}: {error}", dir.display())))?;
    let file = dir.join(format!("{ident}.toml"));
    if let Ok(existing) = std::fs::read_to_string(&file) {
        if existing != body {
            return Err(syn::Error::new(
                span,
                format!("two fragments are named `{ident}`"),
            ));
        }
        return Ok(());
    }
    std::fs::write(&file, body)
        .map_err(|error| syn::Error::new(span, format!("writing {}: {error}", file.display())))
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
            name: String::new(),
            about: "say \"hi\"\nnext".to_owned(),
            long_about: String::new(),
            after_help: String::new(),
            after_long_help: String::new(),
            package_version: None,
            items: vec![item],
            flatten: Vec::new(),
            sequence: None,
            subcommand: None,
            subcommand_required: false,
            help_flag: false,
            help_short: false,
            long_only: false,
            unknown_flags_value: false,
        };
        let text = args_toml(&doc);
        assert!(
            text.contains("about = \"say \\\"hi\\\"\\nnext\"\n"),
            "{text}"
        );
        assert!(text.contains("help = \"true\"\n"), "{text}");
        assert!(text.contains("choices = [\"no\", \"4\"]\n"), "{text}");
        assert!(text.contains("help_flag = false\n"), "{text}");
        assert!(!text.contains("heading"), "{text}");
        assert!(!text.contains("name ="), "{text}");
        assert!(!text.contains("values ="), "{text}");
    }
}
