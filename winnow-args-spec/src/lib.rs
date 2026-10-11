//! Command trees stitched from the derive's TOML fragments.
//!
//! Each derived type is one file, in the directory of its crate and of its
//! source file. A flatten or a subcommand is the other type as the source
//! writes it, `add::Opts`, not a copy of its fields. [`Catalog::stitch`]
//! follows those names: to the type of that name in the same crate, else to
//! the only one of that name in the others. Where several types have the
//! name, the modules written before it say which file's is meant, and a bare
//! name is the type of the same file.
//!
//! Order is the order of the lists in the files: a command's own items, then
//! each flatten's resolved items, then its sequence; its own subcommands,
//! then each flatten's resolved subcommands.

use std::collections::BTreeMap;
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    /// `-c`.
    pub short: Option<char>,
    /// Further short letters.
    pub more_shorts: Vec<char>,
    /// `+c`.
    pub plus: Option<char>,
    /// `--name`.
    pub long: Option<String>,
    /// Further long names. Help does not show them, and neither do the pages.
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
    /// How many words one occurrence takes. Zero is read as one.
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
    pub globals: Vec<&'a Item>,
}

/// Fragments loaded from a directory or from strings, by crate.
#[derive(Debug, Default)]
pub struct Catalog {
    /// Fragments given without a crate are under `""`.
    crates: BTreeMap<String, Vec<Entry>>,
}

/// One type: the name it is filed under, and the source file it is in.
#[derive(Debug)]
struct Entry {
    ident: String,
    /// From the package's directory, `/` between its parts. Empty for a
    /// fragment written by hand that does not say.
    file: String,
    fragment: Fragment,
}

/// A fragment found, and its crate.
type Found<'a> = (&'a str, &'a Entry);

/// Where a name is read: the crate and the source file of the type that
/// holds it. Both are empty for a name given from outside.
#[derive(Clone, Copy)]
struct At<'a> {
    krate: &'a str,
    file: &'a str,
}

const OUTSIDE: At<'static> = At {
    krate: "",
    file: "",
};

#[derive(Debug)]
enum Fragment {
    Args(Box<ArgsFrag>),
    Subcommands(SubsFrag),
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
struct SubsFrag {
    about: String,
    long_about: String,
    variants: Vec<VariantFrag>,
}

#[derive(Debug)]
struct VariantFrag {
    name: String,
    aliases: Vec<String>,
    about: String,
    long_about: String,
    hide: bool,
    ty: Option<String>,
}

#[derive(Debug)]
struct ItemDraft {
    item: Item,
    choices_ty: Option<String>,
    keywords: Option<String>,
}

impl Catalog {
    /// Every `*.toml` file under `dir`. A subdirectory is a crate, as the
    /// derive writes one, and a file directly in `dir` has none. A fragment
    /// says which source file its type is in; the directories below a crate's
    /// only keep two of one name apart.
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let mut files = Vec::new();
        collect(dir, &mut files)?;
        files.sort();
        let mut catalog = Self::default();
        for path in files {
            let text = fs::read_to_string(&path)
                .map_err(|error| Error::new(format!("reading {}: {error}", path.display())))?;
            let mut inside = path.strip_prefix(dir).unwrap_or(&path).components();
            let krate = match (inside.next(), inside.next()) {
                (Some(first), Some(_)) => first.as_os_str().to_string_lossy().into_owned(),
                _ => String::new(),
            };
            catalog.insert(&krate, &path.display().to_string(), &text)?;
        }
        Ok(catalog)
    }

    /// Fragments already in memory, of no crate. `name` is only used in errors.
    pub fn from_pairs<N, T>(pairs: impl IntoIterator<Item = (N, T)>) -> Result<Self, Error>
    where
        N: AsRef<str>,
        T: AsRef<str>,
    {
        let mut catalog = Self::default();
        for (name, text) in pairs {
            catalog.insert("", name.as_ref(), text.as_ref())?;
        }
        Ok(catalog)
    }

    fn insert(&mut self, krate: &str, name: &str, text: &str) -> Result<(), Error> {
        let entry = parse_fragment(text).map_err(|error| Error::new(format!("{name}: {error}")))?;
        let entries = self.crates.entry(krate.to_owned()).or_default();
        let twice = |other: &Entry| other.ident == entry.ident && other.file == entry.file;
        if entries.iter().any(twice) {
            return Err(Error::new(format!(
                "{name}: a second fragment named `{}`",
                entry.ident
            )));
        }
        entries.push(entry);
        Ok(())
    }

    /// The command a type names, with every link followed.
    ///
    /// `root` is a type's name. When several types have it, its module or
    /// its crate tells them apart: `add::Opts`, `my_crate::Opts`.
    pub fn stitch(&self, root: &str) -> Result<Command, Error> {
        let mut stack = Vec::new();
        self.command(OUTSIDE, root, &mut stack)
    }

    /// The command no other fragment names.
    ///
    /// A documentation build of a program has one such command, the program.
    /// A crate with several is asked for its [`roots`](Self::roots).
    pub fn root(&self) -> Result<String, Error> {
        let mut roots = self.roots();
        match roots.len() {
            1 => Ok(roots.remove(0)),
            0 => Err(Error::new("no unreferenced command")),
            _ => Err(Error::new(format!(
                "more than one unreferenced command: {}",
                roots.join(", ")
            ))),
        }
    }

    /// Every command no other fragment names, sorted, each as
    /// [`stitch`](Self::stitch) takes it.
    ///
    /// Flatten, sequence, subcommand and keywords count as names. A subcommand
    /// enum nothing names is a command: a program may be one.
    pub fn roots(&self) -> Vec<String> {
        let mut referenced = HashSet::new();
        for (krate, entries) in &self.crates {
            for entry in entries {
                let at = At {
                    krate,
                    file: &entry.file,
                };
                let links: Vec<(&String, Kind)> = match &entry.fragment {
                    Fragment::Args(args) => args
                        .flatten
                        .iter()
                        .map(|name| (name, COMMAND))
                        .chain(args.sequence.iter().map(|name| (name, OCCURRENCE)))
                        .chain(args.subcommand.iter().map(|name| (name, SUBCOMMANDS)))
                        .chain(
                            args.items
                                .iter()
                                .filter_map(|item| Some((item.keywords.as_ref()?, COMMAND))),
                        )
                        .collect(),
                    Fragment::Subcommands(subs) => subs
                        .variants
                        .iter()
                        .filter_map(|variant| Some((variant.ty.as_ref()?, COMMAND)))
                        .collect(),
                    Fragment::Occurrence(_) | Fragment::Choices(_) => Vec::new(),
                };
                for (name, kind) in links {
                    if let Ok(Some((_, to))) = self.find(at, name, kind) {
                        referenced.insert(std::ptr::from_ref(to));
                    }
                }
            }
        }
        let mut roots = Vec::new();
        for (krate, entries) in &self.crates {
            for entry in entries {
                if !COMMAND(&entry.fragment) || referenced.contains(&std::ptr::from_ref(entry)) {
                    continue;
                }
                // The shortest of its names that finds it.
                let whole = whole_name(krate, entry);
                let names = [entry.ident.clone(), format!("{krate}::{}", entry.ident)];
                let found = |name: &String| matches!(self.find(OUTSIDE, name, COMMAND), Ok(Some((_, to))) if std::ptr::eq(to, entry));
                roots.push(names.into_iter().find(found).unwrap_or(whole));
            }
        }
        roots.sort();
        roots
    }
}

/// Which fragments a name may be: a flatten is not a list of choices.
type Kind = fn(&Fragment) -> bool;

const COMMAND: Kind = |fragment| matches!(fragment, Fragment::Args(_) | Fragment::Subcommands(_));
const SUBCOMMANDS: Kind = |fragment| matches!(fragment, Fragment::Subcommands(_));
const OCCURRENCE: Kind = |fragment| matches!(fragment, Fragment::Occurrence(_));
const CHOICES: Kind = |fragment| matches!(fragment, Fragment::Choices(_));

/// `src/cli/add.rs` as the modules a path to its types may name: `src`,
/// `cli`, `add`. A `mod.rs` is its directory's module.
fn modules_of(file: &str) -> Vec<&str> {
    let mut modules: Vec<&str> = file.split('/').filter(|part| !part.is_empty()).collect();
    if let Some(last) = modules.pop() {
        let stem = last.rsplit_once('.').map_or(last, |(stem, _)| stem);
        if stem != "mod" {
            modules.push(stem);
        }
    }
    modules
}

/// A type by its crate, its file's modules and its name: no other has all three.
fn whole_name(krate: &str, entry: &Entry) -> String {
    let mut parts = modules_of(&entry.file);
    if !krate.is_empty() {
        parts.insert(0, krate);
    }
    parts.push(&entry.ident);
    parts.join("::")
}

impl Command {
    /// `prog [OPTIONS] <FILE> [COMMAND]`.
    pub fn usage(&self, path: &[String]) -> String {
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
        let (arguments, flags): (Vec<&Item>, Vec<&Item>) = self
            .items
            .iter()
            .filter(|item| !item.hide)
            .partition(|item| item.positional);
        let mut options = Vec::new();
        let mut headings: Vec<(&str, Vec<&Item>)> = Vec::new();
        for item in flags {
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

    /// Description used on a long page: the long text, or the short one.
    pub fn prose(&self) -> &str {
        if self.long_about.is_empty() {
            &self.about
        } else {
            &self.long_about
        }
    }

    /// Text after the lists on a long page: the long text, or the short one.
    pub fn after(&self) -> &str {
        if self.after_long_help.is_empty() {
            &self.after_help
        } else {
            &self.after_long_help
        }
    }

    /// The supplied help and version flags, as `(synopsis, description)`.
    pub fn builtins(&self) -> Vec<(&'static str, &'static str)> {
        let mut rows = Vec::new();
        if self.help_flag {
            rows.push(if self.help_short {
                ("-h, --help", "Print help (see a summary with '-h')")
            } else {
                ("--help", "Print help")
            });
        }
        if self.package_version.is_some() {
            rows.push(("-V, --version", "Print version"));
        }
        rows
    }

    /// What the rows of a page do not say about how this command reads its
    /// words: `long_only`, and `unknown_flags = "value"`.
    pub fn notes(&self) -> Vec<&'static str> {
        let mut notes = Vec::new();
        if self.long_only {
            notes.push("A long option may also be spelled with one dash.");
        }
        if self.unknown_flags_value {
            notes.push("A word that looks like a flag and names none is a positional value.");
        }
        notes
    }
}

impl Item {
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
        if self.two_dashes {
            append_note(&mut text, "Spelled with two dashes only.");
        }
        text
    }

    /// The left-hand spelling help prints for this item.
    pub fn synopsis(&self) -> String {
        let mut line = self.names();
        if !self.positional {
            line.push_str(&self.value_suffix());
        }
        line
    }

    /// [`synopsis`](Self::synopsis) without the value placeholder of a flag.
    ///
    /// A long name is shown with two dashes, as help shows it. Under
    /// `long_only` its one-dash spelling is a sentence of the page, in
    /// [`Command::notes`].
    pub fn names(&self) -> String {
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
        if let Some(long) = &self.long {
            names.push(format!("--{long}"));
        }
        let mut line = names.join(", ");
        if let Some(negate) = &self.negate {
            if !line.is_empty() {
                line.push_str(" / ");
            }
            line.push_str("--");
            line.push_str(negate);
        }
        line
    }
}

/// A one-line description takes the note after a space. One of several lines
/// takes it on a line of its own, as `--help` does, so that a list or a code
/// fence that ends the description stays closed.
fn append_note(text: &mut String, note: &str) {
    if !text.is_empty() {
        text.push(if text.contains('\n') { '\n' } else { ' ' });
    }
    text.push_str(note);
}

impl Page<'_> {
    /// The file name without its extension: the path joined with `-`.
    pub fn stem(&self) -> String {
        self.path.join("-")
    }
}

/// The root page, then one page per visible subcommand, at every depth.
///
/// A page is a file named after its path, so an empty word or one with a
/// path separator is an error, and so are two paths that join to one name. `tool use add` and
/// `tool use-add` would be the same file, and the second page would replace
/// the first.
pub fn pages<'a>(root: &'a Command, bin: &str) -> Result<Vec<Page<'a>>, Error> {
    let mut out = Vec::new();
    walk(root, vec![bin.to_owned()], &[], &mut out);
    let mut seen = HashSet::new();
    for page in &out {
        if let Some(word) = page
            .path
            .iter()
            .find(|word| word.is_empty() || word.contains(['/', '\\']))
        {
            return Err(Error::new(format!(
                "`{word}` cannot be part of a file name"
            )));
        }
        let stem = page.stem();
        if !seen.insert(stem.clone()) {
            return Err(Error::new(format!("two pages would be named {stem}")));
        }
    }
    Ok(out)
}

fn walk<'a>(
    command: &'a Command,
    path: Vec<String>,
    ancestors: &[&'a Item],
    out: &mut Vec<Page<'a>>,
) {
    let globals = ancestors
        .iter()
        .copied()
        .filter(|item| !item.hide && !item.positional)
        .collect();
    let mut next = ancestors.to_vec();
    next.extend(command.items.iter().filter(|item| item.global));
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

fn parse_fragment(text: &str) -> Result<Entry, Error> {
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
        "subcommands" => Fragment::Subcommands(SubsFrag {
            about: opt_str(&table, "about")?,
            long_about: opt_str(&table, "long_about")?,
            variants: parse_variants(&table)?,
        }),
        "occurrence" => Fragment::Occurrence(parse_items(&table)?),
        "choices" => Fragment::Choices(strings(&table, "choices")?),
        other => return Err(Error::new(format!("unknown kind `{other}`"))),
    };
    Ok(Entry {
        ident,
        file: opt_str(&table, "file")?,
        fragment,
    })
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
            long_about: opt_str(row, "long_about")?,
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
            item: Item {
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
                require_equals: flag(row, "require_equals")?,
                global: flag(row, "global")?,
                optional_value: flag(row, "optional_value")?,
                values: value_count(row)?,
                more_values: flag(row, "more_values")?,
                two_dashes: flag(row, "two_dashes")?,
                prefix: flag(row, "prefix")?,
                stop_flags: flag(row, "stop_flags")?,
                vocabulary: None,
            },
            choices_ty: opt_present(row, "choices_ty")?,
            keywords: opt_present(row, "keywords")?,
        });
    }
    Ok(items)
}

impl Catalog {
    /// The fragment `name` refers to where it is read.
    ///
    /// The last part of `name` is what the type is filed under. It is looked
    /// for in the crate the parts before it start with, when they name one,
    /// then in the crate it is read in, then in all the others. Where several
    /// types have the name, the one whose file is the module `name` gives is
    /// meant, or the one in the file `name` is read in when it gives none.
    fn find(&self, at: At<'_>, name: &str, kind: Kind) -> Result<Option<Found<'_>>, Error> {
        let mut modules: Vec<&str> = name.split("::").collect();
        let ident = modules.pop().unwrap_or(name);
        let named = |krate: &str| {
            let (krate, entries) = self.crates.get_key_value(krate)?;
            let named = entries.iter().filter(|entry| entry.ident == ident);
            Some(named.map(|entry| (krate.as_str(), entry)).collect())
        };
        let stated: Vec<Found<'_>> = modules
            .first()
            .and_then(|first| named(first))
            .unwrap_or_default();
        let own: Vec<Found<'_>> = named(at.krate).unwrap_or_default();
        let others = self
            .crates
            .keys()
            .filter(|krate| *krate != at.krate)
            .filter_map(|krate| named(krate))
            .flatten()
            .collect();
        let rest = modules.get(1..).unwrap_or_default();
        for (found, modules) in [(stated, rest), (own, &modules[..]), (others, &modules[..])] {
            if let Some(one) = pick(found, modules, at.file, kind, name)? {
                return Ok(Some(one));
            }
        }
        Ok(None)
    }

    fn fragment(&self, at: At<'_>, name: &str, kind: Kind) -> Result<Found<'_>, Error> {
        self.find(at, name, kind)?
            .ok_or_else(|| Error::new(format!("no fragment named `{name}`")))
    }

    fn command(
        &self,
        at: At<'_>,
        name: &str,
        stack: &mut Vec<*const Entry>,
    ) -> Result<Command, Error> {
        let (krate, entry) = self.fragment(at, name, COMMAND)?;
        let ident = &entry.ident;
        if stack.contains(&std::ptr::from_ref(entry)) {
            return Err(Error::new(format!("`{ident}` refers to itself")));
        }
        let at = At {
            krate,
            file: &entry.file,
        };
        let args = match &entry.fragment {
            Fragment::Args(args) => args,
            // An enum that is the whole command line.
            Fragment::Subcommands(subs) => {
                stack.push(entry);
                let subcommands = self.variants(at, subs, stack)?;
                stack.pop();
                return Ok(Command {
                    about: subs.about.clone(),
                    long_about: subs.long_about.clone(),
                    subcommands,
                    subcommand_required: true,
                    help_flag: true,
                    help_short: true,
                    ..Command::default()
                });
            }
            _ => return Err(Error::new(format!("`{ident}` is not a command"))),
        };
        stack.push(entry);
        let mut items = Vec::new();
        for draft in &args.items {
            items.push(self.item(at, draft, stack)?);
        }
        let mut subcommands = self.subs(at, args.subcommand.as_deref(), stack)?;
        for name in &args.flatten {
            let flattened = self.command(at, name, stack)?;
            items.extend(flattened.items);
            subcommands.extend(flattened.subcommands);
        }
        if let Some(sequence) = &args.sequence {
            let (krate, entry) = self.fragment(at, sequence, OCCURRENCE)?;
            let Fragment::Occurrence(drafts) = &entry.fragment else {
                return Err(Error::new(format!(
                    "`{sequence}` is not an occurrence enum"
                )));
            };
            let at = At {
                krate,
                file: &entry.file,
            };
            for draft in drafts {
                items.push(self.item(at, draft, stack)?);
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

    fn subs(
        &self,
        at: At<'_>,
        name: Option<&str>,
        stack: &mut Vec<*const Entry>,
    ) -> Result<Vec<Sub>, Error> {
        let Some(name) = name else {
            return Ok(Vec::new());
        };
        let (krate, entry) = self.fragment(at, name, SUBCOMMANDS)?;
        let Fragment::Subcommands(subs) = &entry.fragment else {
            return Err(Error::new(format!("`{name}` is not a subcommand enum")));
        };
        let at = At {
            krate,
            file: &entry.file,
        };
        self.variants(at, subs, stack)
    }

    fn variants(
        &self,
        at: At<'_>,
        frag: &SubsFrag,
        stack: &mut Vec<*const Entry>,
    ) -> Result<Vec<Sub>, Error> {
        let mut subs = Vec::new();
        for variant in &frag.variants {
            let mut command = match &variant.ty {
                Some(ty) => self.command(at, ty, stack)?,
                None => Command {
                    name: variant.name.clone(),
                    about: variant.about.clone(),
                    long_about: variant.long_about.clone(),
                    help_flag: true,
                    help_short: true,
                    ..Command::default()
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

    fn item(
        &self,
        at: At<'_>,
        draft: &ItemDraft,
        stack: &mut Vec<*const Entry>,
    ) -> Result<Item, Error> {
        let mut item = draft.item.clone();
        if let Some(ty) = draft
            .choices_ty
            .as_deref()
            .filter(|_| item.choices.is_empty())
        {
            // A name with no value enum under it is an open value (`String`,
            // a number), or a type that parses itself.
            if let Some((_, entry)) = self.find(at, ty, CHOICES)?
                && let Fragment::Choices(choices) = &entry.fragment
            {
                item.choices.clone_from(choices);
            }
        }
        item.vocabulary = draft
            .keywords
            .as_deref()
            .map(|ty| self.command(at, ty, stack).map(Box::new))
            .transpose()?;
        Ok(item)
    }
}

/// The one of `found` that `name` means, when they all have its last part.
///
/// Several are told apart by what `name` may be, then by the modules it
/// gives, or by the file it is read in when it gives none.
fn pick<'a>(
    mut found: Vec<Found<'a>>,
    modules: &[&str],
    file: &str,
    kind: Kind,
    name: &str,
) -> Result<Option<Found<'a>>, Error> {
    if found.len() > 1 {
        found.retain(|(_, entry)| kind(&entry.fragment));
    }
    if found.len() < 2 {
        return Ok(found.pop());
    }
    let meant = |(_, entry): &Found<'_>| match modules {
        [] => entry.file == file,
        _ => modules_of(&entry.file).ends_with(modules),
    };
    if let [one] = found.iter().copied().filter(meant).collect::<Vec<_>>()[..] {
        return Ok(Some(one));
    }
    let all: Vec<String> = found
        .iter()
        .map(|(krate, entry)| format!("`{}`", whole_name(krate, entry)))
        .collect();
    Err(Error::new(format!(
        "`{name}` may be {}: where it is named, state the module too, or give one of the types \
         another name for the documentation, as `#[arg(spec = \"Name\")]`",
        all.join(" or ")
    )))
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
        let pages = pages(&command, "tool").unwrap();
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
                .any(|global| global.long.as_deref() == Some("verbose"))
        );
        assert!(pages[0].globals.is_empty());
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
        let error = pages(&command, "tool").unwrap_err();
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
        assert_eq!(two.roots(), ["One", "Two"]);
        let error = two.root().unwrap_err();
        assert!(error.to_string().contains("One"), "{error}");
        assert!(error.to_string().contains("Two"), "{error}");
    }

    #[test]
    fn a_name_is_its_own_crate_s_type_first() {
        let args = |ident: &str, rest: &str| {
            format!("version = 1\nkind = \"args\"\nident = \"{ident}\"\n{rest}")
        };
        let mut catalog = Catalog::default();
        for (krate, text) in [
            (
                "app",
                args("Cli", "name = \"app\"\nflatten = [\"Common\", \"Shared\"]"),
            ),
            ("app", args("Common", "[[item]]\nlong = \"mine\"")),
            ("dep", args("Cli", "name = \"dep\"")),
            ("dep", args("Common", "[[item]]\nlong = \"theirs\"")),
            ("dep", args("Shared", "[[item]]\nlong = \"shared\"")),
        ] {
            catalog.insert(krate, krate, &text).unwrap();
        }
        assert_eq!(catalog.roots(), ["app::Cli", "dep::Cli", "dep::Common"]);
        let error = catalog.stitch("Cli").unwrap_err();
        assert!(error.to_string().contains("app::Cli"), "{error}");
        let command = catalog.stitch("app::Cli").unwrap();
        let longs: Vec<_> = command
            .items
            .iter()
            .map(|item| item.long.as_deref().unwrap())
            .collect();
        assert_eq!(longs, ["mine", "shared"]);
    }

    #[test]
    fn a_module_or_the_same_file_tells_two_types_of_one_name_apart() {
        let args = |file: &str, ident: &str, rest: &str| {
            format!("version = 1\nfile = \"{file}\"\nkind = \"args\"\nident = \"{ident}\"\n{rest}")
        };
        let mut catalog = Catalog::default();
        for (krate, text) in [
            (
                "app",
                args(
                    "src/main.rs",
                    "Cli",
                    "flatten = [\"Opts\", \"cli::add::Opts\", \"dep::Opts\"]",
                ),
            ),
            (
                "app",
                args("src/main.rs", "Opts", "[[item]]\nlong = \"main\""),
            ),
            (
                "app",
                args("src/cli/add.rs", "Opts", "[[item]]\nlong = \"add\""),
            ),
            (
                "app",
                args(
                    "src/cli/remove/mod.rs",
                    "Opts",
                    "[[item]]\nlong = \"remove\"",
                ),
            ),
            ("app", args("src/other.rs", "Lost", "flatten = [\"Opts\"]")),
            (
                "dep",
                args("src/lib.rs", "Opts", "[[item]]\nlong = \"dep\""),
            ),
        ] {
            catalog.insert(krate, krate, &text).unwrap();
        }
        let command = catalog.stitch("Cli").unwrap();
        let longs: Vec<_> = command
            .items
            .iter()
            .map(|item| item.long.as_deref().unwrap())
            .collect();
        assert_eq!(longs, ["main", "add", "dep"]);
        assert_eq!(
            catalog.roots(),
            ["Cli", "Lost", "app::src::cli::remove::Opts"]
        );
        let one = catalog.stitch("app::src::cli::remove::Opts").unwrap();
        assert_eq!(one.items[0].long.as_deref(), Some("remove"));
        assert_eq!(
            catalog.stitch("remove::Opts").unwrap().items[0]
                .long
                .as_deref(),
            Some("remove")
        );
        // No module, and no `Opts` in the file that names it: any of three.
        let error = catalog.stitch("Lost").unwrap_err().to_string();
        assert!(error.contains("`Opts` may be"), "{error}");
        assert!(error.contains("app::src::cli::add::Opts"), "{error}");
        assert!(!error.contains("dep::"), "{error}");
    }

    #[test]
    fn a_subcommand_enum_nothing_names_is_the_program() {
        let catalog = Catalog::from_pairs([
            (
                "Tool",
                r#"
                version = 1
                kind = "subcommands"
                ident = "Tool"
                about = "A tool."

                [[variant]]
                name = "run"
                ty = "Run"
                "#,
            ),
            (
                "Run",
                r#"
                version = 1
                kind = "args"
                ident = "Run"
                "#,
            ),
        ])
        .unwrap();
        assert_eq!(catalog.root().unwrap(), "Tool");
        let command = catalog.stitch("Tool").unwrap();
        assert_eq!(command.about, "A tool.");
        assert!(command.subcommand_required);
        assert_eq!(
            command.usage(&["tool".to_owned()]),
            "tool [OPTIONS] <COMMAND>"
        );
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
            aliases = ["really"]
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
            command.items[0].synopsis(),
            "-f, --force / --no-force=<MODE>"
        );
        assert!(command.items[0].choices.is_empty());
        assert_eq!(command.items[1].synopsis(), "<FILE>...");
        assert_eq!(
            command.notes(),
            ["A long option may also be spelled with one dash."]
        );
        assert_eq!(
            command.items[0].description(),
            "Force it. [default: on] The rest of the word is the value."
        );
    }
}
