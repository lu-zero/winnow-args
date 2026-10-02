//! mold's `--features winnow-args` parser, from the option chain of its own.
//!
//! mold's `parse_args` tries each option in turn against the current word:
//! one `if … else if` chain. Each arm's matchers (`read_flag`, `read_arg!`,
//! `read_eq!`, `read_switch`, the `-z` forms, the LTO table) become variants
//! of an `Occurrence` enum, the `-z` keywords `z_opt`, and the arms one
//! `match` over the parsed sequence, each running mold's own body. The
//! built-in parser stays, compiled out by the feature statement by statement.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{Result, regex};

const TEMPLATE: &str = include_str!("../templates/mold.rs");

const CURSOR: &str = "let mut cursor = ArgCursor { args: raw_cmdline, index: 1 };";
const LOOP: &str = "while cursor.index < raw_cmdline.len() {";
/// The value a flag given without one gets, to tell `--x` from `--x=`.
const BARE: &str = "\\0";
/// How the arm of `-dynamic` tests for it; the lexer leaves it as an unknown flag.
const DYNAMIC: &str = "cursor.text() == \"-dynamic\"";
/// What the built-in parser alone uses.
const LEGACY_ITEMS: &[&str] = &[
    "fn match_option<",
    "struct ArgCursor<",
    "impl<'a> ArgCursor<'a> {",
];

/// `read_lto_option`'s table: the spelling, and the plugin option it adds.
const LTO_FLAGS: &[(&str, &str)] = &[
    ("--lto-cs-profile-generate", "cs-profile-generate"),
    ("--lto-debug-pass-manager", "debug-pass-manager"),
    ("disable-verify", "disable-verify"),
    ("--lto-emit-asm", "emit-asm"),
    ("no-legacy-pass-manager", "legacy-pass-manager"),
    ("no-lto-legacy-pass-manager", "new-pass-manager"),
    ("--opt-remarks-with-hotness", "opt-remarks-with-hotness"),
    (
        "lto-pseudo-probe-for-profiling",
        "pseudo-probe-for-profiling",
    ),
    ("save-temps", "save-temps"),
    ("thinlto-emit-imports-files", "thinlto-emit-imports-files"),
    ("thinlto-index-only", "thinlto-index-only"),
];
const LTO_ARGS: &[(&str, &str)] = &[
    ("--lto-cs-profile-file", "cs-profile-path="),
    ("--lto-partitions", "lto-partitions="),
    ("--lto-obj-path", "obj-path="),
    ("--opt-remarks-filename", "opt-remarks-filename="),
    ("--opt-remarks-format", "opt-remarks-format="),
    (
        "--opt-remarks-hotness-threshold",
        "opt-remarks-hotness-threshold=",
    ),
    ("--opt-remarks-passes", "opt-remarks-passes="),
    ("--lto-sample-profile", "sample-profile="),
    ("thinlto-index-only", "thinlto-index-only="),
    (
        "thinlto-object-suffix-replace",
        "thinlto-object-suffix-replace=",
    ),
    ("thinlto-prefix-replace", "thinlto-prefix-replace="),
    ("thinlto-cache-dir", "cache-dir="),
    ("thinlto-cache-policy", "cache-policy="),
    ("thinlto-jobs", "jobs="),
];

pub(crate) fn generate(mold: &Path) -> Result<()> {
    let pristine = Command::new("git")
        .current_dir(mold)
        .args(["show", "origin/main:src/cmdline.rs"])
        .output()?;
    if !pristine.status.success() {
        return Err(String::from_utf8_lossy(&pristine.stderr).into());
    }
    let source = String::from_utf8(pristine.stdout)?;
    let lines: Vec<&str> = source.split('\n').collect();
    let chain = chain(&lines)?;
    let parser = Parser::new(&chain)?;

    let required_equals: Vec<String> = parser
        .variants
        .iter()
        .filter(|v| v.require_equals)
        .map(|v| format!("    \"--{}\"", v.base))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let module = TEMPLATE
        .replace("@ENUM@", &parser.item_enum())
        .replace("@ZOPT@", &parser.z_opt()?)
        .replace("@EQ@", &required_equals.join(",\n"));
    let (cmdline, winnow) = (
        mold.join("src/cmdline.rs"),
        mold.join("src/cmdline_winnow.rs"),
    );
    fs::write(&winnow, module)?;
    fs::write(&cmdline, patched(&lines, &parser.fold()?)?)?;
    Command::new("cargo")
        .current_dir(mold)
        .args(["fmt", "--", "src/cmdline.rs", "src/cmdline_winnow.rs"])
        .status()?;
    println!(
        "{} variants, {} z variants, {} match arms",
        parser.variants.len(),
        parser.z_variants.len(),
        parser.match_arms.len()
    );
    Ok(())
}

/// The index of the first line at or after `from` that `is` holds for.
fn line(lines: &[&str], from: usize, what: &str, is: impl Fn(&str) -> bool) -> Result<usize> {
    lines[from..]
        .iter()
        .position(|l| is(l))
        .map(|at| from + at)
        .ok_or_else(|| format!("mold's cmdline.rs has no {what}").into())
}

/// How an arm recognizes its option.
#[derive(Clone, Copy, PartialEq)]
enum Matcher {
    Flag,
    Arg,
    Eq,
    Switch,
    ZFlag,
    ZArg,
    ZSwitch,
    Lto,
}

/// One arm of the chain: its matchers with their arguments, and its body.
struct Arm<'a> {
    matchers: Vec<(Matcher, Vec<Option<String>>)>,
    body: &'a [&'a str],
    /// It is the arm of `-dynamic`.
    dynamic: bool,
}

/// The option loop's arms, and the statements ahead of them (inputs, `--help`).
struct Chain<'a> {
    arms: Vec<Arm<'a>>,
    input: String,
}

fn chain<'a>(lines: &'a [&'a str]) -> Result<Chain<'a>> {
    let start = line(lines, 0, "option loop", |l| l.trim() == LOOP)?;
    let first = line(lines, start, "`-o` arm", |l| {
        l.starts_with("        if read_arg!(\"o\", true)")
    })?;
    let end = line(lines, first, "end of the option loop", |l| l == "    }")?;
    let patterns = [
        (Matcher::Flag, regex(r#"cursor\.read_flag\("([^"]+)"\)"#)),
        (Matcher::Arg, regex(r#"read_arg!\("([^"]+)"(, true)?\)"#)),
        (Matcher::Eq, regex(r#"read_eq!\("([^"]+)"(, true)?\)"#)),
        (
            Matcher::Switch,
            regex(r#"cursor\.read_switch\(\s*"([^"]+)",\s*"([^"]+)"\s*\)"#),
        ),
        (Matcher::ZFlag, regex(r#"cursor\.read_z_flag\("([^"]+)"\)"#)),
        (Matcher::ZArg, regex(r#"read_z_arg!\("([^"]+)"\)"#)),
        (
            Matcher::ZSwitch,
            regex(r#"cursor\.read_z_switch\(\s*"([^"]+)",\s*"([^"]+)"\s*\)"#),
        ),
        (Matcher::Lto, regex(r"cursor\.read_lto_option\(\)")),
    ];

    let mut arms = Vec::new();
    let mut at = first;
    while at < end {
        let opens = lines[at];
        let mut header_end = at;
        let header = if opens == "        } else {" {
            "else".to_owned()
        } else if opens.starts_with("        if ") || opens.starts_with("        } else if ") {
            while !lines[header_end].trim_end().ends_with('{') {
                header_end += 1;
            }
            let header: Vec<&str> = lines[at..=header_end].iter().map(|l| l.trim()).collect();
            header.join(" ")
        } else {
            return Err(format!("line {}: not an arm of the chain: {opens}", at + 1).into());
        };
        let body_len = lines[header_end + 1..end]
            .iter()
            .take_while(|l| !l.starts_with("        }"))
            .count();
        let body = &lines[header_end + 1..header_end + 1 + body_len];

        let mut matchers = Vec::new();
        for (matcher, pattern) in &patterns {
            for found in pattern.captures_iter(&header) {
                let groups = found.iter().skip(1);
                let groups = groups.map(|g| g.map(|g| g.as_str().to_owned()));
                let start = found.get(0).expect("the match").start();
                matchers.push((start, *matcher, groups.collect()));
            }
        }
        matchers.sort_by_key(|(start, ..)| *start);
        let matchers = matchers.into_iter().map(|(_, m, g)| (m, g)).collect();
        arms.push(Arm {
            matchers,
            body,
            dynamic: header.contains(DYNAMIC),
        });

        at = header_end + 1 + body_len;
        if lines[at] == "        }" {
            break;
        }
    }
    Ok(Chain {
        arms,
        input: lines[start + 1..first].join("\n"),
    })
}

/// How a spelling takes its value.
#[derive(Clone, Copy, PartialEq)]
enum Takes {
    /// None: `read_flag`.
    Nothing,
    /// Attached or in the next word: `read_arg!`.
    Value,
    /// Attached only: `read_eq!`.
    Attached,
}

/// One spelling of an option: the arm that handles it and how it is written.
struct Spelling {
    arm: usize,
    takes: Takes,
    /// The arm reads the value as an `OsStr`, not as UTF-8.
    raw: bool,
    /// The value a `read_switch` spelling stands for.
    switch: Option<bool>,
    /// For the LTO table: the plugin option, and whether a value follows it.
    lto: Option<(&'static str, bool)>,
    two_dashes: bool,
    base: String,
    /// `sha1` of a flag written `build-id=sha1`.
    literal: Option<String>,
    short: bool,
}

impl Spelling {
    fn new(arm: usize, takes: Takes, name: &str) -> Self {
        let two_dashes = name.starts_with("--");
        let name = name.strip_prefix("--").unwrap_or(name);
        let (base, literal) = match name.split_once('=') {
            Some((base, literal)) if takes == Takes::Nothing => (base, Some(literal.to_owned())),
            _ => (name, None),
        };
        Spelling {
            arm,
            takes,
            raw: false,
            switch: None,
            lto: None,
            two_dashes,
            base: base.to_owned(),
            literal,
            short: base.chars().count() == 1,
        }
    }

    fn spelled(&self) -> String {
        let dashes = if self.two_dashes { "--" } else { "-" };
        match &self.literal {
            Some(literal) => format!("{dashes}{}={literal}", self.base),
            None => format!("{dashes}{}", self.base),
        }
    }
}

/// One `-z` keyword.
struct ZSpelling {
    arm: usize,
    takes_value: bool,
    name: String,
    switch: Option<bool>,
}

/// A variant of `Item`: every spelling with the same name.
struct Variant {
    base: String,
    spellings: Vec<usize>,
    name: String,
    unit: bool,
    require_equals: bool,
    default_missing: bool,
    two_dashes: bool,
    short: bool,
}

/// A `-z` keyword whose arm has no other spelling: a variant of its own.
struct ZVariant {
    keyword: String,
    name: String,
    takes_value: bool,
}

/// An arm of the generated `match`: its pattern, the bindings mold's body
/// expects, and the arm of the chain whose body it runs.
struct MatchArm {
    pattern: String,
    prelude: Vec<String>,
    arm: Option<usize>,
}

struct Parser<'a> {
    chain: &'a Chain<'a>,
    spellings: Vec<Spelling>,
    z_spellings: Vec<ZSpelling>,
    variants: Vec<Variant>,
    z_variants: Vec<ZVariant>,
    /// The `-z`-only arms.
    z_only: HashSet<usize>,
    match_arms: Vec<MatchArm>,
}

impl<'a> Parser<'a> {
    fn new(chain: &'a Chain<'a>) -> Result<Self> {
        let (spellings, z_spellings) = spellings(&chain.arms)?;
        let mut names = HashSet::new();
        let variants = variants(&chain.arms, &spellings, &mut names);
        let z_only: HashSet<usize> = z_spellings
            .iter()
            .map(|z| z.arm)
            .filter(|arm| !spellings.iter().any(|s| s.arm == *arm))
            .collect();
        let mut z_variants: Vec<ZVariant> = Vec::new();
        for z in z_spellings.iter().filter(|z| z_only.contains(&z.arm)) {
            let variant = ZVariant {
                keyword: z.name.clone(),
                name: unique(format!("Z{}", camel(&z.name)), &mut names),
                takes_value: z.takes_value,
            };
            match z_variants.iter_mut().find(|v| v.keyword == z.name) {
                Some(known) => *known = variant,
                None => z_variants.push(variant),
            }
        }
        let mut parser = Parser {
            chain,
            spellings,
            z_spellings,
            variants,
            z_variants,
            z_only,
            match_arms: Vec::new(),
        };
        parser.match_arms = parser.arms()?;
        Ok(parser)
    }

    fn body(&self, arm: usize) -> &[&str] {
        self.chain.arms[arm].body
    }

    /// Whether the arm's body names `ident`.
    fn uses(&self, arm: usize, ident: &str) -> bool {
        regex(&format!(r"\b{ident}\b")).is_match(&self.body(arm).join("\n"))
    }

    fn variant(&self, base: &str) -> &Variant {
        let found = self.variants.iter().find(|v| v.base == base);
        found.expect("every spelling has a variant")
    }

    fn item_enum(&self) -> String {
        let mut out = vec![
            "#[derive(Occurrence)]\n#[arg(allow_hyphen_values, keep_equals)]\npub(crate) enum Item {"
                .to_owned(),
        ];
        for v in &self.variants {
            let mut attrs = Vec::new();
            if v.short {
                attrs.push(match v.base.as_str() {
                    "'" => "short = '\\''".to_owned(),
                    base => format!("short = '{base}'"),
                });
                if v.base == "l" {
                    attrs.push("prefix".to_owned());
                }
            } else {
                attrs.push(format!("long = {}", rust_str(&v.base)));
                if v.two_dashes {
                    attrs.push("two_dashes".to_owned());
                }
            }
            if v.require_equals {
                attrs.push("require_equals".to_owned());
            }
            if v.default_missing {
                attrs.push(format!("default_missing = \"{BARE}\""));
            }
            let spelled: BTreeSet<String> = v
                .spellings
                .iter()
                .map(|s| self.spellings[*s].spelled())
                .collect();
            let spelled: Vec<String> = spelled.into_iter().collect();
            out.push(format!("    /// `{}`", spelled.join(", ")));
            out.push(format!("    #[arg({})]", attrs.join(", ")));
            let value = if v.unit { "" } else { "(OsString)" };
            out.push(format!("    {}{value},", v.name));
        }
        out.push(
            "    /// `-z KEYWORD`, `-zKEYWORD`: mapped by [`z_opt`] as it is handled;\n    \
             /// one that stays is unknown.\n    #[arg(short = 'z')]\n    Z(Spanned<OsString>),"
                .to_owned(),
        );
        for z in &self.z_variants {
            let (doc, value) = match z.takes_value {
                true => ("=VALUE", "(OsString)"),
                false => ("", ""),
            };
            out.push(format!(
                "    /// `-z {}{doc}`.\n    #[arg(skip)]\n    {}{value},",
                z.keyword, z.name
            ));
        }
        out.push(
            "    /// A flag nothing above names, whole: `--lto-O3`, or an error.\n    \
             #[arg(unknown)]\n    Unknown(OsString),"
                .to_owned(),
        );
        out.push(
            "    /// Several short options in one word (`-sS`): accepted, with a warning,\n    \
             /// as GNU ld does.\n    #[arg(bundle)]\n    Grouped(OsString),"
                .to_owned(),
        );
        out.push("    /// An input file.\n    #[arg(positional)]\n    Input(OsString),".to_owned());
        out.push("    /// `--help`.\n    #[arg(long = \"help\")]\n    Help,".to_owned());
        out.push("}".to_owned());
        out.join("\n")
    }

    /// The item a `-z` keyword of `arm` stands for: the arm's first spelling.
    fn representative(&self, arm: usize) -> Result<String> {
        let spelling = self.spellings.iter().find(|s| s.arm == arm);
        let spelling = spelling.ok_or_else(|| format!("arm {arm} has no spelling"))?;
        let name = &self.variant(&spelling.base).name;
        Ok(if self.variant(&spelling.base).unit {
            format!("Item::{name}")
        } else if let Some(literal) = &spelling.literal {
            format!("Item::{name}(OsString::from({}))", rust_str(literal))
        } else if spelling.takes == Takes::Nothing {
            format!("Item::{name}(OsString::from(\"{BARE}\"))")
        } else {
            format!("Item::{name}(OsString::new())")
        })
    }

    fn z_variant(&self, keyword: &str) -> &ZVariant {
        let found = self.z_variants.iter().find(|v| v.keyword == keyword);
        found.expect("a `-z`-only keyword has a variant")
    }

    fn z_opt(&self) -> Result<String> {
        let mut out = vec![
            "/// The option a `-z` keyword stands for, as mold reads `-z` (the exact\n\
             /// keyword, or `name=value`); `None` for an unknown one.\n\
             pub(crate) fn z_opt(word: &OsStr) -> Option<Item> {"
                .to_owned(),
            "    let word = word.to_str()?;".to_owned(),
            "    Some(match word {".to_owned(),
        ];
        for z in self.z_spellings.iter().filter(|z| !z.takes_value) {
            let target = if self.z_only.contains(&z.arm) {
                format!("Item::{}", self.z_variant(&z.name).name)
            } else {
                self.representative(z.arm)?
            };
            out.push(format!("        {} => {target},", rust_str(&z.name)));
        }
        out.push("        _ => {".to_owned());
        for z in self.z_spellings.iter().filter(|z| z.takes_value) {
            let prefix = rust_str(&format!("{}=", z.name));
            let (unused, target) = if self.z_only.contains(&z.arm) {
                let name = &self.z_variant(&z.name).name;
                ("", format!("Item::{name}(OsString::from(value))"))
            } else {
                (
                    "                let _ = value;\n",
                    self.representative(z.arm)?,
                )
            };
            out.push(format!(
                "            if let Some(value) = word.strip_prefix({prefix}) {{\n{unused}                \
                 return Some({target});\n            }}"
            ));
        }
        out.push("            return None;\n        }\n    })\n}".to_owned());
        Ok(out.join("\n"))
    }

    fn arms(&self) -> Result<Vec<MatchArm>> {
        let mut arms: Vec<MatchArm> = Vec::new();
        for v in &self.variants {
            let (mut guarded, mut plain) = (Vec::new(), Vec::new());
            for spelling in v.spellings.iter().map(|s| &self.spellings[*s]) {
                let mut prelude = Vec::new();
                if let Some(value) = spelling.switch {
                    prelude.push(format!("let value = {value};"));
                }
                match spelling.lto {
                    Some((option, false)) => {
                        prelude.push(format!("let option = b{}.to_vec();", rust_str(option)));
                    }
                    Some((option, true)) => prelude.push(format!(
                        "let option = [b{}.as_slice(), raw_arg.as_encoded_bytes()].concat();",
                        rust_str(option)
                    )),
                    None => {}
                }
                let has_value = spelling.takes != Takes::Nothing || spelling.lto.is_some();
                if !v.unit && has_value {
                    let lto_value = matches!(spelling.lto, Some((_, true)));
                    if !spelling.raw && spelling.lto.is_none() && self.uses(spelling.arm, "arg") {
                        let dashes = if spelling.short { "-" } else { "--" };
                        let spelled = rust_str(&format!("{dashes}{}", spelling.base));
                        prelude.insert(0, format!("let arg = utf8_arg(value_os, {spelled});"));
                    }
                    if lto_value || self.uses(spelling.arm, "raw_arg") {
                        prelude.insert(0, "let raw_arg: &OsStr = value_os;".to_owned());
                    }
                }
                let guard = match &spelling.literal {
                    _ if v.unit => None,
                    Some(literal) => Some(rust_str(literal)),
                    None if spelling.takes == Takes::Nothing => Some(format!("\"{BARE}\"")),
                    None => None,
                };
                match guard {
                    Some(guard) => guarded.push((Some(guard), prelude, spelling.arm)),
                    None => plain.push((None, prelude, spelling.arm)),
                }
            }
            let catch_all = !v.unit && plain.is_empty();
            for (guard, prelude, arm) in guarded.into_iter().chain(plain) {
                let reads = guard.is_some() || prelude.iter().any(|p| p.contains("value_os"));
                let binding = match (v.unit, reads) {
                    (true, _) => "",
                    (false, true) => "(value_os)",
                    (false, false) => "(_)",
                };
                let guard = guard.map_or(String::new(), |guard| {
                    format!(" if value_os.as_encoded_bytes() == b{guard}")
                });
                arms.push(MatchArm {
                    pattern: format!("Item::{}{binding}{guard}", v.name),
                    prelude,
                    arm: Some(arm),
                });
            }
            if catch_all {
                arms.push(MatchArm {
                    pattern: format!("Item::{}(value_os)", v.name),
                    prelude: vec![format!(
                        "fatal!(\"unknown command line option: --{}={{}}\", value_os.to_string_lossy());",
                        v.base
                    )],
                    arm: None,
                });
            }
        }

        for z in &self.z_variants {
            let spelling = self.z_spellings.iter().find(|s| s.name == z.keyword);
            let spelling = spelling.expect("a `-z` variant has a spelling");
            let mut prelude = Vec::new();
            if let Some(value) = spelling.switch {
                prelude.push(format!("let value = {value};"));
            }
            if z.takes_value {
                prelude.push(format!(
                    "let arg = utf8_arg(value_os, \"-z {}\");",
                    z.keyword
                ));
            }
            let binding = if z.takes_value { "(value_os)" } else { "" };
            arms.push(MatchArm {
                pattern: format!("Item::{}{binding}", z.name),
                prelude,
                arm: Some(spelling.arm),
            });
        }

        // Spellings of one mold arm with the same handling share a match arm: `A | B`.
        let mut merged: Vec<(Vec<String>, MatchArm)> = Vec::new();
        for arm in arms {
            let bare = |prelude: &[String]| -> Vec<String> {
                let kept = prelude.iter().filter(|l| !l.contains("utf8_arg"));
                kept.cloned().collect()
            };
            let joins = merged.last().is_some_and(|(patterns, last)| {
                let previous = patterns.last().expect("an arm has a pattern");
                !arm.pattern.contains(" if ")
                    && arm.arm.is_some()
                    && last.arm == arm.arm
                    && !previous.contains(" if ")
                    && arm.pattern.contains("(value_os)") == previous.contains("(value_os)")
                    && arm.pattern.contains("(_)") == previous.contains("(_)")
                    && bare(&arm.prelude) == bare(&last.prelude)
            });
            match merged.last_mut() {
                Some((patterns, _)) if joins => patterns.push(arm.pattern),
                _ => merged.push((vec![arm.pattern.clone()], arm)),
            }
        }
        Ok(merged
            .into_iter()
            .map(|(patterns, arm)| MatchArm {
                pattern: patterns.join(" | "),
                ..arm
            })
            .collect())
    }

    /// The loop over the parsed options, each handled by mold's own body.
    fn fold(&self) -> Result<String> {
        let mut out: Vec<String> = [
            "let mut opts = crate::cmdline_winnow::parse(raw_cmdline);",
            "for opt in &mut opts {",
            "    if let Item::Z(z) = opt",
            "        && let Some(known) = crate::cmdline_winnow::z_opt(&z.value)",
            "    {",
            "        *opt = known;",
            "    }",
            "    match opt {",
        ]
        .map(str::to_owned)
        .into();
        let deeper = |lines: &str| -> Vec<String> {
            lines.split('\n').map(|l| format!("    {l}")).collect()
        };

        let input = regex(
            r#"(?s)if !cursor\.current\(\)\.as_encoded_bytes\(\)\.starts_with\(b"-"\) \{\n(.*?)\n            cursor\.index \+= 1;\n            continue;\n        \}"#,
        );
        let input = input.captures(&self.chain.input);
        let input = input.ok_or("mold's cmdline.rs has no input-file block")?;
        out.push("        Item::Input(value_os) => {".to_owned());
        out.extend(deeper(&input[1].replace(
            "PathBuf::from(&cursor.current())",
            "PathBuf::from(std::mem::take(value_os))",
        )));
        out.push("        }".to_owned());

        let help = regex(r#"(?s)if cursor\.read_flag\("help"\) \{\n(.*?)\n        \}"#);
        let help = help.captures(&self.chain.input);
        let help = help.ok_or("mold's cmdline.rs has no `--help` block")?;
        out.push("        Item::Help => {".to_owned());
        out.extend(deeper(&help[1]));
        out.push("        }".to_owned());

        let body = |arm: usize, indent: &str| -> Vec<String> {
            let lines = self.body(arm).iter();
            lines
                .map(|l| match l.trim().is_empty() {
                    true => String::new(),
                    false => format!("{indent}{l}"),
                })
                .collect()
        };
        for arm in &self.match_arms {
            out.push(format!("        {} => {{", arm.pattern));
            out.extend(arm.prelude.iter().map(|l| format!("            {l}")));
            if let Some(arm) = arm.arm {
                out.extend(body(arm, "    "));
            }
            out.push("        }".to_owned());
        }

        let dynamic = self.chain.arms.iter().position(|arm| arm.dynamic);
        let dynamic = dynamic.ok_or("mold's cmdline.rs has no `-dynamic` arm")?;
        out.extend(
            [
                "        Item::Grouped(value_os) => {",
                "            warn!(",
                "                \"grouped short command line options are deprecated: {}\",",
                "                value_os.to_string_lossy()",
                "            );",
                "        }",
                "        Item::Unknown(value_os) => {",
                "            if let Some(level) = value_os.as_encoded_bytes().strip_prefix(b\"--lto-O\") {",
                "                a.plugin_opt.push([b\"O\", level].concat());",
                "            } else if value_os.as_os_str() == \"-dynamic\" {",
            ]
            .map(str::to_owned),
        );
        out.extend(body(dynamic, "        "));
        out.extend(
            [
                "            } else {",
                "                fatal!(\"unknown command line option: {}\", value_os.to_string_lossy());",
                "            }",
                "        }",
                "        Item::Z(z) => {",
                "            if z.attached {",
                "                warn!(\"unknown command line option: -z{}\", z.value.to_string_lossy());",
                "            } else {",
                "                warn!(\"unknown command line option: -z {}\", z.value.to_string_lossy());",
                "            }",
                "        }",
                "    }",
                "}",
            ]
            .map(str::to_owned),
        );
        Ok(out.join("\n"))
    }
}

/// Every spelling the chain's matchers name, in order.
fn spellings(arms: &[Arm<'_>]) -> Result<(Vec<Spelling>, Vec<ZSpelling>)> {
    let (mut spellings, mut z_spellings) = (Vec::new(), Vec::new());
    for (arm, Arm { matchers, .. }) in arms.iter().enumerate() {
        for (matcher, groups) in matchers {
            let group = |n: usize| -> Result<&str> {
                let group = groups.get(n).and_then(|g| g.as_deref());
                group.ok_or_else(|| format!("arm {arm}: a matcher without its name").into())
            };
            let raw = groups.get(1).is_some_and(Option::is_some);
            let z = |name: &str, takes_value, switch| ZSpelling {
                arm,
                takes_value,
                name: name.to_owned(),
                switch,
            };
            match matcher {
                Matcher::Flag => spellings.push(Spelling::new(arm, Takes::Nothing, group(0)?)),
                Matcher::Arg | Matcher::Eq => {
                    let takes = match matcher {
                        Matcher::Arg => Takes::Value,
                        _ => Takes::Attached,
                    };
                    spellings.push(Spelling {
                        raw,
                        ..Spelling::new(arm, takes, group(0)?)
                    });
                }
                Matcher::Switch => {
                    for (n, value) in [(0, true), (1, false)] {
                        spellings.push(Spelling {
                            switch: Some(value),
                            ..Spelling::new(arm, Takes::Nothing, group(n)?)
                        });
                    }
                }
                Matcher::ZFlag => z_spellings.push(z(group(0)?, false, None)),
                Matcher::ZArg => z_spellings.push(z(group(0)?, true, None)),
                Matcher::ZSwitch => {
                    z_spellings.push(z(group(0)?, false, Some(true)));
                    z_spellings.push(z(group(1)?, false, Some(false)));
                }
                Matcher::Lto => {
                    for (name, option) in LTO_FLAGS {
                        spellings.push(Spelling {
                            lto: Some((*option, false)),
                            ..Spelling::new(arm, Takes::Nothing, name)
                        });
                    }
                    for (name, option) in LTO_ARGS {
                        spellings.push(Spelling {
                            raw: true,
                            lto: Some((*option, true)),
                            ..Spelling::new(arm, Takes::Value, name)
                        });
                    }
                }
            }
        }
    }
    Ok((spellings, z_spellings))
}

/// One variant per spelling name, in order of first appearance.
fn variants(arms: &[Arm<'_>], spellings: &[Spelling], names: &mut HashSet<String>) -> Vec<Variant> {
    let mut groups: Vec<(&str, Vec<usize>)> = Vec::new();
    let mut by_base: HashMap<&str, usize> = HashMap::new();
    for (at, spelling) in spellings.iter().enumerate() {
        let group = *by_base.entry(&spelling.base).or_insert_with(|| {
            groups.push((&spelling.base, Vec::new()));
            groups.len() - 1
        });
        groups[group].1.push(at);
    }
    // An arm whose body is empty or all comments: accepted and ignored.
    let ignored = |arm: usize| {
        let mut code = arms[arm].body.iter().map(|l| l.trim());
        !code.any(|l| !l.is_empty() && !l.starts_with("//"))
    };
    let first_long = |arm: usize| {
        let long = spellings.iter().find(|s| s.arm == arm && !s.short);
        long.map(|s| s.base.as_str())
    };

    let mut variants = Vec::new();
    for (base, members) in groups {
        let of = |at: &usize| &spellings[*at];
        let first = of(&members[0]);
        let flag = |s: &Spelling| s.takes == Takes::Nothing;
        let unit = members
            .iter()
            .map(of)
            .all(|s| flag(s) && s.literal.is_none());
        let any_flag = members.iter().map(of).any(flag);
        let any_value = members.iter().map(of).any(|s| s.takes == Takes::Value);
        let letter = base.chars().next().filter(|_| first.short);
        // A letter is named for its case: `ShortS` for `-S`, `ShortSLower` for `-s`.
        let cased = |letter: char| {
            let lower = if letter.is_uppercase() { "" } else { "Lower" };
            format!("Short{}{lower}", camel(base).to_uppercase())
        };
        let name = match letter {
            Some(letter) if ignored(first.arm) && letter.is_alphabetic() => {
                format!("Ignored{}", cased(letter))
            }
            _ if ignored(first.arm) => format!("Ignored{}", camel(base)),
            Some(letter) => match first_long(first.arm) {
                Some(long) => format!("{}Short", camel(long)),
                None => cased(letter),
            },
            None if base.starts_with(':') => format!("Internal{}", camel(base)),
            None => camel(base),
        };
        variants.push(Variant {
            base: base.to_owned(),
            name: unique(name, names),
            unit,
            require_equals: !unit && (any_flag || !any_value),
            default_missing: !unit
                && members
                    .iter()
                    .map(of)
                    .any(|s| flag(s) && s.literal.is_none()),
            two_dashes: members.iter().map(of).all(|s| s.two_dashes),
            short: first.short,
            spellings: members,
        });
    }
    variants
}

/// `name`, with `X`s appended until no other variant has it.
fn unique(mut name: String, names: &mut HashSet<String>) -> String {
    while !names.insert(name.clone()) {
        name.push('X');
    }
    name
}

/// `as-needed` → `AsNeeded`.
fn camel(name: &str) -> String {
    let name = match name {
        "(" => "open-paren",
        ")" => "close-paren",
        name => name,
    };
    let mut out = String::new();
    for part in name.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut letters = part.chars();
        if let Some(first) = letters.next() {
            out.extend(first.to_uppercase());
            out.push_str(letters.as_str());
        }
    }
    out
}

fn rust_str(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// mold's `cmdline.rs` with the generated loop next to the built-in one, each
/// under its `cfg`.
fn patched(lines: &[&str], fold: &str) -> Result<String> {
    let start = line(lines, 0, "argument cursor", |l| l.trim() == CURSOR)?;
    let open = line(lines, start + 1, "option loop", |l| l.trim() == LOOP)?;
    let end = line(lines, open + 1, "end of the option loop", |l| l == "    }")?;

    let mut out: Vec<String> = lines[..start].iter().map(|l| (*l).to_owned()).collect();
    out.push(
        "    // The built-in parser: each option tried in turn against the current word."
            .to_owned(),
    );
    let mut depth = 0i32;
    for l in &lines[start..=end] {
        // Each top-level statement of the built-in parser, compiled out by the feature.
        if depth == 0 && !l.trim().is_empty() && !l.trim().starts_with("//") {
            out.push("    #[cfg(not(feature = \"winnow-args\"))]".to_owned());
        }
        out.push((*l).to_owned());
        for c in l.chars() {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }
    }
    out.extend(
        [
            "",
            "    // winnow-args lexes the whole command line into options, in order;",
            "    // each is handled as the built-in parser handles it.",
            "    #[cfg(feature = \"winnow-args\")]",
            "    {",
            "        use crate::cmdline_winnow::{Item, utf8_arg};",
            "",
        ]
        .map(str::to_owned),
    );
    out.extend(fold.split('\n').map(|l| match l.is_empty() {
        true => String::new(),
        false => format!("        {l}"),
    }));
    out.push("    }".to_owned());
    out.extend(lines[end + 1..].iter().map(|l| (*l).to_owned()));

    let mut text = out.join("\n");
    for item in LEGACY_ITEMS {
        let at = format!("\n{item}");
        if text.matches(&at).count() != 1 {
            return Err(format!("mold's cmdline.rs has not exactly one `{item}`").into());
        }
        text = text.replace(
            &at,
            &format!("\n#[cfg_attr(feature = \"winnow-args\", allow(dead_code))]{at}"),
        );
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"fn parse_args() {
    let mut cursor = ArgCursor { args: raw_cmdline, index: 1 };
    while cursor.index < raw_cmdline.len() {
        if !cursor.current().as_encoded_bytes().starts_with(b"-") {
            a.inputs.push(PathBuf::from(&cursor.current()));
            cursor.index += 1;
            continue;
        }
        if cursor.read_flag("help") {
            print_help();
        }
        if read_arg!("o", true) || read_arg!("output", true) {
            a.output = raw_arg.into();
        } else if cursor.read_flag("build-id") || read_eq!("build-id") {
            a.build_id = arg;
        } else if cursor.read_switch("as-needed", "no-as-needed") {
            a.as_needed = value;
        } else if cursor.read_flag("s") || cursor.read_z_flag("nodump") {
            // Ignored.
        } else if read_z_arg!("stack-size") {
            a.stack_size = arg;
        } else {
            fatal!("unknown");
        }
    }
}
"#;

    fn parser(check: impl Fn(&Parser<'_>)) {
        let lines: Vec<&str> = SOURCE.split('\n').collect();
        let chain = chain(&lines).unwrap();
        check(&Parser::new(&chain).unwrap());
    }

    #[test]
    fn spellings_become_variants() {
        parser(|parser| {
            let item = parser.item_enum();
            for expected in [
                "    /// `-o`\n    #[arg(short = 'o')]\n    OutputShort(OsString),",
                "    /// `-output`\n    #[arg(long = \"output\")]\n    Output(OsString),",
                "    #[arg(long = \"build-id\", require_equals, default_missing = \"\\0\")]\n    BuildId(OsString),",
                "    #[arg(long = \"as-needed\")]\n    AsNeeded,",
                "    #[arg(short = 's')]\n    IgnoredShortSLower,",
                "    /// `-z stack-size=VALUE`.\n    #[arg(skip)]\n    ZStackSize(OsString),",
            ] {
                assert!(item.contains(expected), "{expected}\nnot in\n{item}");
            }
        });
    }

    #[test]
    fn z_keywords_map_to_their_arm() {
        parser(|parser| {
            let z_opt = parser.z_opt().unwrap();
            assert!(
                z_opt.contains("\"nodump\" => Item::IgnoredShortSLower,"),
                "{z_opt}"
            );
            assert!(
                z_opt.contains("return Some(Item::ZStackSize(OsString::from(value)));"),
                "{z_opt}"
            );
        });
    }

    #[test]
    fn arms_run_molds_bodies() {
        parser(|parser| {
            let patterns: Vec<&str> = parser
                .match_arms
                .iter()
                .map(|a| a.pattern.as_str())
                .collect();
            assert_eq!(
                patterns,
                [
                    "Item::OutputShort(value_os) | Item::Output(value_os)",
                    "Item::BuildId(value_os) if value_os.as_encoded_bytes() == b\"\\0\"",
                    "Item::BuildId(value_os)",
                    "Item::AsNeeded",
                    "Item::NoAsNeeded",
                    "Item::IgnoredShortSLower",
                    "Item::ZStackSize(value_os)",
                ]
            );
            assert_eq!(
                parser.match_arms[0].prelude,
                ["let raw_arg: &OsStr = value_os;"]
            );
            assert_eq!(
                parser.match_arms[2].prelude,
                ["let arg = utf8_arg(value_os, \"--build-id\");"]
            );
            assert_eq!(parser.match_arms[3].prelude, ["let value = true;"]);
        });
    }

    #[test]
    fn the_built_in_parser_is_kept_under_its_cfg() {
        let source = format!(
            "{SOURCE}\nfn match_option<T>() {{}}\nstruct ArgCursor<'a>;\nimpl<'a> ArgCursor<'a> {{}}\n"
        );
        let lines: Vec<&str> = source.split('\n').collect();
        let text = patched(&lines, "let x = 1;").unwrap();
        assert!(
            text.contains("    #[cfg(not(feature = \"winnow-args\"))]\n    while cursor.index"),
            "{text}"
        );
        assert!(text.contains("    #[cfg(feature = \"winnow-args\")]\n    {\n        use crate::cmdline_winnow::{Item, utf8_arg};\n\n        let x = 1;\n    }"), "{text}");
        assert_eq!(
            text.matches("#[cfg_attr(feature = \"winnow-args\", allow(dead_code))]")
                .count(),
            3
        );
    }

    #[test]
    fn names() {
        assert_eq!(camel("as-needed"), "AsNeeded");
        assert_eq!(camel("("), "OpenParen");
        assert_eq!(camel(":lto-pass2"), "LtoPass2");
        let mut used = HashSet::new();
        assert_eq!(unique("M".to_owned(), &mut used), "M");
        assert_eq!(unique("M".to_owned(), &mut used), "MX");
    }
}
