//! Command trees stitched from the derive's TOML fragments.
//!
//! Each derived type is one file. A flatten or a subcommand is the other
//! type's name, not a copy of its fields. [`Catalog::stitch`] follows those
//! names. Order is the order of the lists in the files: a command's own
//! items, then each flatten's resolved items, then its sequence; its own
//! subcommands, then each flatten's resolved subcommands.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;

use toml::Table;
use toml::Value;

/// A failure while reading or stitching fragments, or naming their pages.
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// One command after its flattens and subcommands have been followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Program or subcommand word. Empty when the fragment gave none.
    pub name: String,
    /// First paragraph of the description.
    pub about: String,
    /// The whole description.
    pub long_about: String,
    /// Text after the lists.
    pub after_help: String,
    /// Longer text after the lists.
    pub after_long_help: String,
    /// Flags and positionals, in the stitched order.
    pub items: Vec<Item>,
    /// Subcommands, in the stitched order.
    pub subcommands: Vec<Sub>,
    /// A subcommand word is required.
    pub subcommand_required: bool,
    /// `--help` is supplied.
    pub help_flag: bool,
    /// The supplied help flag also has `-h`.
    pub help_short: bool,
    /// A long name may also be spelled with one dash.
    pub long_only: bool,
    /// Unknown flag-like words are positional values.
    pub unknown_flags_value: bool,
    /// Text of `-V`/`--version`, when those flags are supplied.
    pub package_version: Option<String>,
}

/// A subcommand entry on its parent's page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sub {
    /// The word that selects it.
    pub name: String,
    /// Other words, shown beside the name.
    pub aliases: Vec<String>,
    /// The one-line description on the parent page.
    pub about: String,
    /// Left out of the rendered pages.
    pub hide: bool,
    /// The command itself.
    pub command: Command,
}

/// A flag or positional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// `-c`.
    pub short: Option<char>,
    /// Further short letters.
    pub more_shorts: Vec<char>,
    /// `+c`.
    pub plus: Option<char>,
    /// `--name`.
    pub long: Option<String>,
    /// Further long names.
    pub aliases: Vec<String>,
    /// `--no-name`, the spelling that sets the flag false.
    pub negate: Option<String>,
    /// Placeholder, without brackets.
    pub value_name: Option<String>,
    /// First paragraph of the description.
    pub help: String,
    /// The whole description.
    pub long_help: String,
    /// Grouping heading. Empty means the ordinary Options or Arguments list.
    pub heading: Option<String>,
    /// Left out of the rendered pages.
    pub hide: bool,
    /// A positional word rather than a flag.
    pub positional: bool,
    /// The invocation must supply it.
    pub required: bool,
    /// It repeats.
    pub multiple: bool,
    /// Only words after `--` fill it.
    pub trailing: bool,
    /// Shown default.
    pub default: Option<String>,
    /// Environment variable.
    pub env: Option<String>,
    /// The only values accepted.
    pub choices: Vec<String>,
    /// The value must be attached (`--name=value`).
    pub require_equals: bool,
    /// Also accepted on subcommands of this command.
    pub global: bool,
    /// A bare flag is a value (`--name` and `--name=value`).
    pub optional_value: bool,
    /// How many words one occurrence takes.
    pub values: usize,
    /// Further words are accepted while they are not flag-like.
    pub more_values: bool,
    /// Under `long_only`, this long name refuses a one-dash spelling.
    pub two_dashes: bool,
    /// A short letter takes the rest of its word.
    pub prefix: bool,
    /// Flags stop once this positional has a value.
    pub stop_flags: bool,
    /// The flags of a `-z` keyword vocabulary.
    pub vocabulary: Option<Box<Command>>,
}

/// A global flag inherited from an ancestor command.
#[derive(Debug)]
pub struct Global<'a> {
    /// The flag.
    pub item: &'a Item,
    /// `long_only` of the command that declared it.
    pub long_only: bool,
}

/// Visible flags and positionals in the order help lists them.
#[derive(Debug)]
pub struct Grouped<'a> {
    /// Positionals.
    pub arguments: Vec<&'a Item>,
    /// Flags with no heading of their own.
    pub options: Vec<&'a Item>,
    /// Custom headings, in the order they first appear.
    pub headings: Vec<(&'a str, Vec<&'a Item>)>,
}

/// One command in a walk of the tree, with the global flags of its parents.
#[derive(Debug)]
pub struct Page<'a> {
    /// Program name, then each subcommand word.
    pub path: Vec<String>,
    /// The command this page documents.
    pub command: &'a Command,
    /// Global flags inherited from parents, in the order they were declared.
    pub globals: Vec<Global<'a>>,
}

/// Fragments loaded from a directory or from strings, indexed by type name.
#[derive(Debug)]
pub struct Catalog {
    fragments: HashMap<String, Fragment>,
}

#[derive(Debug)]
enum Fragment {
    Args(Box<ArgsFrag>),
    Subcommands(Vec<VariantFrag>),
    Occurrence(Vec<ItemDraft>),
    Choices(Vec<String>),
}

#[derive(Debug)]
struct ArgsFrag {
    name: String,
    about: String,
    long_about: String,
    after_help: String,
    after_long_help: String,
    items: Vec<ItemDraft>,
    flatten: Vec<String>,
    sequence: Option<String>,
    subcommand: Option<String>,
    subcommand_required: bool,
    help_flag: bool,
    help_short: bool,
    long_only: bool,
    unknown_flags_value: bool,
    package_version: Option<String>,
}

#[derive(Debug)]
struct VariantFrag {
    name: String,
    aliases: Vec<String>,
    about: String,
    hide: bool,
    ty: Option<String>,
}

#[derive(Debug)]
struct ItemDraft {
    short: Option<char>,
    more_shorts: Vec<char>,
    plus: Option<char>,
    long: Option<String>,
    aliases: Vec<String>,
    negate: Option<String>,
    value_name: Option<String>,
    help: String,
    long_help: String,
    heading: Option<String>,
    hide: bool,
    positional: bool,
    required: bool,
    multiple: bool,
    trailing: bool,
    default: Option<String>,
    env: Option<String>,
    choices: Vec<String>,
    choices_ty: Option<String>,
    require_equals: bool,
    global: bool,
    optional_value: bool,
    values: usize,
    more_values: bool,
    two_dashes: bool,
    prefix: bool,
    stop_flags: bool,
    keywords: Option<String>,
}

impl Catalog {
    /// Every `*.toml` file under `dir`, including crate subdirectories.
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let mut files = Vec::new();
        collect(dir, &mut files)?;
        files.sort();
        let mut fragments = HashMap::new();
        for path in files {
            let text = fs::read_to_string(&path)
                .map_err(|error| Error::new(format!("reading {}: {error}", path.display())))?;
            let (ident, fragment) = parse_fragment(&text)
                .map_err(|error| Error::new(format!("{}: {error}", path.display())))?;
            insert(&mut fragments, ident, fragment)?;
        }
        Ok(Self { fragments })
    }

    /// Fragments already in memory. `name` is only used in errors.
    pub fn from_pairs<N, T>(pairs: impl IntoIterator<Item = (N, T)>) -> Result<Self, Error>
    where
        N: AsRef<str>,
        T: AsRef<str>,
    {
        let mut fragments = HashMap::new();
        for (name, text) in pairs {
            let (ident, fragment) = parse_fragment(text.as_ref())
                .map_err(|error| Error::new(format!("{}: {error}", name.as_ref())))?;
            insert(&mut fragments, ident, fragment)?;
        }
        Ok(Self { fragments })
    }

    /// The command named by the root type, with every link followed.
    pub fn stitch(&self, root: &str) -> Result<Command, Error> {
        let mut stack = Vec::new();
        self.command(root, &mut stack)
    }

    /// The command no other fragment names.
    ///
    /// Flatten, sequence, subcommand and keywords count as names. A
    /// documentation build has one such command, the program.
    pub fn root(&self) -> Result<String, Error> {
        let mut referenced = HashSet::new();
        for fragment in self.fragments.values() {
            match fragment {
                Fragment::Args(args) => {
                    referenced.extend(args.flatten.iter().cloned());
                    referenced.extend(args.sequence.iter().cloned());
                    referenced.extend(args.subcommand.iter().cloned());
                    referenced.extend(args.items.iter().filter_map(|item| item.keywords.clone()));
                }
                Fragment::Subcommands(variants) => {
                    referenced.extend(variants.iter().filter_map(|variant| variant.ty.clone()));
                }
                Fragment::Occurrence(_) | Fragment::Choices(_) => {}
            }
        }
        let mut roots: Vec<String> = self
            .fragments
            .iter()
            .filter(|(ident, fragment)| {
                matches!(fragment, Fragment::Args(_)) && !referenced.contains(*ident)
            })
            .map(|(ident, _)| ident.clone())
            .collect();
        roots.sort();
        match roots.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err(Error::new("no unreferenced command")),
            many => Err(Error::new(format!(
                "more than one unreferenced command: {}",
                many.join(", ")
            ))),
        }
    }
}

impl Command {
    /// `prog [OPTIONS] <FILE> [COMMAND]`.
    pub fn usage(&self, path: &[&str]) -> String {
        let mut line = path.join(" ");
        if self.items.iter().any(|item| !item.positional && !item.hide) || self.help_flag {
            line.push_str(" [OPTIONS]");
        }
        for item in self
            .items
            .iter()
            .filter(|item| item.positional && !item.hide && !item.trailing)
        {
            line.push(' ');
            line.push_str(&positional_placeholder(item));
        }
        if !self.subcommands.is_empty() {
            line.push_str(if self.subcommand_required {
                " <COMMAND>"
            } else {
                " [COMMAND]"
            });
        }
        for item in self
            .items
            .iter()
            .filter(|item| item.positional && !item.hide && item.trailing)
        {
            line.push_str(" [-- ");
            line.push_str(&positional_placeholder(item));
            line.push(']');
        }
        line
    }

    /// Visible items in help's section order.
    pub fn grouped(&self) -> Grouped<'_> {
        let visible: Vec<&Item> = self.items.iter().filter(|item| !item.hide).collect();
        let arguments = visible
            .iter()
            .copied()
            .filter(|item| item.positional)
            .collect();
        let mut options = Vec::new();
        let mut headings: Vec<(&str, Vec<&Item>)> = Vec::new();
        for item in visible.into_iter().filter(|item| !item.positional) {
            match item.heading.as_deref() {
                None => options.push(item),
                Some(heading) => match headings.iter_mut().find(|(name, _)| *name == heading) {
                    Some((_, rows)) => rows.push(item),
                    None => headings.push((heading, vec![item])),
                },
            }
        }
        Grouped {
            arguments,
            options,
            headings,
        }
    }

    /// The sentence for `unknown_flags = "value"`.
    pub fn unknown_flags_note(&self) -> Option<&'static str> {
        self.unknown_flags_value
            .then_some("A word that looks like a flag and names none is a positional value.")
    }
}

impl Item {
    /// Every spelling, in the order help prints them.
    ///
    /// Under `long_only`, a long name also contributes its one-dash form,
    /// unless the item refuses that form.
    pub fn terms(&self, long_only: bool) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(short) = self.short {
            names.push(format!("-{short}"));
        }
        for short in &self.more_shorts {
            names.push(format!("-{short}"));
        }
        if let Some(plus) = self.plus {
            names.push(format!("+{plus}"));
        }
        push_long(&mut names, self.long.as_deref(), long_only, self.two_dashes);
        for alias in &self.aliases {
            push_long(&mut names, Some(alias), long_only, self.two_dashes);
        }
        if let Some(negate) = &self.negate {
            names.push(format!("--{negate}"));
        }
        names
    }

    /// The value placeholder that follows the spellings, including its separator.
    pub fn value_suffix(&self) -> String {
        let Some(name) = self.value_name.as_deref() else {
            return if self.multiple && !self.positional {
                "...".to_owned()
            } else {
                String::new()
            };
        };
        let count = self.values.max(1);
        let mut value = vec![format!("<{name}>"); count].join(" ");
        if self.more_values {
            value.push_str("...");
        }
        let wrapped = match (self.require_equals, self.optional_value) {
            (true, true) => format!("[={value}]"),
            (true, false) => format!("={value}"),
            (false, true) => format!(" [{value}]"),
            (false, false) => format!(" {value}"),
        };
        if self.multiple {
            format!("{wrapped}...")
        } else {
            wrapped
        }
    }

    /// Description used on a long page: the long text, or the short one.
    pub fn prose(&self) -> &str {
        if self.long_help.is_empty() {
            &self.help
        } else {
            &self.long_help
        }
    }

    /// [`prose`](Self::prose) plus the notes help appends.
    pub fn description(&self) -> String {
        let mut text = self.prose().to_owned();
        if let Some(env) = &self.env {
            append_note(&mut text, &format!("[env: {env}]"));
        }
        if let Some(default) = &self.default {
            append_note(&mut text, &format!("[default: {default}]"));
        }
        if !self.choices.is_empty() {
            append_note(
                &mut text,
                &format!("[possible values: {}]", self.choices.join(", ")),
            );
        }
        if self.prefix {
            append_note(&mut text, "The rest of the word is the value.");
        }
        if self.stop_flags {
            append_note(&mut text, "Flags stop once this argument has a value.");
        }
        text
    }

    /// The left-hand spelling help prints for this item.
    pub fn synopsis(&self, long_only: bool) -> String {
        if self.positional {
            return positional_placeholder(self);
        }
        let mut names = Vec::new();
        if let Some(short) = self.short {
            names.push(format!("-{short}"));
        }
        for short in &self.more_shorts {
            names.push(format!("-{short}"));
        }
        if let Some(plus) = self.plus {
            names.push(format!("+{plus}"));
        }
        push_long(&mut names, self.long.as_deref(), long_only, self.two_dashes);
        for alias in &self.aliases {
            push_long(&mut names, Some(alias), long_only, self.two_dashes);
        }
        let mut line = names.join(", ");
        if let Some(negate) = &self.negate {
            if !line.is_empty() {
                line.push_str(" / ");
            }
            line.push_str("--");
            line.push_str(negate);
        }
        line.push_str(&self.value_suffix());
        line
    }
}

fn append_note(text: &mut String, note: &str) {
    if !text.is_empty() {
        text.push(if text.ends_with('\n') { '\n' } else { ' ' });
    }
    text.push_str(note);
}

/// The root page, then one page per visible subcommand, at every depth.
pub fn pages<'a>(root: &'a Command, bin: &str) -> Vec<Page<'a>> {
    let mut out = Vec::new();
    walk(root, vec![bin.to_owned()], &[], &mut out);
    out
}

/// Errors when two page paths join to one file name.
///
/// Renderers join the path with `-`. `tool use add` and `tool use-add` would
/// be the same file, and the second page would replace the first.
pub fn distinct_page_stems(pages: &[Page<'_>]) -> Result<(), Error> {
    let mut seen = HashSet::new();
    for page in pages {
        let stem = page.path.join("-");
        if !seen.insert(stem.clone()) {
            return Err(Error::new(format!("two pages would be named {stem}")));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Ancestor<'a> {
    item: &'a Item,
    long_only: bool,
}

fn walk<'a>(
    command: &'a Command,
    path: Vec<String>,
    ancestors: &[Ancestor<'a>],
    out: &mut Vec<Page<'a>>,
) {
    let globals = ancestors
        .iter()
        .filter(|ancestor| ancestor.item.global && !ancestor.item.hide && !ancestor.item.positional)
        .map(|ancestor| Global {
            item: ancestor.item,
            long_only: ancestor.long_only,
        })
        .collect();
    let mut next = ancestors.to_vec();
    next.extend(
        command
            .items
            .iter()
            .filter(|item| item.global)
            .map(|item| Ancestor {
                item,
                long_only: command.long_only,
            }),
    );
    out.push(Page {
        path: path.clone(),
        command,
        globals,
    });
    for sub in command.subcommands.iter().filter(|sub| !sub.hide) {
        let mut child = path.clone();
        child.push(sub.name.clone());
        walk(&sub.command, child, &next, out);
    }
}

fn push_long(names: &mut Vec<String>, long: Option<&str>, long_only: bool, two_dashes: bool) {
    let Some(long) = long else {
        return;
    };
    if long_only && !two_dashes {
        names.push(format!("-{long}"));
    }
    names.push(format!("--{long}"));
}

fn positional_placeholder(item: &Item) -> String {
    let name = item.value_name.as_deref().unwrap_or("VALUE");
    let mut text = if item.required {
        format!("<{name}>")
    } else {
        format!("[{name}]")
    };
    if item.multiple {
        text.push_str("...");
    }
    text
}

fn collect(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), Error> {
    let entries = fs::read_dir(dir)
        .map_err(|error| Error::new(format!("reading {}: {error}", dir.display())))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| Error::new(format!("reading {}: {error}", dir.display())))?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "toml") {
            files.push(path);
        }
    }
    Ok(())
}

fn insert(
    map: &mut HashMap<String, Fragment>,
    ident: String,
    fragment: Fragment,
) -> Result<(), Error> {
    if map.contains_key(&ident) {
        return Err(Error::new(format!("two fragments are named `{ident}`")));
    }
    map.insert(ident, fragment);
    Ok(())
}

fn parse_fragment(text: &str) -> Result<(String, Fragment), Error> {
    let table: Table = text
        .parse()
        .map_err(|error| Error::new(format!("not TOML: {error}")))?;
    let version = int(&table, "version")?.ok_or_else(|| Error::new("missing `version`"))?;
    if version != 1 {
        return Err(Error::new(format!("version {version} is not supported")));
    }
    let ident = req_str(&table, "ident")?;
    let kind = req_str(&table, "kind")?;
    let fragment = match kind.as_str() {
        "args" => Fragment::Args(Box::new(parse_args(&table)?)),
        "subcommands" => Fragment::Subcommands(parse_variants(&table)?),
        "occurrence" => Fragment::Occurrence(parse_items(&table)?),
        "choices" => Fragment::Choices(strings(&table, "choices")?),
        other => return Err(Error::new(format!("unknown kind `{other}`"))),
    };
    Ok((ident, fragment))
}

fn parse_args(table: &Table) -> Result<ArgsFrag, Error> {
    Ok(ArgsFrag {
        name: opt_str(table, "name")?,
        about: opt_str(table, "about")?,
        long_about: opt_str(table, "long_about")?,
        after_help: opt_str(table, "after_help")?,
        after_long_help: opt_str(table, "after_long_help")?,
        items: parse_items(table)?,
        flatten: strings(table, "flatten")?,
        sequence: opt_present(table, "sequence")?,
        subcommand: opt_present(table, "subcommand")?,
        subcommand_required: flag(table, "subcommand_required")?,
        help_flag: flag(table, "help_flag")?,
        help_short: flag(table, "help_short")?,
        long_only: flag(table, "long_only")?,
        unknown_flags_value: flag(table, "unknown_flags_value")?,
        package_version: opt_present(table, "package_version")?,
    })
}

fn parse_variants(table: &Table) -> Result<Vec<VariantFrag>, Error> {
    let mut variants = Vec::new();
    for value in array(table, "variant")? {
        let row = value
            .as_table()
            .ok_or_else(|| Error::new("`variant` entries are tables"))?;
        variants.push(VariantFrag {
            name: req_str(row, "name")?,
            aliases: strings(row, "aliases")?,
            about: opt_str(row, "about")?,
            hide: flag(row, "hide")?,
            ty: opt_present(row, "ty")?,
        });
    }
    Ok(variants)
}

fn parse_items(table: &Table) -> Result<Vec<ItemDraft>, Error> {
    let mut items = Vec::new();
    for value in array(table, "item")? {
        let row = value
            .as_table()
            .ok_or_else(|| Error::new("`item` entries are tables"))?;
        items.push(ItemDraft {
            short: one_char(row, "short")?,
            more_shorts: chars(row, "more_shorts")?,
            plus: one_char(row, "plus")?,
            long: opt_present(row, "long")?,
            aliases: strings(row, "aliases")?,
            negate: opt_present(row, "negate")?,
            value_name: opt_present(row, "value_name")?,
            help: opt_str(row, "help")?,
            long_help: opt_str(row, "long_help")?,
            heading: opt_present(row, "heading")?,
            hide: flag(row, "hide")?,
            positional: flag(row, "positional")?,
            required: flag(row, "required")?,
            multiple: flag(row, "multiple")?,
            trailing: flag(row, "trailing")?,
            default: opt_present(row, "default")?,
            env: opt_present(row, "env")?,
            choices: strings(row, "choices")?,
            choices_ty: opt_present(row, "choices_ty")?,
            require_equals: flag(row, "require_equals")?,
            global: flag(row, "global")?,
            optional_value: flag(row, "optional_value")?,
            values: value_count(row)?,
            more_values: flag(row, "more_values")?,
            two_dashes: flag(row, "two_dashes")?,
            prefix: flag(row, "prefix")?,
            stop_flags: flag(row, "stop_flags")?,
            keywords: opt_present(row, "keywords")?,
        });
    }
    Ok(items)
}

impl Catalog {
    fn command(&self, ident: &str, stack: &mut Vec<String>) -> Result<Command, Error> {
        if stack.iter().any(|name| name == ident) {
            return Err(Error::new(format!("`{ident}` refers to itself")));
        }
        let fragment = self
            .fragments
            .get(ident)
            .ok_or_else(|| Error::new(format!("no fragment named `{ident}`")))?;
        let Fragment::Args(args) = fragment else {
            return Err(Error::new(format!("`{ident}` is not a command")));
        };
        stack.push(ident.to_owned());
        let mut items = Vec::new();
        for draft in &args.items {
            items.push(self.item(draft, stack)?);
        }
        let mut subcommands = self.subs(args.subcommand.as_deref(), stack)?;
        for name in &args.flatten {
            let flattened = self.command(name, stack)?;
            items.extend(flattened.items);
            subcommands.extend(flattened.subcommands);
        }
        if let Some(sequence) = &args.sequence {
            for draft in self.occurrence(sequence)? {
                items.push(self.item(draft, stack)?);
            }
        }
        stack.pop();
        Ok(Command {
            name: args.name.clone(),
            about: args.about.clone(),
            long_about: args.long_about.clone(),
            after_help: args.after_help.clone(),
            after_long_help: args.after_long_help.clone(),
            items,
            subcommands,
            subcommand_required: args.subcommand_required,
            help_flag: args.help_flag,
            help_short: args.help_short,
            long_only: args.long_only,
            unknown_flags_value: args.unknown_flags_value,
            package_version: args.package_version.clone(),
        })
    }

    fn subs(&self, ident: Option<&str>, stack: &mut Vec<String>) -> Result<Vec<Sub>, Error> {
        let Some(ident) = ident else {
            return Ok(Vec::new());
        };
        let fragment = self
            .fragments
            .get(ident)
            .ok_or_else(|| Error::new(format!("no fragment named `{ident}`")))?;
        let Fragment::Subcommands(variants) = fragment else {
            return Err(Error::new(format!("`{ident}` is not a subcommand enum")));
        };
        let mut subs = Vec::new();
        for variant in variants {
            let mut command = match &variant.ty {
                Some(ty) => self.command(ty, stack)?,
                None => Command {
                    name: variant.name.clone(),
                    about: variant.about.clone(),
                    long_about: variant.about.clone(),
                    after_help: String::new(),
                    after_long_help: String::new(),
                    items: Vec::new(),
                    subcommands: Vec::new(),
                    subcommand_required: false,
                    help_flag: true,
                    help_short: true,
                    long_only: false,
                    unknown_flags_value: false,
                    package_version: None,
                },
            };
            if command.name.is_empty() {
                command.name = variant.name.clone();
            }
            let about = if variant.about.is_empty() {
                command.about.clone()
            } else {
                variant.about.clone()
            };
            subs.push(Sub {
                name: variant.name.clone(),
                aliases: variant.aliases.clone(),
                about,
                hide: variant.hide,
                command,
            });
        }
        Ok(subs)
    }

    fn occurrence(&self, ident: &str) -> Result<&[ItemDraft], Error> {
        match self.fragments.get(ident) {
            Some(Fragment::Occurrence(items)) => Ok(items),
            Some(_) => Err(Error::new(format!("`{ident}` is not an occurrence enum"))),
            None => Err(Error::new(format!("no fragment named `{ident}`"))),
        }
    }

    fn item(&self, draft: &ItemDraft, stack: &mut Vec<String>) -> Result<Item, Error> {
        let choices = if !draft.choices.is_empty() {
            draft.choices.clone()
        } else if let Some(ty) = &draft.choices_ty {
            self.choices(ty)?
        } else {
            Vec::new()
        };
        let vocabulary = match &draft.keywords {
            Some(ty) => Some(Box::new(self.command(ty, stack)?)),
            None => None,
        };
        Ok(Item {
            short: draft.short,
            more_shorts: draft.more_shorts.clone(),
            plus: draft.plus,
            long: draft.long.clone(),
            aliases: draft.aliases.clone(),
            negate: draft.negate.clone(),
            value_name: draft.value_name.clone(),
            help: draft.help.clone(),
            long_help: draft.long_help.clone(),
            heading: draft.heading.clone(),
            hide: draft.hide,
            positional: draft.positional,
            required: draft.required,
            multiple: draft.multiple,
            trailing: draft.trailing,
            default: draft.default.clone(),
            env: draft.env.clone(),
            choices,
            require_equals: draft.require_equals,
            global: draft.global,
            optional_value: draft.optional_value,
            values: draft.values,
            more_values: draft.more_values,
            two_dashes: draft.two_dashes,
            prefix: draft.prefix,
            stop_flags: draft.stop_flags,
            vocabulary,
        })
    }

    fn choices(&self, ident: &str) -> Result<Vec<String>, Error> {
        match self.fragments.get(ident) {
            Some(Fragment::Choices(choices)) => Ok(choices.clone()),
            Some(_) => Err(Error::new(format!("`{ident}` is not a value enum"))),
            // An open value (`String`, a number) has no fragment.
            None => Ok(Vec::new()),
        }
    }
}

fn req_str(table: &Table, key: &str) -> Result<String, Error> {
    match table.get(key) {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(_) => Err(Error::new(format!("`{key}` is not a string"))),
        None => Err(Error::new(format!("missing `{key}`"))),
    }
}

fn opt_str(table: &Table, key: &str) -> Result<String, Error> {
    Ok(opt_present(table, key)?.unwrap_or_default())
}

fn opt_present(table: &Table, key: &str) -> Result<Option<String>, Error> {
    match table.get(key) {
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(Error::new(format!("`{key}` is not a string"))),
        None => Ok(None),
    }
}

fn flag(table: &Table, key: &str) -> Result<bool, Error> {
    match table.get(key) {
        Some(Value::Boolean(value)) => Ok(*value),
        Some(_) => Err(Error::new(format!("`{key}` is not a boolean"))),
        None => Ok(false),
    }
}

fn value_count(table: &Table) -> Result<usize, Error> {
    match int(table, "values")? {
        None => Ok(1),
        Some(value) if value >= 1 => Ok(usize::try_from(value).unwrap_or(usize::MAX)),
        Some(value) => Err(Error::new(format!("`values` is {value}"))),
    }
}

fn int(table: &Table, key: &str) -> Result<Option<i64>, Error> {
    match table.get(key) {
        Some(Value::Integer(value)) => Ok(Some(*value)),
        Some(_) => Err(Error::new(format!("`{key}` is not an integer"))),
        None => Ok(None),
    }
}

fn strings(table: &Table, key: &str) -> Result<Vec<String>, Error> {
    let mut out = Vec::new();
    for value in array(table, key)? {
        let Some(text) = value.as_str() else {
            return Err(Error::new(format!("`{key}` holds a non-string")));
        };
        out.push(text.to_owned());
    }
    Ok(out)
}

fn chars(table: &Table, key: &str) -> Result<Vec<char>, Error> {
    strings(table, key)?
        .into_iter()
        .map(|text| one(&text, key))
        .collect()
}

fn one_char(table: &Table, key: &str) -> Result<Option<char>, Error> {
    opt_present(table, key)?
        .map(|text| one(&text, key))
        .transpose()
}

fn one(text: &str, key: &str) -> Result<char, Error> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(char), None) => Ok(char),
        _ => Err(Error::new(format!("`{key}` is not one character"))),
    }
}

fn array<'a>(table: &'a Table, key: &str) -> Result<&'a [Value], Error> {
    match table.get(key) {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(Error::new(format!("`{key}` is not an array"))),
        None => Ok(&[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stitched() -> Command {
        Catalog::from_pairs([
            (
                "Cli",
                r#"
                version = 1
                kind = "args"
                ident = "Cli"
                name = "tool"
                about = "A tool."
                subcommand = "Command"
                subcommand_required = true
                help_flag = true
                help_short = true
                flatten = ["Globals"]

                [[item]]
                positional = true
                value_name = "names"
                multiple = true
                help = "Names."
                "#,
            ),
            (
                "Globals",
                r#"
                version = 1
                kind = "args"
                ident = "Globals"

                [[item]]
                short = "v"
                long = "verbose"
                global = true
                help = "Verbose."
                "#,
            ),
            (
                "Command",
                r#"
                version = 1
                kind = "subcommands"
                ident = "Command"

                [[variant]]
                name = "use"
                aliases = ["u"]
                about = "Install."
                ty = "UseArgs"

                [[variant]]
                name = "true"
                about = "Success."
                "#,
            ),
            (
                "UseArgs",
                r#"
                version = 1
                kind = "args"
                ident = "UseArgs"
                subcommand = "UseMore"

                [[item]]
                long = "global"
                value_name = "VER"
                help = "A version."
                choices_ty = "When"
                "#,
            ),
            (
                "UseMore",
                r#"
                version = 1
                kind = "subcommands"
                ident = "UseMore"

                [[variant]]
                name = "add"
                ty = "AddArgs"
                "#,
            ),
            (
                "AddArgs",
                r#"
                version = 1
                kind = "args"
                ident = "AddArgs"
                about = "Add one."
                "#,
            ),
            (
                "When",
                r#"
                version = 1
                kind = "choices"
                ident = "When"
                choices = ["auto", "always"]
                "#,
            ),
        ])
        .unwrap()
        .stitch("Cli")
        .unwrap()
    }

    #[test]
    fn order_follows_own_items_then_flattens_then_nested_commands() {
        let command = stitched();
        let names: Vec<_> = command
            .items
            .iter()
            .map(|item| item.value_name.as_deref().or(item.long.as_deref()).unwrap())
            .collect();
        assert_eq!(names, ["names", "verbose"]);
        assert_eq!(command.subcommands.len(), 2);
        assert_eq!(command.subcommands[0].name, "use");
        assert_eq!(command.subcommands[0].aliases, ["u"]);
        let use_cmd = &command.subcommands[0].command;
        assert_eq!(use_cmd.items[0].choices, ["auto", "always"]);
        assert_eq!(use_cmd.subcommands[0].name, "add");
        assert_eq!(use_cmd.subcommands[0].command.about, "Add one.");
        assert!(command.subcommands[1].command.items.is_empty());
    }

    #[test]
    fn a_flatten_may_contribute_subcommands() {
        let command = Catalog::from_pairs([
            (
                "Parent",
                r#"
            version = 1
            kind = "args"
            ident = "Parent"
            flatten = ["Child"]
            "#,
            ),
            (
                "Child",
                r#"
            version = 1
            kind = "args"
            ident = "Child"
            subcommand = "Ops"
            "#,
            ),
            (
                "Ops",
                r#"
            version = 1
            kind = "subcommands"
            ident = "Ops"

            [[variant]]
            name = "run"
            "#,
            ),
        ])
        .unwrap()
        .stitch("Parent")
        .unwrap();
        assert_eq!(command.subcommands[0].name, "run");
    }

    #[test]
    fn pages_carry_parent_globals_in_declaration_order() {
        let command = stitched();
        let pages = pages(&command, "tool");
        assert_eq!(
            pages
                .iter()
                .map(|page| page.path.join(" "))
                .collect::<Vec<_>>(),
            ["tool", "tool use", "tool use add", "tool true"]
        );
        assert!(
            pages[1]
                .globals
                .iter()
                .any(|global| global.item.long.as_deref() == Some("verbose"))
        );
        assert!(pages[0].globals.is_empty());
        assert!(distinct_page_stems(&pages).is_ok());
    }

    #[test]
    fn hyphen_joined_paths_that_are_one_file_are_an_error() {
        let command = Catalog::from_pairs([
            (
                "Cli",
                r#"
                version = 1
                kind = "args"
                ident = "Cli"
                subcommand = "Ops"
                "#,
            ),
            (
                "Ops",
                r#"
                version = 1
                kind = "subcommands"
                ident = "Ops"

                [[variant]]
                name = "use"
                ty = "Use"

                [[variant]]
                name = "use-add"
                "#,
            ),
            (
                "Use",
                r#"
                version = 1
                kind = "args"
                ident = "Use"
                subcommand = "More"
                "#,
            ),
            (
                "More",
                r#"
                version = 1
                kind = "subcommands"
                ident = "More"

                [[variant]]
                name = "add"
                "#,
            ),
        ])
        .unwrap()
        .stitch("Cli")
        .unwrap();
        let error = distinct_page_stems(&pages(&command, "tool")).unwrap_err();
        assert!(error.to_string().contains("tool-use-add"), "{error}");
    }

    #[test]
    fn the_root_is_the_unreferenced_command() {
        let catalog = Catalog::from_pairs([
            (
                "Parent",
                r#"
                version = 1
                kind = "args"
                ident = "Parent"
                flatten = ["Child"]
                subcommand = "Ops"
                "#,
            ),
            (
                "Child",
                r#"
                version = 1
                kind = "args"
                ident = "Child"
                "#,
            ),
            (
                "Ops",
                r#"
                version = 1
                kind = "subcommands"
                ident = "Ops"

                [[variant]]
                name = "run"
                ty = "Child"
                "#,
            ),
        ])
        .unwrap();
        assert_eq!(catalog.root().unwrap(), "Parent");

        let two = Catalog::from_pairs([
            (
                "One",
                r#"
                version = 1
                kind = "args"
                ident = "One"
                "#,
            ),
            (
                "Two",
                r#"
                version = 1
                kind = "args"
                ident = "Two"
                "#,
            ),
        ])
        .unwrap();
        let error = two.root().unwrap_err();
        assert!(error.to_string().contains("One"), "{error}");
        assert!(error.to_string().contains("Two"), "{error}");
    }

    #[test]
    fn a_cycle_is_an_error_and_a_missing_vocabulary_is_too() {
        let catalog = Catalog::from_pairs([
            (
                "Parent",
                r#"
                version = 1
                kind = "args"
                ident = "Parent"
                flatten = ["Child"]
                "#,
            ),
            (
                "Child",
                r#"
                version = 1
                kind = "args"
                ident = "Child"
                flatten = ["Parent"]
                "#,
            ),
        ])
        .unwrap();
        let error = catalog.stitch("Parent").unwrap_err();
        assert!(error.to_string().contains("refers to itself"), "{error}");

        let missing = Catalog::from_pairs([(
            "Flag",
            r#"
            version = 1
            kind = "args"
            ident = "Flag"

            [[item]]
            long = "z"
            keywords = "Missing"
            "#,
        )])
        .unwrap();
        let error = missing.stitch("Flag").unwrap_err();
        assert!(
            error.to_string().contains("no fragment named `Missing`"),
            "{error}"
        );
    }

    #[test]
    fn synopsis_follows_help_and_an_open_choice_stays_empty() {
        let command = Catalog::from_pairs([(
            "Tool",
            r#"
            version = 1
            kind = "args"
            ident = "Tool"
            long_only = true
            package_version = "1.2.3"

            [[item]]
            short = "f"
            long = "force"
            negate = "no-force"
            value_name = "MODE"
            require_equals = true
            choices_ty = "String"
            help = "Force it."
            default = "on"
            prefix = true

            [[item]]
            positional = true
            required = true
            multiple = true
            value_name = "FILE"
            "#,
        )])
        .unwrap()
        .stitch("Tool")
        .unwrap();
        assert_eq!(command.package_version.as_deref(), Some("1.2.3"));
        assert_eq!(
            command.items[0].synopsis(command.long_only),
            "-f, -force, --force / --no-force=<MODE>"
        );
        assert!(command.items[0].choices.is_empty());
        assert_eq!(command.items[1].synopsis(false), "<FILE>...");
        assert_eq!(
            command.items[0].description(),
            "Force it. [default: on] The rest of the word is the value."
        );
    }
}
