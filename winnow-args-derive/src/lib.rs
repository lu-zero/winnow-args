//! The derives of winnow-args: `Args`, `Subcommand`, `ValueEnum` and `Occurrence`.
//!
//! Each page lists every attribute its derive accepts. `#[winnow_args(...)]`
//! is `#[arg(...)]` where another derive claims `arg`.
//!
//! `#[derive(Args)]` generates one loop:
//!
//! 1. read the next item: a flag, a word, or `--`;
//! 2. `match` its name against the struct's flags and store the value;
//! 3. give a plain word to the next positional, in order. A word that names a
//!    subcommand selects it, and that variant's `Args` parses the rest;
//! 4. apply the defaults, check the rules and build the struct.
//!
//! The names are patterns of a `match`, which rustc compiles. No list of
//! parsers is walked.
//!
//! # Documentation
//!
//! When `WINNOW_ARGS_SPEC` is the absolute path of a directory, each derive
//! writes `CRATE/FILE/Type.toml` there. `FILE` is the type's source file. The
//! file records what help knows. A flatten or a subcommand is the other type's
//! path, written as the source writes it. A binary or an example writes under
//! `CRATE-bin`.
//!
//! `winnow-args-spec` reads those files for `winnow-args-man` and
//! `winnow-args-markdown`. `cargo check` is enough. Cargo compiles the crate
//! again when the variable changes.
//!
//! Two types of one name are distinguished by the modules of the source file,
//! then by the `mod name { }` blocks around the type. The file is read for
//! those blocks. `add::Opts` is the `Opts` of a module `add`. A bare `Opts` is
//! what a `use` in the file brings in, or the `Opts` the file declares.
//!
//! Where the modules and `use` lines leave the type unnamed,
//! `#[arg(spec = "AddOpts")]` files the type under that name, and the field
//! or variant that holds it repeats it. Two types of one name in one file
//! need this. The build fails when the path does not lead to the type: an
//! alias, a module under another name, or a `spec` missing on one side.
//!
//! Set the variable for the documentation build alone. A test or a doctest
//! would write its types next to the program's. rust-analyzer's macro server
//! writes nothing.
//!
//! A value's choices are filed under its type's name. A field fails the build
//! when that type is an alias, or implements `FromArg` by hand, until it
//! states `spec = "Name"` or `choices(...)`. Choices the type does not accept
//! fail the build again.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Ident, LitByteStr, LitChar, LitStr, PathArguments,
    Type, ext::IdentExt as _, parse_macro_input, spanned::Spanned as _,
};

mod spec;

/// Derive `Args` for a struct with named fields, or for a unit struct
/// (`struct Noop;`), which takes no words.
///
/// Attributes are `#[arg(...)]`, or `#[winnow_args(...)]` where another derive
/// claims `arg`. With no `short`, `long` or `plus`, a field is `--field-name`.
/// The doc comment is the help. A `` `quoted` ``, `*emphasized*` or
/// `**strong**` span in it is painted. Backticks are kept, and stars are
/// dropped where the span is painted.
///
/// ```ignore
/// use winnow_args::Args;
///
/// /// Copy files.
/// #[derive(Args, Debug)]
/// #[arg(name = "cp", version)]
/// struct Cp {
///     /// Explain what is done; repeat for more.
///     #[arg(short, long, count)]
///     verbose: u8,
///     /// Overwrite without asking.
///     #[arg(short, long, negate)]
///     force: bool,
///     /// Attributes to keep.
///     #[arg(long, delimiter = ',')]
///     preserve: Vec<String>,
///     /// Jobs to run.
///     #[arg(short, long, env = "CP_JOBS", default = "1")]
///     jobs: usize,
///     #[arg(positional)]
///     source: String,
///     #[arg(positional)]
///     dest: Option<String>,
/// }
///
/// let cp = Cp::parse_from(["-vvf", "--preserve=mode,links", "a", "--no-force", "b"])?;
/// assert_eq!((cp.verbose, cp.force, cp.jobs), (2, false, 1));
/// assert_eq!(cp.preserve, ["mode", "links"]);
/// assert_eq!((cp.source.as_str(), cp.dest.as_deref()), ("a", Some("b")));
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
/// The type says how many values a field takes. A repeated flag that takes one
/// value keeps the last. `values` and `delimiter` change a `Vec`.
///
/// | Type | Flag | Positional |
/// |---|---|---|
/// | `bool` | a switch | |
/// | an integer, with `count` | how many times the flag was given | |
/// | `T` | one value. Absent is an error without `env` or `default` | a required word |
/// | `Option<T>` | one value, or absent. With `require_equals`, a bare flag is `None` | an optional word |
/// | `Vec<T>` | one value each time the flag is given | every word left |
/// | `Option<bool>`, `plus` equal to `short` | `-x` is `Some(true)`, `+x` is `Some(false)` | |
///
/// # Struct options
///
/// Name and help:
///
/// - `name = "…"`: the program name in the usage line, where an empty name
///   means `argv[0]`. A completion script needs the name set.
/// - `version`, or `version = "…"`: add `-V` and `--version`. With no string,
///   the text is the crate's `CARGO_PKG_VERSION`.
/// - `about`: the description on `-h`. It defaults to the doc comment's first
///   paragraph.
/// - `long_about`: the description on `--help`. It defaults to the whole doc
///   comment, and `--help` shows `about` when this is empty.
/// - `after_help`: text after the lists, on `-h`.
/// - `after_long_help`: that text on `--help`. `--help` shows `after_help`
///   when this is empty.
/// - `disable_help_flag`, `disable_version_flag`, `disable_help_subcommand`:
///   leave that item out. `disable_help_short` keeps `--help` and leaves `-h` out.
/// - `arg_required_else_help`: an empty command line asks for help.
/// - `example = "tool build --release"`: one line in an Examples section after
///   the lists, shown as written. Help paints its subcommands, flags,
///   `<NAME>` placeholders and `# comment`; a man page sets the program,
///   subcommands and flags in bold and placeholders in italic. Repeat it for
///   another line.
///   `example("…", header = "…", help = "…")` sets a heading above the line
///   and the sentence `--help` shows for it. Either key may be left out.
/// - `next_help_heading = "…"`: the heading for fields that set no
///   `help_heading`. On a flattened struct, that heading is a section of the
///   page that takes its flags.
/// - `heading("Title", help = "…")`: the paragraph `--help` shows under that
///   heading, before the entries. A flattened struct's paragraph is used for
///   a title unless the parent gives one.
/// - `flatten_help`: each subcommand's usage line, description and arguments
///   replace the subcommand list. It needs a `subcommand` field.
/// - `default_subcommand_help`: the default subcommand's page follows this
///   one. It needs `default_subcommand`. The list marks that subcommand
///   `(default)` whenever one is set.
///
/// For the pages `winnow-args-spec` builds. Parsing ignores them:
///
/// - `effect = "read"`, `"write"` or `"destructive"`: what the command does.
/// - `output("json", default, framing = "json", help = "…")`: one form of the
///   output. Repeat it for another. A flattened struct's forms are included.
/// - `exit_code(2, "invalid atom")`: what an exit status means. Repeat it.
///   A flattened struct's codes are included.
///
/// The command line:
///
/// - `unknown_flags = "value"`: a flag-like word that names no flag is a
///   positional value. `"error"` is the default.
/// - `long_only`: a long name may be written with one dash (`getopt_long_only`).
/// - `plus_options`: `+abc` is a bundle of `+` options, for fields with `plus`.
/// - `restart_token = "…"`: that word starts the positionals again. Flags
///   resume and keep their values.
/// - `default_subcommand = "…"`: the subcommand used when the first word names
///   none. It needs a `subcommand` field.
/// - `default_subcommand_on_empty`: a line that names no subcommand and gives
///   no positional and no `--` runs the default subcommand too, with no words
///   of its own: `tool` and `tool -v` both run it. Not beside
///   `arg_required_else_help`.
/// - `default_subcommand_flags`: with `default_subcommand`, a flag this struct
///   does not take selects it, and it reads that flag (`tool -p pkg` is
///   `tool run -p pkg`). The words and flags after that one are the
///   subcommand's, and a word that names another subcommand is one of those
///   words. A flag of this struct without `global` comes first (`tool -qp pkg`).
/// - `group("name", required, multiple, message = "…")`: a group. A field
///   joins it with `group = "name"`. `required` means at least one member is
///   present. Without `multiple`, a second member conflicts with the first.
///   `message` is said in place of either error, `{0}` and `{1}` the two
///   members given, or the group's name and its members when none was.
/// - `validate = path`: a function such as
///   `fn check(cli: &Cli) -> Result<(), winnow_args::Error>`, called with the
///   struct once it is built and its rules hold. It returns
///   `Error::invalid` for a fault of several fields, or `Error::invalid_value`
///   for one value. A flattened struct's runs when that struct is built.
/// - `spec = "…"`: the name the pages file this type under, instead of its own.
///   A field or a variant that holds the type states the same name. See
///   [Documentation](crate#documentation).
///
/// # Field roles
///
/// A field is a flag unless it has one of these roles. `flatten`, `sequence`,
/// `unknown` and `skip` take no other option. `sequence` may add `unknown`.
/// `flatten` and `sequence` may add `spec`.
///
/// - `positional`: a plain word. The fields run required `T`, then `Option<T>`,
///   then one `Vec<T>` last. Another order is a compile error.
/// - `subcommand`: an enum that derives `Subcommand`, as `E` or `Option<E>`.
///   It takes no other option.
/// - `flatten`: another `Args` struct's flags, parsed in this loop, including
///   a struct it flattens. A shared spelling is a compile error. A `global`
///   flag there is global here. That struct is flags only: no positional,
///   subcommand, `sequence`, `unknown`, `keywords`, generic or
///   `unknown_flags = "value"`. This struct has no generic either.
/// - `sequence`: a `Vec` of an `Occurrence` enum. Its flags and words stay in
///   order, and its flags appear in this help. One per struct. A spelling
///   shared with this struct is a compile error.
/// - `sequence, unknown`: as `sequence`, and an unknown flag goes to the
///   enum's `unknown` variant.
/// - `unknown`: a `Vec<T>` of the flag-like words no flag takes, each word
///   whole. One per struct.
/// - `skip`: holds `Default::default()`. The command line does not set it.
///
/// # Flag names
///
/// - `short`, or `short = 'x'`: `-x`. With no letter, the field name's first
///   character. A second `short` is another letter, and help lists both.
/// - `long`, or `long = "name"`: `--name`, matched in full. With no string,
///   `_` in the field name is written `-`. Help lists the first `long` only.
/// - `alias = "…"`, or `alias("…", …)`: further long names. Help leaves them out.
/// - `negate`, or `negate = "no-name"`: `--no-name` sets the `bool` false.
///   With no string, the name is `no-` plus the long name. A leading `--` on
///   the string is dropped.
/// - `two_dashes`: with `long_only`, this long name keeps both dashes (`--omagic`).
/// - `prefix`: an ASCII `short` takes the rest of its word (`-lfoo`). The flag
///   takes a value.
/// - `plus = 'c'`: also `+c`, on a struct with `plus_options`. On `Option<bool>`
///   whose `short` is the same letter, `-c` is `Some(true)` and `+c` is
///   `Some(false)`.
/// - `global`: accepted after a subcommand word, at any depth.
///
/// # Values
///
/// A value is any `T: FromArg`. The table says how many the type takes.
///
/// - `from_str`: parse with `FromStr` instead of `FromArg`, for a type from
///   another crate (`camino::Utf8PathBuf`, `url::Url`). The error text is that
///   parse error. The type brings no choices and no completion hint: set
///   `choices` or `hint`.
/// - `count`: an integer. Each time the flag is given adds one.
/// - `value_name = "…"`: the placeholder in help. With none, the field name
///   in capitals.
/// - `delimiter = ','`: split one value of a `Vec` on this ASCII character.
/// - `values = N`: each time a `Vec` flag is given, it takes `N` words,
///   whatever they look like.
/// - `values = 1..`, or `values = 2..=3`: at least, and at most, that many
///   words each time. After the first, a word is taken while it does not look
///   like a flag. A positional after this flag needs a flag, or `--`, before it.
/// - `value_terminator = ";"`: the word that ends a `values` range. The word
///   is dropped.
/// - `choices("a", "b")`: the only values accepted.
/// - `env = "VAR"`: used when the command line left the field unset. On
///   `Option<T>` the field is then `Some`. A switch is true unless the
///   variable is empty, `0`, `false`, `no` or `off`. A `count` variable that
///   does not parse is left unset.
/// - `default = "…"`: parsed as the field's value when the command line, `env`
///   and `default_if` left it unset. On `Option<T>` the field is then `Some`.
/// - `default_fn = path`: a function returning the field's value, or the whole
///   list for a `Vec`, called at the same point as `default`. Setting both is
///   a compile error. `default_note = "…"` is the text help shows for it.
/// - `default_from(input, |input: &PathBuf| input.with_extension("out"))`: a default
///   made from the fields named, by the function or closure last, called with
///   a reference to each as declared. On a struct without generics a closure
///   needs no types: `|input| input.with_extension("out")`. It is computed after `env`, `default_if`
///   and the other defaults, a field after those it is made from; one that is
///   required and still unset leaves it unset. Help shows "from `input`", or
///   its `default_note`.
/// - `default_if("--flag", "v")`: this default when `--flag` was given.
///   `default_if("--flag", "x", "v")` when that field's value equals `x`
///   (`PartialEq`). Tried after `env` and before `default`, in the order
///   written. The first string names a field, as under [Rules](#rules).
/// - `default_missing = "…"`: the value when the flag was given without one.
///   The value is then optional.
/// - `require_equals`: the value is attached (`--name=v`, `-nv`). With
///   `default_missing`, a bare flag leaves the next word alone. On `Option<T>`,
///   a bare flag is `None`.
/// - `keep_equals`: an attached short value keeps a leading `=` (`-L=dir`).
/// - `allow_hyphen_values`: the next word is the value when it looks like a
///   flag, and when it is `--`.
/// - `allow_negative_numbers`: a negative number is a value, on a flag or a
///   positional.
/// - `keywords`: the flag's type derives `Args`, and each value is a flag of
///   that type (`-z now`).
/// - `spec = "…"`: the name the pages find this field's type under, when that
///   is not the type's own name. The type has `spec`, or this field names it
///   through an alias. `flatten` and `sequence` take it too.
///
/// # Positionals
///
/// - `double_dash = "…"`: `"optional"` (default) takes a word on either side
///   of `--`. `"required"` takes every word after `--` and none before: at
///   most one, outside the order in [Field roles](#field-roles).
///   `"automatic"` stops flags once the field has a value. `"preserve"` takes
///   a `--` that reaches it as a value, and that `--` stops nothing.
/// - `stop_flags`: once it has a value, flags stop, for any `double_dash`.
///
/// # Rules
///
/// A string here names another field: `"--long"`, `"-s"`, a positional's
/// placeholder, or the field's name.
///
/// - `required`: after `env` and the defaults, the field holds a value. A
///   `Vec` without `required` may be empty.
/// - `required_unless("…", …)`: required unless one of them has a value.
/// - `required_unless_all("…", …)`: required unless all of them have a value.
/// - `required_if("--flag")`: required when `--flag` was given.
///   `required_if("--flag", "x")`: when its value equals `x` (`PartialEq`).
///   Judged on the command line and `env`, before the defaults. Repeat it for
///   another condition: any one makes the field required.
/// - `requires_if("x", "--other")`: when this field's value equals `x`,
///   `--other` must have one too.
/// - `exclusive`: given on the command line or by `env`, no other field of
///   this struct may be given with it. Defaults do not count, and neither do
///   the flags of a flattened struct, of a `sequence` or of an `unknown` field.
/// - `conflicts("…", …)`: the parse fails when this field and any of those are
///   both supplied.
/// - `overrides("…", …)`: of this field and those, the last one supplied is kept.
/// - `requires("…", …)`: when this field is supplied, each of those has a value.
/// - `message = "…"` after the names of `conflicts` or `requires`: said in
///   place of the default error, `{0}` this field and `{1}` the other, each by
///   its long name, else its letter. The error's kind is unchanged.
/// - `group = "name"`: this field is a member of that group.
///
/// # Help
///
/// - `help = "…"`: replaces the first paragraph of the doc comment, and the
///   whole text when `long_help` is absent.
/// - `long_help = "…"`: replaces the whole text.
/// - `verbatim_doc_comment`: the doc comment's lines are kept as written.
///   Otherwise the lines of a paragraph are joined, for help to wrap to the
///   terminal; an indented line is kept. A struct, a `Subcommand` enum and a
///   variant take it too.
/// - `help_heading = "…"`: list the field under this heading. Fields with none
///   are listed under "Options", or under the struct's `next_help_heading`.
/// - `warning = "…"`: `--help` shows it as `Warning: …`.
/// - `hide_default_value`, `hide_env`: help and the pages leave out the
///   default, or the environment variable; both still apply.
/// - `display_order = n`: list the flag at place `n` among the fields, which
///   otherwise keep their declaration's place, counted from 0. It goes before
///   the field already there. Parsing is unchanged.
/// - `effect = "read"`, `"write"` or `"destructive"`: what giving the flag
///   does, for the pages.
/// - `hint = "file"`, `"dir"`, `"command"` or `"none"`: what a completing
///   shell offers for the value. With no hint, `FromArg::HINT` decides: a
///   file for `PathBuf`, `String` and `OsString`, and nothing for any other type.
/// - `complete = path`: `fn(&str) -> Vec<winnow_args::complete::Candidate>`,
///   called with the text typed so far. It is the value's only completion, so
///   the field has no `hint`. Each candidate is one word, and the script
///   inserts it as it is.
/// - `hide`: left out of help and of completion.
#[proc_macro_derive(Args, attributes(arg, winnow_args))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    finish(expand(&input))
}

/// The expansion, or its error, and what tells Cargo to compile the crate
/// again when `WINNOW_ARGS_SPEC` changes.
fn finish(expanded: syn::Result<TokenStream2>) -> TokenStream {
    let expanded = expanded.unwrap_or_else(syn::Error::into_compile_error);
    quote! {
        #expanded
        const _: ::core::option::Option<&str> = ::core::option_env!("WINNOW_ARGS_SPEC");
    }
    .into()
}

/// Derive `Subcommand` for an enum of subcommands. The enum also implements
/// `Args`, so it can be a whole command line.
///
/// A variant is a unit, holds one `Args` type (which may be `Box`ed), or has
/// named fields. A parent struct holds the enum in a `subcommand` field. The
/// doc comment is the help. Used as the whole command line, the enum has no
/// program name and no version. A completion script needs a struct with `name`.
///
/// ```ignore
/// use winnow_args::{Args, Subcommand};
///
/// #[derive(Args, Debug, PartialEq)]
/// struct Add {
///     #[arg(positional)]
///     paths: Vec<String>,
/// }
///
/// #[derive(Subcommand, Debug, PartialEq)]
/// enum Command {
///     /// Stage files.
///     Add(Add),
///     /// Show what changed.
///     #[arg(alias = "st")]
///     Status,
/// }
///
/// #[derive(Args, Debug)]
/// struct Git {
///     /// Say less.
///     #[arg(short, long, global)]
///     quiet: bool,
///     #[arg(subcommand)]
///     command: Command,
/// }
///
/// let git = Git::parse_from(["add", "-q", "a", "b"])?;
/// assert!(git.quiet);
/// assert_eq!(git.command, Command::Add(Add { paths: vec!["a".into(), "b".into()] }));
/// assert_eq!(Git::parse_from(["st"])?.command, Command::Status);
///
/// match git.command {
///     Command::Add(Add { paths }) => println!("staging {paths:?}"),
///     Command::Status => println!("on branch main"),
/// }
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
/// # Enum
///
/// - `rename_all = "kebab-case"` (default), `"lowercase"`, `"UPPERCASE"`,
///   `"snake_case"` or `"verbatim"`. `DryRun` is then `dry-run`, `dryrun`,
///   `DRYRUN`, `dry_run` or `DryRun`. A variant's `name` replaces this.
/// - `spec = "…"`: the name the pages file this enum under, instead of its own.
/// - `run` or `run_async`, once. The enum implements `Run` or `RunAsync` by
///   calling that method on the held type. Every variant holds a type that
///   implements the trait, two may hold the same type, and all share one
///   `Output`. The enum implements it for every context those types take.
///
/// # Variants
///
/// - `name = "…"`: the word that selects it, instead of `rename_all`.
/// - `alias = "…"`, or `alias("…", …)`: other words, shown in the list.
///   `alias_hidden` is not shown.
/// - `hide`: left out of help and of completion.
/// - `spec = "…"`: the name the pages find this variant's type under. A unit
///   holds nothing, and `spec` there is an error.
/// - `help = "…"`: the one line the list shows. It replaces the first
///   paragraph of the doc comment. A unit's `--help` shows the whole doc
///   comment when this is absent, and this line when it is set.
/// - `long_help = "…"`: on a variant with named fields, that command's
///   `long_about`. The command list still shows one line. On a unit, or a
///   variant that holds a type, `long_help` is unused.
///
/// A variant with named fields is a command. Its fields are declared as a
/// struct's are, and its doc comment and other options (`example`,
/// `flatten_help`, `group`, …) are that command's. Its `validate` function
/// takes the enum. An enum with `run` has no such variant. A generic enum
/// holds a struct that derives `Args`.
///
/// ```ignore
/// #[derive(Subcommand, Debug, PartialEq)]
/// enum Command {
///     /// Compile the package
///     Build {
///         #[arg(long)]
///         release: bool,
///     },
///     Clean,
/// }
///
/// let cli = Cli::parse_from(["build", "--release"])?;
/// assert_eq!(cli.command, Command::Build { release: true });
/// # Ok::<(), winnow_args::Error>(())
/// ```
#[proc_macro_derive(Subcommand, attributes(arg, winnow_args))]
pub fn derive_subcommand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    finish(expand_subcommand(&input))
}

/// One `match` on the subcommand's name; the variant's `Args` parses the rest.
fn expand_subcommand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Subcommand` can only be derived for enums",
        ));
    };
    let mut runs = None;
    let case = rename_all(input, Some(&mut runs))?;
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let mut patterns = Vec::new();
    let mut subs = Vec::new();
    // Each variant and the type it holds, for `run`.
    let mut held: Vec<(&Ident, Type, bool)> = Vec::new();
    let spec_on = spec::enabled();
    let filed = filed(input)?;
    let identity = spec::identity(input.ident.span(), &filed);
    let mut spec_variants = Vec::new();
    let mut spec_checks = Vec::new();
    // The struct each variant with fields is parsed as.
    let mut structs = Vec::new();
    for variant in &data.variants {
        let ident = &variant.ident;
        let info = variant_names(variant, &mut names, case)?;
        let pattern = byte_patterns(&info.names);
        let primary = LitStr::new(&info.names[0], Span::call_site());
        let fields_of = match &variant.fields {
            Fields::Named(_) => Some(variant_struct(input, variant)?),
            _ => None,
        };
        let inner = match (&variant.fields, &fields_of) {
            (Fields::Unnamed(fields), _) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                Some(boxed(ty).unwrap_or(ty).clone())
            }
            (_, Some((name, ..))) => Some(syn::parse_quote!(#name)),
            _ => None,
        };
        // The name a variant with fields is filed under, when it is not its
        // struct's own.
        let mut filed_as = None;
        let mut validate = None;
        if let Some((_, tokens, stated, check)) = fields_of {
            structs.push(tokens);
            filed_as = stated;
            validate = check;
            if runs.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "`run` calls the type a variant holds, and this variant has fields of its own",
                ));
            }
        }
        match &inner {
            Some(_) if matches!(variant.fields, Fields::Named(_)) => {}
            Some(ty) => {
                let is_boxed = matches!(&variant.fields, Fields::Unnamed(f) if boxed(&f.unnamed[0].ty).is_some());
                held.push((ident, ty.clone(), is_boxed));
            }
            None if runs.is_some() => {
                return Err(syn::Error::new(
                    ident.span(),
                    "`run` calls the type a variant holds, and this variant holds none",
                ));
            }
            None => {}
        }
        // The whole doc comment, when the description is its first paragraph.
        let unit_long_about = inner.is_none().then(|| {
            let (doc, long_doc) = docs(&variant.attrs);
            if doc == info.about {
                long_doc
            } else {
                info.about.clone()
            }
        });
        let command = match (&inner, &unit_long_about) {
            (Some(ty), _) => quote!(<#ty as ::winnow_args::Args>::HELP),
            (None, long_about) => {
                let long_about = text(long_about.as_deref().unwrap_or_default());
                let about = text(&info.about);
                quote!(&::winnow_args::help::Command {
                    about: #about,
                    long_about: #long_about,
                    help_flag: true,
                    help_short: true,
                    ..::winnow_args::help::Command::EMPTY
                })
            }
        };
        let about = match (&inner, info.about.is_empty()) {
            (Some(ty), true) => quote!(<#ty as ::winnow_args::Args>::HELP.about),
            _ => text(&info.about),
        };
        let shown = &info.names[1..=info.shown];
        let all = &info.names;
        let hide = info.hide;
        if spec_on {
            let ty = match &inner {
                Some(ty) => {
                    let stated = spec_name(&variant.attrs)?.or(filed_as);
                    let name = spec_type(stated, ty)?;
                    spec_checks.extend(spec_filed(&input.generics, quote!(Args), ty, &name));
                    Some(name)
                }
                None => None,
            };
            spec_variants.push(spec::VariantDoc {
                name: info.names[0].clone(),
                aliases: shown.to_vec(),
                hidden_aliases: info.names[info.shown + 1..].to_vec(),
                about: info.about.clone(),
                long_about: unit_long_about.unwrap_or_default(),
                hide: info.hide,
                ty,
            });
        }
        subs.push(quote! {
            ::winnow_args::help::Sub {
                name: #primary,
                aliases: &[#(#shown),*],
                names: &[#(#all),*],
                command: #command,
                about: #about,
                hide: #hide,
            }
        });
        // Read here, so a payload only printed is not dead code.
        let read_payload = quote! {
            |__item| match __item {
                Self::#ident(__read) => {
                    let _ = __read;
                }
                #[allow(unreachable_patterns)]
                _ => {}
            }
        };
        let parse = match &variant.fields {
            Fields::Unit => {
                let index = subs.len() - 1;
                quote! {
                    __wa::finish_with(
                        __input,
                        __globals,
                        <Self as ::winnow_args::Args>::HELP.subcommands[#index].command,
                    )
                    .map(|()| Self::#ident)
                }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                match boxed(ty) {
                    Some(inner) => quote! {
                        <#inner as ::winnow_args::Args>::parse_argv_with(__input, __globals)
                            .map(|v| Self::#ident(::std::boxed::Box::new(v)))
                            .inspect(#read_payload)
                    },
                    None => quote! {
                        <#ty as ::winnow_args::Args>::parse_argv_with(__input, __globals)
                            .map(Self::#ident)
                            .inspect(#read_payload)
                    },
                }
            }
            Fields::Named(fields) => {
                let ty = inner.as_ref().expect("a struct for the fields");
                let names = fields.named.iter().map(|field| &field.ident);
                let names: Vec<_> = names.collect();
                let check = validate.as_ref().map(|path| {
                    quote! {
                        .and_then(|__built| {
                            #path(&__built)?;
                            ::core::result::Result::Ok(__built)
                        })
                    }
                });
                quote! {
                    <#ty as ::winnow_args::Args>::parse_argv_with(__input, __globals)
                        .map(|#ty { #(#names),* }| Self::#ident { #(#names),* })
                        #check
                }
            }
            _ => {
                return Err(syn::Error::new(
                    variant.span(),
                    "a subcommand variant is a unit, holds one `Args` type or has named fields",
                ));
            }
        };
        arms.push(quote!(#pattern => (#parse).map_err(|e| e.within(#primary)),));
        patterns.push(pattern);
    }
    let (about_text, long_about_text) = docs(&input.attrs);
    let (about, long_about) = (text(&about_text), text(&long_about_text));
    if spec_on {
        spec::write_subcommands(
            input.ident.span(),
            &filed,
            &about_text,
            &long_about_text,
            &spec_variants,
        )?;
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let run_impl = runs.map(|runs| run_impl(input, runs, &held));
    Ok(quote! {
        #(#structs)*
        #(#spec_checks)*
        #run_impl
        impl #impl_generics ::winnow_args::Subcommand for #name #ty_generics #where_clause {
            const SPEC: &'static str = #identity;

            #[inline]
            fn has(__name: &[u8]) -> bool {
                matches!(__name, #(#patterns)|*)
            }

            fn parse_subcommand(
                __name: &[u8],
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                match __name {
                    #(#arms)*
                    _ => ::core::result::Result::Err(__wa::Error::unexpected_arg(
                        __input.offset(),
                        ::std::string::String::from_utf8_lossy(__name),
                    )),
                }
            }
        }

        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            const SPEC: &'static str = #identity;

            // Compiled here and never inlined, so a program in another crate has
            // no copy of the help.
            #[inline(never)]
            fn help_data() -> &'static ::winnow_args::help::Command {
                <Self as ::winnow_args::Args>::HELP
            }

            const HELP: &'static ::winnow_args::help::Command = &::winnow_args::help::Command {
                about: #about,
                long_about: #long_about,
                subcommands: &[#(#subs),*],
                subcommand_required: true,
                help_flag: true,
                help_short: true,
                ..::winnow_args::help::Command::EMPTY
            };

            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                let __help = <Self as ::winnow_args::Args>::HELP;
                // Inherited global flags may come before the subcommand's name.
                loop {
                    if __input.is_empty() {
                        return ::core::result::Result::Err(
                            __wa::Error::missing_subcommand(__input.offset()),
                        );
                    }
                    let __arg = __wa::arg(__input)?;
                    match __arg {
                        __wa::Arg::Word(__word) => {
                            if <Self as __wa::Subcommand>::has(&**__word.value) {
                                return <Self as __wa::Subcommand>::parse_subcommand(
                                    &**__word.value, __input, __globals,
                                );
                            }
                            if &**__word.value == b"help" {
                                return ::core::result::Result::Err(__wa::help_word(__help, __input));
                            }
                        }
                        __wa::Arg::Long(_) | __wa::Arg::Short(_)
                            if __globals.bind(&__arg, __input)? =>
                        {
                            continue;
                        }
                        __wa::Arg::Long(__flag) if __flag.name == b"help" => {
                            return ::core::result::Result::Err(__wa::Error::help(__help, true));
                        }
                        __wa::Arg::Short(__flag) if __flag.letter == 'h' => {
                            return ::core::result::Result::Err(__wa::Error::help(__help, false));
                        }
                        _ => {}
                    }
                    return ::core::result::Result::Err(__arg.unexpected());
                }
            }
        }
    })
}

/// The struct a variant with named fields is parsed as: its name, and its
/// definition with the `Args` it derives. The variant's doc comment and the
/// options that are not about its name are the struct's.
fn variant_struct(
    input: &DeriveInput,
    variant: &syn::Variant,
) -> syn::Result<(Ident, TokenStream2, Option<String>, Option<syn::Expr>)> {
    let Fields::Named(fields) = &variant.fields else {
        unreachable!("called for a variant with named fields");
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            variant.ident.span(),
            "a variant with fields in a generic enum is not supported: hold a struct that derives `Args`",
        ));
    }
    let name = quote::format_ident!("__{}_{}", input.ident, variant.ident);
    let mut options: Vec<syn::Meta> = Vec::new();
    // The enum calls it with the variant built, not the struct.
    let mut validate = None;
    for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
        let metas = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;
        for meta in metas {
            let key = meta.path().get_ident().map(Ident::to_string);
            match (key.as_deref(), &meta) {
                // Its name among the subcommands, which the enum reads.
                (Some("name" | "alias" | "alias_hidden" | "hide"), _) => {}
                (Some("validate"), syn::Meta::NameValue(pair)) => {
                    if validate.is_some() {
                        return Err(syn::Error::new_spanned(
                            &pair.path,
                            "`validate` is given twice",
                        ));
                    }
                    validate = Some(pair.value.clone());
                }
                (Some("help"), syn::Meta::NameValue(pair)) => {
                    let text = &pair.value;
                    options.push(syn::parse_quote!(about = #text));
                }
                (Some("long_help"), syn::Meta::NameValue(pair)) => {
                    let text = &pair.value;
                    options.push(syn::parse_quote!(long_about = #text));
                }
                _ => options.push(meta),
            }
        }
    }
    // An enum filed under another name has its variants filed under it too,
    // so two enums of one name in a file are told apart as they are. A
    // variant that states its own name keeps it: `spec` is among the options.
    let enum_name = filed(input)?;
    let stated = (spec_name(&variant.attrs)?.is_none() && input.ident != enum_name.as_str())
        .then(|| format!("__{enum_name}_{}", variant.ident));
    if let Some(stated) = &stated {
        options.push(syn::parse_quote!(spec = #stated));
    }
    let docs = variant.attrs.iter().filter(|a| a.path().is_ident("doc"));
    let docs: Vec<_> = docs.collect();
    let named = &fields.named;
    let derived: DeriveInput = syn::parse_quote! {
        #(#docs)*
        #[arg(#(#options),*)]
        struct #name { #named }
    };
    let args = expand(&derived)?;
    // The definition has no derive, so no attribute of ours or of another
    // derive of the enum: only what the compiler itself reads.
    let plain = named.iter().map(|field| {
        let attrs = field.attrs.iter().filter(|a| a.path().is_ident("cfg"));
        let (ident, ty) = (&field.ident, &field.ty);
        quote!(#(#attrs)* #ident: #ty)
    });
    let tokens = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #name { #(#plain),* }
        #args
    };
    Ok((name, tokens, stated, validate))
}

/// `Run` or `RunAsync` for a `Subcommand` enum: the command it holds is run,
/// with whatever context it takes. Every variant's type produces the same.
fn run_impl(input: &DeriveInput, runs: Runs, held: &[(&Ident, Type, bool)]) -> TokenStream2 {
    if held.is_empty() {
        // An enum with no variant holds nothing to run: it is never built.
        return quote!();
    }
    let name = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    // The enum's own parameters, without the defaults an `impl` may not state.
    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        match param {
            syn::GenericParam::Type(param) => (param.eq_token, param.default) = (None, None),
            syn::GenericParam::Const(param) => (param.eq_token, param.default) = (None, None),
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    let params = generics.params.iter();
    let (the_trait, method) = match runs {
        Runs::Sync => (quote!(::winnow_args::run::Run), quote!(run)),
        Runs::Async => (quote!(::winnow_args::run::RunAsync), quote!(run_async)),
    };
    // One output for all: a type two variants hold is then bound twice alike.
    let bounds = held
        .iter()
        .map(|(_, ty, _)| quote!(#ty: #the_trait<__C, Output = __O>));
    let own_bounds = where_clause
        .iter()
        .flat_map(|clause| clause.predicates.iter());
    let arms = held.iter().map(|(ident, ty, is_boxed)| {
        let command = match is_boxed {
            true => quote!(*__command),
            false => quote!(__command),
        };
        let call = quote!(<#ty as #the_trait<__C>>::#method(#command, __context));
        match runs {
            Runs::Sync => quote!(Self::#ident(__command) => #call,),
            Runs::Async => quote!(Self::#ident(__command) => #call.await,),
        }
    });
    let function = match runs {
        Runs::Sync => quote!(fn run(self, __context: __C) -> Self::Output),
        Runs::Async => quote!(async fn run_async(self, __context: __C) -> Self::Output),
    };
    quote! {
        #[automatically_derived]
        // Whether the future is `Send` is the commands' to say, not this `match`'s.
        #[allow(clippy::future_not_send)]
        impl<#(#params,)* __C, __O> #the_trait<__C> for #name #ty_generics
        where
            #(#own_bounds,)*
            #(#bounds,)*
        {
            type Output = __O;

            #function {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

/// Derive `Occurrence` for an enum of flags and positionals. An
/// `#[arg(sequence)]` field of an `Args` struct keeps them in command-line order.
///
/// A variant is a unit (a switch) or holds one value. `Spanned<T>` records the
/// offset and whether the value was attached. With no `short`, `long` or
/// `alias`, the name is kebab case (`AsNeeded` is `--as-needed`). The doc
/// comment is the help. A spelling used twice, here or on the parent, is a
/// compile error.
///
/// ```ignore
/// use winnow_args::{Args, Occurrence};
///
/// #[derive(Occurrence, Debug, PartialEq)]
/// enum Item {
///     /// Link what follows only if needed.
///     #[arg(long)]
///     AsNeeded,
///     /// Search for a library.
///     #[arg(short = 'l', prefix)]
///     Library(String),
///     #[arg(positional, value_name = "FILE")]
///     Input(String),
/// }
///
/// #[derive(Args, Debug)]
/// struct Ld {
///     #[arg(short, long)]
///     output: Option<String>,
///     #[arg(sequence)]
///     items: Vec<Item>,
/// }
///
/// let ld = Ld::parse_from(["a.o", "--as-needed", "-lm", "-o", "out", "b.o"])?;
/// assert_eq!(ld.output.as_deref(), Some("out"));
/// assert_eq!(
///     ld.items,
///     [
///         Item::Input("a.o".into()),
///         Item::AsNeeded,
///         Item::Library("m".into()),
///         Item::Input("b.o".into()),
///     ]
/// );
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
/// # Enum
///
/// - `spec = "…"`: the name the pages file this enum under, instead of its own.
///
/// The default for every variant that holds a value:
///
/// - `allow_hyphen_values`: the next word is the value when it looks like a
///   flag, and when it is `--`.
/// - `keep_equals`: an attached short value keeps a leading `=`.
///
/// # Names
///
/// - `short = 'x'`: `-x`. One letter.
/// - `long`, or `long = "name"`: `--name`, matched in full. With no string,
///   the variant's name in kebab case. A second `long` is another name, and
///   help lists the first.
/// - `alias = "…"`, or `alias("…", …)`: more long names. Help lists an alias
///   only when it is the first long name.
/// - `two_dashes`: with the parent's `long_only`, this name keeps both dashes.
/// - `prefix`: an ASCII `short` takes the rest of its word (`-lfoo`). The
///   variant holds a value.
///
/// # Values
///
/// - `value_name = "…"`: the placeholder in help. With none, a flag's is
///   `VALUE` and a positional's is the variant's name in capitals.
/// - `require_equals`: the value is attached (`--name=v`, `-nv`). On
///   `Option<T>`, a bare flag is `None`.
/// - `default_missing = "…"`: the value when the flag was given without one.
/// - `allow_negative_numbers`: a negative number is a value.
/// - `spec = "…"`: the name the pages find this value's type under, when that
///   is not the type's own name.
///
/// A variant may also set `allow_hyphen_values` or `keep_equals`.
///
/// # Roles
///
/// At most one `positional`, one `unknown` and one `bundle`. Each holds its value.
///
/// - `positional`: each plain word is this variant. The parent then has no
///   positional of its own.
/// - `unknown`: a flag that no variant names, kept whole. The parent's field
///   is `sequence, unknown`.
/// - `bundle`: a word of several short flags (`-sS`), the whole word, ahead of
///   its letters.
/// - `skip`: the program builds it. It is never parsed and is left out of help.
#[proc_macro_derive(Occurrence, attributes(arg, winnow_args))]
pub fn derive_occurrence(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    finish(expand_occurrence(&input))
}

/// Derive `FromArg` for an enum of unit variants. The value's bytes select a
/// variant. Help lists each visible variant's name as a possible value.
///
/// A variant holds no data.
///
/// ```ignore
/// use winnow_args::{Args, ErrorKind, ValueEnum};
///
/// #[derive(ValueEnum, Debug, PartialEq)]
/// enum Color {
///     Always,
///     #[arg(alias = "no")]
///     Never,
///     #[arg(name = "auto")]
///     WhenTerminal,
/// }
///
/// #[derive(Args, Debug)]
/// struct Cli {
///     #[arg(long)]
///     color: Option<Color>,
/// }
///
/// assert_eq!(Cli::parse_from(["--color=auto"])?.color, Some(Color::WhenTerminal));
/// assert_eq!(Cli::parse_from(["--color", "no"])?.color, Some(Color::Never));
/// let error = Cli::parse_from(["--color=red"]).unwrap_err();
/// assert_eq!(error.kind(), ErrorKind::InvalidChoice);
/// # Ok::<(), winnow_args::Error>(())
/// ```
///
/// - `rename_all = "kebab-case"` (default), `"lowercase"`, `"UPPERCASE"`,
///   `"snake_case"` or `"verbatim"`. `DryRun` is then `dry-run`, `dryrun`,
///   `DRYRUN`, `dry_run` or `DryRun`. A variant's `name` replaces this.
/// - `spec = "…"`: the name the pages file the choices under, instead of the
///   enum's own.
/// - `name = "…"`: the value that selects this variant.
/// - `alias = "…"`, `alias("…", …)` or `alias_hidden`: other values that
///   select it. Help and errors list the primary name only.
/// - `hide`: the value is accepted. Help and errors omit its name.
/// - `help = "…"` and `long_help = "…"`: accepted, and unused.
#[proc_macro_derive(ValueEnum, attributes(arg, winnow_args))]
pub fn derive_value_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    finish(expand_value_enum(&input))
}

/// `from_arg`: a `match` on the long name and one on the letter, each arm
/// building its variant; `from_word`: the positional variant, if any.
fn expand_occurrence(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Occurrence` can only be derived for enums",
        ));
    };
    let (mut long_arms, mut short_arms) = (Vec::new(), Vec::new());
    let (mut single_dash, mut prefixes) = (Vec::new(), Vec::new());
    let mut letters = Vec::new();
    let (mut word, mut unknown, mut bundle) = (None, None, None);
    let mut seen = std::collections::HashMap::new();
    // What help lists: one row a variant, in declaration order.
    let mut help_items = Vec::new();
    let spec_on = spec::enabled();
    let filed = filed(input)?;
    let identity = spec::identity(input.ident.span(), &filed);
    let mut spec_rows = Vec::new();
    let mut spec_checks = Vec::new();
    // `allow_hyphen_values` and `keep_equals` on the enum: every value
    // variant's default.
    let (mut enum_hyphen_values, mut enum_keep_equals) = (false, false);
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("allow_hyphen_values") {
                enum_hyphen_values = true;
                Ok(())
            } else if meta.path.is_ident("keep_equals") {
                enum_keep_equals = true;
                Ok(())
            } else if skip_spec(&meta)? {
                Ok(())
            } else {
                Err(meta.error("expected `allow_hyphen_values`, `keep_equals` or `spec`"))
            }
        })?;
    }
    for variant in &data.variants {
        let ident = &variant.ident;
        let (mut short, mut longs, mut positional) = (None, Vec::new(), false);
        let (mut two_dashes, mut prefix, mut value_name) = (false, false, None);
        let (mut hyphen_values, mut negative_numbers) = (enum_hyphen_values, false);
        let (mut require_equals, mut default_missing) = (false, None);
        let mut keep_equals = enum_keep_equals;
        let (mut skip, mut is_unknown, mut is_bundle) = (false, false, false);
        let stated = spec_name(&variant.attrs)?;
        for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("allow_hyphen_values") {
                    hyphen_values = true;
                } else if meta.path.is_ident("allow_negative_numbers") {
                    negative_numbers = true;
                } else if meta.path.is_ident("require_equals") {
                    require_equals = true;
                } else if meta.path.is_ident("keep_equals") {
                    keep_equals = true;
                } else if meta.path.is_ident("default_missing") {
                    default_missing = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("bundle") {
                    is_bundle = true;
                } else if meta.path.is_ident("unknown") {
                    is_unknown = true;
                } else if meta.path.is_ident("short") {
                    if short.is_some() {
                        return Err(meta.error("an `Occurrence` variant has one `short`"));
                    }
                    short = Some(meta.value()?.parse::<LitChar>()?.value());
                } else if meta.path.is_ident("long") {
                    longs.push(if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<LitStr>()?.value()
                    } else {
                        kebab_case(&ident.to_string())
                    });
                } else if meta.path.is_ident("alias") {
                    longs.extend(aliases(&meta)?);
                } else if meta.path.is_ident("positional") {
                    positional = true;
                } else if meta.path.is_ident("two_dashes") {
                    two_dashes = true;
                } else if meta.path.is_ident("prefix") {
                    prefix = true;
                } else if meta.path.is_ident("value_name") {
                    value_name = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if skip_spec(&meta)? || meta.path.is_ident("verbatim_doc_comment") {
                } else {
                    return Err(meta.error(
                        "expected `short`, `long`, `alias`, `positional`, `two_dashes`, `prefix`, \
                         `value_name`, `verbatim_doc_comment`, `allow_hyphen_values`, `allow_negative_numbers`, \
                         `require_equals`, `keep_equals`, `default_missing`, `skip`, `unknown` or `bundle`",
                    ));
                }
                Ok(())
            })?;
        }
        let takes = match &variant.fields {
            Fields::Unit => None,
            Fields::Unnamed(f) if f.unnamed.len() == 1 => Some(&f.unnamed[0].ty),
            _ => {
                return Err(syn::Error::new(
                    variant.span(),
                    "an `Occurrence` variant is a unit (a switch) or holds one value",
                ));
            }
        };
        if skip {
            // Built by the program itself (from a `-z` keyword, say), never parsed.
            continue;
        }
        // A `Spanned<T>` value also records where it was and whether it was attached.
        let spanned = takes.and_then(|ty| {
            last_segment(ty)
                .filter(|s| s.ident == "Spanned")
                .and_then(inner)
        });
        // An `Option<T>` field takes `T`, and a bare flag is `None` under
        // `require_equals`, not a converted default.
        let optional = takes.and_then(|ty| {
            last_segment(ty)
                .filter(|s| s.ident == "Option")
                .and_then(inner)
        });
        let wrap = |value: TokenStream2, offset: TokenStream2, attached: TokenStream2| {
            if spanned.is_some() {
                quote! {
                    ::winnow_args::value::Spanned {
                        value: #value,
                        offset: #offset,
                        attached: #attached,
                    }
                }
            } else {
                value
            }
        };
        if positional || is_unknown || is_bundle {
            let Some(ty) = takes else {
                return Err(syn::Error::new(
                    variant.span(),
                    "a positional, `unknown` or `bundle` variant holds its value",
                ));
            };
            let ty = spanned.unwrap_or(ty);
            let slot = if positional {
                &mut word
            } else if is_unknown {
                &mut unknown
            } else {
                &mut bundle
            };
            if slot.is_some() {
                return Err(syn::Error::new(
                    variant.span(),
                    "at most one positional, one `unknown` and one `bundle` variant",
                ));
            }
            let name = value_name.unwrap_or_else(|| ident.to_string().to_uppercase());
            if positional {
                help_items.push(occurrence_item(
                    variant,
                    None,
                    &[],
                    Some(&name),
                    Some(ty),
                    false,
                    false,
                ));
                if spec_on {
                    spec_checks.extend(spec_check(&input.generics, Some(ty), stated.as_ref()));
                    spec_rows.push(occurrence_spec(
                        variant,
                        spec::ItemDoc {
                            value_name: Some(name.clone()),
                            choices_ty: choices_name(stated.as_ref(), ty),
                            ..spec::ItemDoc::default()
                        },
                    ));
                }
            }
            let value = wrap(
                quote!(__word.convert::<#ty>(#name)?),
                quote!(__word.offset),
                quote!(false),
            );
            *slot = Some(quote!({
                let __item = Self::#ident(#value);
                // Read here, so a value only printed is not dead code.
                match &__item {
                    Self::#ident(__read) => {
                        let _ = __read;
                    }
                    #[allow(unreachable_patterns)]
                    _ => {}
                }
                ::core::result::Result::Ok(::core::option::Option::Some(__item))
            }));
            continue;
        }
        if short.is_none() && longs.is_empty() {
            longs.push(kebab_case(&ident.to_string()));
        }
        let spelled = short
            .map(|c| format!("-{c}"))
            .into_iter()
            .chain(longs.iter().map(|l| format!("--{l}")));
        for name in spelled {
            if let Some(other) = seen.insert(name.clone(), ident.clone()) {
                return Err(syn::Error::new(
                    variant.span(),
                    format!("`{name}` is already used by `{other}`"),
                ));
            }
        }
        let value = takes.map(|ty| spanned.or(optional).unwrap_or(ty));
        help_items.push(occurrence_item(
            variant,
            short,
            &longs,
            takes
                .is_some()
                .then(|| value_name.as_deref().unwrap_or("VALUE")),
            value,
            require_equals,
            default_missing.is_some(),
        ));
        if spec_on {
            let value_ty = takes.map(|ty| spanned.or(optional).unwrap_or(ty));
            spec_checks.extend(spec_check(&input.generics, value_ty, stated.as_ref()));
            spec_rows.push(occurrence_spec(
                variant,
                spec::ItemDoc {
                    short,
                    long: longs.first().cloned(),
                    aliases: longs.iter().skip(1).cloned().collect(),
                    value_name: value_ty
                        .map(|_| value_name.clone().unwrap_or_else(|| "VALUE".to_owned())),
                    choices_ty: value_ty.and_then(|ty| choices_name(stated.as_ref(), ty)),
                    require_equals,
                    optional_value: default_missing.is_some(),
                    two_dashes,
                    prefix,
                    ..spec::ItemDoc::default()
                },
            ));
        }
        let body = match takes {
            None => quote! {
                __arg.check_switch()?;
                ::core::result::Result::Ok(::core::option::Option::Some(Self::#ident))
            },
            Some(ty) => {
                let ty = spanned.or(optional).unwrap_or(ty);
                let options = quote! {
                    ::winnow_args::token::ValueOptions {
                        negative_numbers: #negative_numbers,
                        hyphen_values: #hyphen_values,
                        require_equals: #require_equals,
                        keep_equals: #keep_equals,
                    }
                };
                let attached = spanned
                    .is_some()
                    .then(|| quote!(let __attached = __arg.has_attached_value(__input);));
                let value = wrap(
                    quote!(__arg.convert::<#ty>(__value)?),
                    quote!(__arg.offset()),
                    quote!(__attached),
                );
                let item = if optional.is_some() && require_equals && default_missing.is_none() {
                    quote! {
                        if let ::core::option::Option::Some(__value) =
                            __arg.read_value_opt_with(__input, #options)
                        {
                            Self::#ident(::core::option::Option::Some(#value))
                        } else {
                            Self::#ident(::core::option::Option::None)
                        }
                    }
                } else {
                    let read = match &default_missing {
                        Some(missing) => {
                            let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                            quote! {
                                __arg.read_value_or_with(
                                    __input,
                                    #options,
                                    ::winnow_args::__private::BStr::new(#bytes),
                                )
                            }
                        }
                        None => quote!(__arg.read_value_with(__input, #options)?),
                    };
                    quote! {{
                        let __value = #read;
                        Self::#ident(#value)
                    }}
                };
                quote! {
                    #attached
                    let __item = #item;
                    // Read here, so a value only printed is not dead code.
                    match &__item {
                        Self::#ident(__read) => {
                            let _ = __read;
                        }
                        #[allow(unreachable_patterns)]
                        _ => {}
                    }
                    ::core::result::Result::Ok(::core::option::Option::Some(__item))
                }
            }
        };
        if !longs.is_empty() {
            let pattern = byte_patterns(&longs);
            long_arms.push(quote!(#pattern => { #body }));
            if !two_dashes {
                single_dash.extend(longs.iter().cloned());
            }
        }
        if let Some(c) = short {
            if prefix {
                if !c.is_ascii() || takes.is_none() {
                    return Err(syn::Error::new(
                        variant.span(),
                        "`prefix` is for a value-taking variant with an ASCII `short` letter",
                    ));
                }
                prefixes.push(c as u8);
            }
            let letter = LitChar::new(c, Span::call_site());
            let takes_value = takes.is_some();
            letters.push(quote!(#letter => ::core::option::Option::Some(#takes_value),));
            short_arms.push(quote!(#letter => { #body }));
        }
    }
    let has_positional = word.is_some();
    let word =
        word.unwrap_or_else(|| quote!(::core::result::Result::Ok(::core::option::Option::None)));
    let bundle = bundle.map(|body| {
        quote! {
            const BUNDLES: bool = true;

            fn from_bundle(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                #body
            }
        }
    });
    let unknown = unknown.map(|body| {
        quote! {
            fn from_unknown(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                #body
            }
        }
    });
    let is_long = matches_name(&single_dash);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if spec_on {
        spec::write_occurrence(input.ident.span(), &filed, &spec_rows)?;
    }
    Ok(quote! {
        #(#spec_checks)*
        impl #impl_generics ::winnow_args::Occurrence for #name #ty_generics #where_clause {
            const SPEC: &'static str = #identity;

            const PREFIXES: &'static [u8] = &[#(#prefixes),*];

            const POSITIONAL: bool = #has_positional;

            const ITEMS: &'static [::winnow_args::help::Item] = &[#(#help_items),*];

            fn from_arg<'__i>(
                __arg: &::winnow_args::Arg<'__i>,
                __input: &mut ::winnow_args::Argv<'__i>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                let _ = &__input;
                match __arg {
                    ::winnow_args::Arg::Long(__flag) => match __flag.name {
                        #(#long_arms)*
                        _ => ::core::result::Result::Ok(::core::option::Option::None),
                    },
                    ::winnow_args::Arg::Short(__flag) if !__flag.plus => match __flag.letter {
                        #(#short_arms)*
                        _ => ::core::result::Result::Ok(::core::option::Option::None),
                    },
                    _ => ::core::result::Result::Ok(::core::option::Option::None),
                }
            }

            fn from_word(
                __word: &::winnow_args::token::Word<'_>,
            ) -> ::core::result::Result<::core::option::Option<Self>, ::winnow_args::Error> {
                let _ = __word;
                #word
            }

            fn is_long(__name: &[u8]) -> bool {
                #is_long
            }

            #unknown

            #bundle

            fn short(__letter: char) -> ::core::option::Option<bool> {
                match __letter {
                    #(#letters)*
                    _ => ::core::option::Option::None,
                }
            }
        }
    })
}

/// An `Occurrence` variant's row in help: a flag by `short` and `longs`, or
/// the positional when it has neither.
fn occurrence_item(
    variant: &syn::Variant,
    short: Option<char>,
    longs: &[String],
    value_name: Option<&str>,
    value: Option<&Type>,
    require_equals: bool,
    optional_value: bool,
) -> TokenStream2 {
    let (doc, long_doc) = docs(&variant.attrs);
    let (help, long_help) = (text(&doc), text(&long_doc));
    // What the value's type says of it, when the variant holds one.
    let (choices, hint) = match value {
        Some(ty) => (
            quote!(<#ty as ::winnow_args::FromArg>::CHOICES),
            quote!(<#ty as ::winnow_args::FromArg>::HINT),
        ),
        None => (quote!(&[]), quote!(::winnow_args::complete::Hint::Nothing)),
    };
    let positional = short.is_none() && longs.is_empty();
    let short = opt_char(short);
    let long = opt_str(longs.first().map(String::as_str));
    let aliases = longs.iter().skip(1);
    let value_name = opt_str(value_name);
    quote!(::winnow_args::help::Item {
        short: #short,
        long: #long,
        aliases: &[#(#aliases),*],
        negate: ::core::option::Option::None,
        value_name: #value_name,
        help: #help,
        long_help: #long_help,
        heading: ::core::option::Option::None,
        hide: false,
        positional: #positional,
        required: false,
        multiple: #positional,
        trailing: false,
        default: ::core::option::Option::None,
        env: ::core::option::Option::None,
        choices: #choices,
        require_equals: #require_equals,
        global: false,
        more_shorts: &[],
        plus: ::core::option::Option::None,
        optional_value: #optional_value,
        values: 1,
        more_values: false,
        hint: #hint,
        complete: ::core::option::Option::None,
        warning: "",
    })
}

/// `spec = "Name"` among the options of a type, a field or a variant: the
/// name the documentation files the type under, or finds it under.
fn spec_name(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    let mut name = None;
    for attr in attrs.iter().filter(|a| is_ours(a)) {
        // The parser of each derive has already rejected what it does not know.
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("spec") {
                name = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.input.peek(syn::Token![=]) {
                meta.value()?.parse::<syn::Expr>()?;
            } else if meta.input.peek(syn::token::Paren) {
                let _content;
                syn::parenthesized!(_content in meta.input);
                _content.parse::<TokenStream2>()?;
            }
            Ok(())
        });
    }
    let Some(name) = name else {
        return Ok(None);
    };
    let text = name.value();
    if text.is_empty() || text.contains(['/', '\\', ':', '.']) {
        return Err(syn::Error::new(
            name.span(),
            "`spec` is a name as a type's is: a file is named after it",
        ));
    }
    Ok(Some(text))
}

/// Reads past `spec = "Name"`, which [`spec_name`] reads.
fn skip_spec(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if !meta.path.is_ident("spec") {
        return Ok(false);
    }
    meta.value()?.parse::<LitStr>()?;
    Ok(true)
}

/// The name a derived type is filed under: its `spec`, or its own.
fn filed(input: &DeriveInput) -> syn::Result<String> {
    Ok(spec_name(&input.attrs)?.unwrap_or_else(|| input.ident.unraw().to_string()))
}

/// A type as it is written, `add::Opts`: its modules tell two types of one
/// name apart. `crate`, `self` and `super` are no module's name. `self` and
/// `super` are written as the modules they are, from `crate`; where those
/// are not known, as `crate` alone, which says the type is this crate's.
///
/// A bare name is the path its file's `use` brings it in by, when one does.
/// One the file neither declares nor brings in by name is `?::Opts`: it may
/// be any type of that name, or none the documentation knows.
fn type_path(ty: &Type) -> Option<String> {
    let Type::Path(path) = boxed(ty).unwrap_or(ty) else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let mut names: Vec<String> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.unraw().to_string())
        .collect();
    if let [name] = &names[..] {
        match spec::origin(ty.span(), name) {
            spec::Origin::Here => {}
            spec::Origin::Imported(from) => names = from,
            spec::Origin::Unknown => names.insert(0, "?".to_owned()),
        }
    }
    let rooted = names
        .iter()
        .take_while(|name| matches!(name.as_str(), "crate" | "self" | "super"))
        .count();
    if rooted > 0 {
        let supers = names[..rooted]
            .iter()
            .filter(|name| *name == "super")
            .count();
        let relative = names[0] != "crate";
        let modules = relative.then(|| spec::above(supers)).flatten();
        let root = ["crate".to_owned()].into_iter();
        names.splice(..rooted, root.chain(modules.unwrap_or_default()));
    }
    (names.len() > usize::from(rooted > 0)).then(|| names.join("::"))
}

/// The name `ty` is looked for under: the one stated where it is referred
/// to, or its own.
fn spec_type(stated: Option<String>, ty: &Type) -> syn::Result<String> {
    stated.or_else(|| type_path(ty)).ok_or_else(|| {
        syn::Error::new(
            ty.span(),
            "this type needs a name when `WINNOW_ARGS_SPEC` is set: state it, as `spec = \"Name\"`",
        )
    })
}

/// A documentation build fails where a type is looked for under a name it is
/// not filed under, so that a page never describes another type of that name.
/// A generic type is not checked: a constant cannot name its parameters.
fn spec_filed(
    generics: &syn::Generics,
    kind: TokenStream2,
    ty: &Type,
    name: &str,
) -> Option<TokenStream2> {
    if !generics.params.is_empty() {
        return None;
    }
    let ty = boxed(ty).unwrap_or(ty);
    let name = name.strip_prefix("?::").unwrap_or(name);
    Some(quote::quote_spanned! {ty.span()=>
        const _: () = ::core::assert!(
            ::winnow_args::__private::filed_as(<#ty as ::winnow_args::#kind>::SPEC, #name),
            "the documentation does not find this type by this path: it is named through an alias or a re-export, its module goes by another name here, or it states a `spec`. State the name it is filed under, as `spec = \"Name\"`",
        );
    })
}

/// The type of a field's value.
fn value_type(field: &Field) -> Option<&Type> {
    if !field.takes_value()
        || field.keywords
        || field.from_str
        || matches!(field.role, Role::Subcommand)
    {
        return None;
    }
    match &field.kind {
        Kind::Optional(ty) | Kind::Required(ty) | Kind::Many(ty) => Some(ty),
        Kind::Switch | Kind::Count(_) => None,
    }
}

/// The type of a field's value, where its choices are looked for: none when
/// the field states them.
fn choices_type(field: &Field) -> Option<&Type> {
    value_type(field).filter(|_| field.choices.is_none())
}

/// A documentation build fails where the choices a field states are not
/// those of its type, so that a list written twice cannot drift.
fn spec_check_stated(generics: &syn::Generics, field: &Field) -> Option<TokenStream2> {
    let stated = field.choices.as_ref()?;
    let ty = value_type(field).filter(|_| generics.params.is_empty())?;
    Some(quote::quote_spanned! {ty.span()=>
        const _: () = ::core::assert!(
            ::winnow_args::__private::stated_choices::<#ty>(&[#(#stated),*]),
            "the choices stated on this field are not those of its type, in number, spelling or order",
        );
    })
}

/// The name the value type's choices are looked for under: the field's
/// `spec`, or the type's own. Only a `ValueEnum` has a fragment under it.
fn choices_link(field: &Field) -> Option<String> {
    choices_name(field.spec.as_ref(), choices_type(field)?)
}

fn choices_name(stated: Option<&String>, ty: &Type) -> Option<String> {
    stated.cloned().or_else(|| type_path(ty))
}

/// A documentation build fails where a value has choices its fragment cannot
/// name, so that a page never leaves them out. A generic type is not checked:
/// a constant cannot name its parameters.
fn spec_check(
    generics: &syn::Generics,
    ty: Option<&Type>,
    stated: Option<&String>,
) -> Option<TokenStream2> {
    let ty = ty.filter(|_| generics.params.is_empty())?;
    let name = choices_name(stated, ty)?;
    let name = name.strip_prefix("?::").unwrap_or(&name);
    Some(quote::quote_spanned! {ty.span()=>
        const _: () = ::core::assert!(
            ::winnow_args::__private::documented_choices::<#ty>(#name),
            "the documentation cannot find the choices of this value: its type is named through an alias or a re-export, states a `spec`, or implements `FromArg` by hand. State the name they are filed under, as `spec = \"Name\"`, or the choices, as `choices(\"a\", \"b\")`",
        );
    })
}

/// A bare `version` becomes `env!("CARGO_PKG_VERSION")` in help. Cargo has
/// already set that variable for the crate being compiled.
fn package_version(version: &Option<TokenStream2>, enabled: bool) -> Option<String> {
    if !enabled {
        return None;
    }
    let tokens = version.as_ref()?;
    if let Ok(lit) = syn::parse2::<LitStr>(tokens.clone()) {
        let text = lit.value();
        return (!text.is_empty()).then_some(text);
    }
    std::env::var("CARGO_PKG_VERSION")
        .ok()
        .filter(|text| !text.is_empty())
}

fn spec_item(field: &Field) -> syn::Result<spec::ItemDoc> {
    let longs = field.longs();
    let keywords = if field.keywords {
        match &field.kind {
            Kind::Required(ty) => Some(spec_type(field.spec.clone(), ty)?),
            _ => {
                return Err(syn::Error::new(
                    field.ident.span(),
                    "`keywords` needs a named type",
                ));
            }
        }
    } else {
        None
    };
    let required = (matches!(field.kind, Kind::Required(_))
        && field.default.is_none()
        && field.default_fn.is_none()
        && field.env.is_none()
        && field.default_from.is_none()
        && !field.keywords)
        || field.required;
    Ok(spec::ItemDoc {
        short: field.short(),
        more_shorts: field.short_aliases.clone(),
        plus: field.plus,
        long: longs.first().map(|name| (*name).to_owned()),
        aliases: longs
            .iter()
            .skip(1)
            .map(|name| (*name).to_owned())
            .collect(),
        negate: field.negate.clone(),
        value_name: if field.is_positional() {
            Some(field.display())
        } else {
            field.value_name.clone()
        },
        help: field.help.clone(),
        long_help: field.long_help.clone(),
        heading: field.heading.clone(),
        hide: field.hide,
        positional: field.is_positional(),
        required,
        multiple: matches!(field.kind, Kind::Many(_) | Kind::Count(_)),
        trailing: field.double_dash() == DoubleDash::Required,
        default: field.shown_default(),
        env: field.shown_env().map(str::to_owned),
        choices: field.choices.clone().unwrap_or_default(),
        choices_ty: choices_link(field),
        require_equals: field.require_equals,
        global: field.is_global(),
        optional_value: field.default_missing.is_some()
            || field.require_equals && matches!(field.kind, Kind::Optional(_)),
        values: field.values,
        more_values: field.values_max.is_some(),
        two_dashes: field.two_dashes,
        prefix: field.prefix,
        stop_flags: field.stop_flags,
        keywords,
        warning: field.warning.clone(),
        effect: field.effect.clone(),
    })
}

fn occurrence_spec(variant: &syn::Variant, mut item: spec::ItemDoc) -> spec::ItemDoc {
    let (help, long_help) = docs(&variant.attrs);
    item.help = help;
    item.long_help = long_help;
    item.positional = item.short.is_none() && item.long.is_none();
    item.multiple = item.positional;
    item
}

/// `FromArg` as one `match` on the value's bytes.
fn expand_value_enum(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`ValueEnum` can only be derived for enums",
        ));
    };
    let case = rename_all(input, None)?;
    let mut names: Vec<(String, &Ident)> = Vec::new();
    let mut arms = Vec::new();
    let mut visible = Vec::new();
    let spec_on = spec::enabled();
    let filed = filed(input)?;
    let identity = spec::identity(input.ident.span(), &filed);
    let mut spec_choices = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "a `ValueEnum` variant holds no data",
            ));
        }
        let Variant {
            names: spellings,
            hide,
            ..
        } = variant_names(variant, &mut names, case)?;
        let name = LitStr::new(&spellings[0], Span::call_site());
        if !hide {
            if spec_on {
                spec_choices.push(spellings[0].clone());
            }
            visible.push(name);
        }
        let pattern = byte_patterns(&spellings);
        let ident = &variant.ident;
        arms.push(quote!(#pattern => ::core::result::Result::Ok(Self::#ident),));
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if spec_on {
        spec::write_choices(input.ident.span(), &filed, &spec_choices)?;
    }
    Ok(quote! {
        impl #impl_generics ::winnow_args::FromArg for #name #ty_generics #where_clause {
            const CHOICES: &'static [&'static str] = &[#(#visible),*];
            const SPEC: &'static str = #identity;

            fn from_arg(
                __value: &::winnow_args::__private::BStr,
            ) -> ::core::result::Result<Self, ::winnow_args::__private::BoxError> {
                match &**__value {
                    #(#arms)*
                    _ => ::core::result::Result::Err(::std::boxed::Box::new(
                        ::winnow_args::ChoiceError { choices: Self::CHOICES },
                    )),
                }
            }
        }
    })
}

/// Help prose, left out of binaries built without winnow-args' `help-text`.
fn text(prose: &str) -> TokenStream2 {
    if prose.is_empty() {
        quote!("")
    } else {
        quote!(::winnow_args::__text!(#prose))
    }
}

/// A variant's spellings and help.
struct Variant {
    /// `name` (or kebab-case) first, then shown aliases, then hidden ones.
    names: Vec<String>,
    /// How many of `names` after the first are shown in help.
    shown: usize,
    hide: bool,
    about: String,
}

/// How a variant's name is spelled when it gives none: an enum's
/// `#[arg(rename_all = "…")]`.
#[derive(Clone, Copy)]
enum Case {
    /// `DryRun` → `dry-run` (the default).
    Kebab,
    /// `DryRun` → `dryrun`.
    Lower,
    /// `DryRun` → `DRYRUN`.
    Upper,
    /// `DryRun` → `dry_run`.
    Snake,
    /// `DryRun` → `DryRun`.
    Verbatim,
}

impl Case {
    fn apply(self, ident: &str) -> String {
        let ident = ident.trim_start_matches("r#");
        match self {
            Case::Kebab => kebab_case(ident),
            Case::Lower => ident.to_lowercase(),
            Case::Upper => ident.to_uppercase(),
            Case::Snake => kebab_case(ident).replace('-', "_"),
            Case::Verbatim => ident.to_owned(),
        }
    }
}

/// How a `Subcommand` enum runs the command it holds: `#[arg(run)]` or
/// `#[arg(run_async)]`.
#[derive(Clone, Copy, PartialEq)]
enum Runs {
    Sync,
    Async,
}

/// An enum's `#[arg(rename_all = "…")]`, kebab-case if absent, and for a
/// `Subcommand` its `run` or `run_async`.
fn rename_all(input: &DeriveInput, runs: Option<&mut Option<Runs>>) -> syn::Result<Case> {
    let mut case = Case::Kebab;
    let mut found = None;
    let may_run = runs.is_some();
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if skip_spec(&meta)? || (may_run && meta.path.is_ident("verbatim_doc_comment")) {
                return Ok(());
            }
            let run = match () {
                () if meta.path.is_ident("run") => Some(Runs::Sync),
                () if meta.path.is_ident("run_async") => Some(Runs::Async),
                () => None,
            };
            if let (Some(run), true) = (run, may_run) {
                if found.replace(run).is_some() {
                    return Err(meta.error("`run` or `run_async`, once"));
                }
                return Ok(());
            }
            if !meta.path.is_ident("rename_all") {
                return Err(meta.error(match may_run {
                    true => "expected `rename_all`, `run`, `run_async` or `spec`",
                    false => "expected `rename_all` or `spec`",
                }));
            }
            let lit = meta.value()?.parse::<LitStr>()?;
            case = match lit.value().as_str() {
                "kebab-case" => Case::Kebab,
                "lowercase" => Case::Lower,
                "UPPERCASE" => Case::Upper,
                "snake_case" => Case::Snake,
                "verbatim" => Case::Verbatim,
                _ => {
                    return Err(syn::Error::new(
                        lit.span(),
                        "expected \"kebab-case\", \"lowercase\", \"UPPERCASE\", \"snake_case\" or \"verbatim\"",
                    ));
                }
            };
            Ok(())
        })?;
    }
    if let Some(runs) = runs {
        *runs = found;
    }
    Ok(case)
}

/// A variant's spellings, checked against `seen` for duplicates, and its help.
fn variant_names<'a>(
    variant: &'a syn::Variant,
    seen: &mut Vec<(String, &'a Ident)>,
    case: Case,
) -> syn::Result<Variant> {
    let holds_type = !matches!(variant.fields, Fields::Unit);
    let (mut name, mut alias, mut hidden, mut hide) = (None, Vec::new(), Vec::new(), false);
    let mut help: Option<String> = None;
    for attr in variant.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("alias") {
                alias.extend(aliases(&meta)?);
            } else if meta.path.is_ident("alias_hidden") {
                hidden.extend(aliases(&meta)?);
            } else if meta.path.is_ident("hide") {
                hide = true;
            } else if meta.path.is_ident("help") {
                help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("long_help") {
                // The listing shows one line; the subcommand's own help has the rest.
                meta.value()?.parse::<LitStr>()?;
            } else if meta.path.is_ident("verbatim_doc_comment") && !holds_type {
            } else if skip_spec(&meta)? {
                if !holds_type {
                    return Err(meta
                        .error("`spec` names the type a variant holds, and this one holds none"));
                }
            } else if matches!(variant.fields, Fields::Named(_)) {
                // An option of the command its fields make, checked as a
                // struct's are.
                if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<syn::Expr>()?;
                } else if meta.input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    content.parse::<TokenStream2>()?;
                }
            } else {
                return Err(meta.error(
                    "expected `name`, `alias`, `alias_hidden`, `hide`, `help` or `long_help`; \
                     a command's options, such as `validate`, go on a variant with fields or on the struct it holds",
                ));
            }
            Ok(())
        })?;
    }
    let ident = &variant.ident;
    let shown = alias.len();
    let names: Vec<String> =
        std::iter::once(name.unwrap_or_else(|| case.apply(&ident.to_string())))
            .chain(alias)
            .chain(hidden)
            .collect();
    for spelling in &names {
        if let Some((_, other)) = seen.iter().find(|(n, _)| n == spelling) {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{spelling}` is already used by `{other}`"),
            ));
        }
        seen.push((spelling.clone(), ident));
    }
    Ok(Variant {
        names,
        shown,
        hide,
        about: help
            .map(|h| {
                h.split("\n\n")
                    .next()
                    .unwrap_or_default()
                    .replace('\n', " ")
            })
            .unwrap_or_else(|| docs(&variant.attrs).0),
    })
}

/// A doc comment as help: its first paragraph, and the whole of it. Each
/// paragraph is one line, so help wraps it to the terminal rather than to the
/// source; an indented line is kept as written, and every line is with
/// `verbatim_doc_comment`.
fn docs(attrs: &[syn::Attribute]) -> (String, String) {
    let verbatim = has_option(attrs, "verbatim_doc_comment");
    let lines = attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        })
        .map(|line| {
            line.strip_prefix(' ')
                .unwrap_or(&line)
                .trim_end()
                .to_owned()
        });
    let mut long = String::new();
    let mut joins = false;
    for line in lines {
        let kept = verbatim || line.starts_with([' ', '\t']);
        if joins && !kept && !line.is_empty() {
            long.push(' ');
        } else if !long.is_empty() {
            long.push('\n');
        }
        long.push_str(&line);
        joins = !kept && !line.is_empty();
    }
    let long = long.trim().to_owned();
    let short = long
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .replace('\n', " ");
    (short, long)
}

/// `error`, said with `message` in place of its default one when there is one.
fn worded(error: TokenStream2, message: Option<&str>) -> TokenStream2 {
    match message {
        Some(message) => quote!(__wa::worded(#error, #message)),
        None => error,
    }
}

/// `Some("s")` or `None`, spelled out.
fn opt_str(s: Option<&str>) -> TokenStream2 {
    match s {
        Some(s) => quote!(::core::option::Option::Some(#s)),
        None => quote!(::core::option::Option::None),
    }
}

/// `Some('c')` or `None`, spelled out.
fn opt_char(c: Option<char>) -> TokenStream2 {
    match c {
        Some(c) => quote!(::core::option::Option::Some(#c)),
        None => quote!(::core::option::Option::None),
    }
}

/// `b"a" | b"b"`.
fn byte_patterns<S: AsRef<str>>(names: &[S]) -> TokenStream2 {
    let literals = names
        .iter()
        .map(|n| LitByteStr::new(n.as_ref().as_bytes(), Span::call_site()));
    quote!(#(#literals)|*)
}

/// Whether `__name` is one of `names`: `matches!(__name, b"a" | b"b")`.
fn matches_name(names: &[String]) -> TokenStream2 {
    if names.is_empty() {
        quote!(false)
    } else {
        let names = byte_patterns(names);
        quote!(matches!(__name, #names))
    }
}

/// `alias = "x"` or `alias("x", "y")`.
fn aliases(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Vec<String>> {
    if meta.input.peek(syn::Token![=]) {
        return Ok(vec![meta.value()?.parse::<LitStr>()?.value()]);
    }
    let content;
    syn::parenthesized!(content in meta.input);
    let names = syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
    Ok(names.iter().map(LitStr::value).collect())
}

/// `key("a", "b", message = "…")` or `key = "a"`: each selector, with the
/// message said in place of the default one, `{0}` this field and `{1}` the other.
fn related(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Vec<(String, Option<String>)>> {
    if meta.input.peek(syn::Token![=]) {
        return Ok(vec![(meta.value()?.parse::<LitStr>()?.value(), None)]);
    }
    let content;
    syn::parenthesized!(content in meta.input);
    let (mut names, mut message) = (Vec::new(), None);
    while !content.is_empty() {
        if content.peek(LitStr) {
            names.push(content.parse::<LitStr>()?.value());
        } else {
            let key = content.parse::<Ident>()?;
            if key != "message" {
                return Err(syn::Error::new(
                    key.span(),
                    "expected a selector or `message`",
                ));
            }
            content.parse::<syn::Token![=]>()?;
            message = Some(content.parse::<LitStr>()?.value());
        }
        if !content.is_empty() {
            content.parse::<syn::Token![,]>()?;
        }
    }
    Ok(names
        .into_iter()
        .map(|name| (name, message.clone()))
        .collect())
}

/// `DryRun` → `dry-run`.
fn kebab_case(ident: &str) -> String {
    let mut out = String::new();
    for (i, c) in ident.trim_start_matches("r#").chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// An attribute for these derives: `#[arg(…)]`, or `#[winnow_args(…)]` where
/// another derive claims `arg`.
fn is_ours(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("arg") || attr.path().is_ident("winnow_args")
}

/// Whether `ty` is `bool`.
fn is_bool(ty: &Type) -> bool {
    last_segment(ty).is_some_and(|s| s.ident == "bool" && s.arguments.is_none())
}

/// Whether a field is `#[arg(sequence)]`: occurrences of an `Occurrence` enum.
fn is_sequence(f: &syn::Field) -> bool {
    has_flag(f, "sequence")
}

/// Whether a field is `#[arg(flatten)]`: another struct's flags, parsed as ours.
fn is_flatten(f: &syn::Field) -> bool {
    has_flag(f, "flatten")
}

/// Whether a field is `#[arg(unknown)]`: the flags no one declared, whole.
fn is_unknown(f: &syn::Field) -> bool {
    has_flag(f, "unknown") && !is_sequence(f)
}

/// Whether a field is `#[arg(skip)]`: left at its default, not parsed.
fn is_skipped(f: &syn::Field) -> bool {
    has_flag(f, "skip")
}

/// Whether one of the field's attributes is the bare word `name`.
fn has_flag(f: &syn::Field, name: &str) -> bool {
    has_option(&f.attrs, name)
}

/// Whether one of `attrs` is `#[arg(name)]`, among other options.
fn has_option(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().filter(|a| is_ours(a)).any(|a| {
        let mut found = false;
        let _ = a.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) {
                found = true;
            } else if meta.input.peek(syn::Token![=]) {
                meta.value()?.parse::<syn::Expr>()?;
            } else if meta.input.peek(syn::token::Paren) {
                let _content;
                syn::parenthesized!(_content in meta.input);
            }
            Ok(())
        });
        found
    })
}

/// A `skip`, `flatten`, `sequence` or `unknown` field takes no other option
/// (`sequence, unknown` aside): none of them would be read.
fn role_alone(f: &syn::Field) -> syn::Result<()> {
    let role = ["skip", "flatten", "sequence", "unknown"]
        .into_iter()
        .find(|role| has_flag(f, role));
    let Some(role) = role else {
        return Ok(());
    };
    for attr in f.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            let alone =
                meta.path.is_ident(role) || (role == "sequence" && meta.path.is_ident("unknown"));
            // The name the documentation finds a flattened struct or a
            // sequence enum under.
            let named = matches!(role, "flatten" | "sequence") && skip_spec(&meta)?;
            if alone || named {
                Ok(())
            } else {
                Err(meta.error(format!("a `{role}` field takes no other `arg` options")))
            }
        })?;
    }
    Ok(())
}

/// `T` in `Box<T>`.
fn boxed(ty: &Type) -> Option<&Type> {
    let last = last_segment(ty)?;
    if last.ident == "Box" {
        inner(last)
    } else {
        None
    }
}

enum Kind {
    /// `bool`: present or not.
    Switch,
    /// `#[arg(count)]` on an integer: how many times it was given.
    Count(Type),
    /// `Option<T>`: one value, may be absent.
    Optional(Type),
    /// `T`: one value, required.
    Required(Type),
    /// `Vec<T>`: a repeatable flag's values, or every remaining word.
    Many(Type),
}

enum Role {
    Flag {
        short: Option<char>,
        long: Option<String>,
        /// More long names, matched like `long`.
        aliases: Vec<String>,
        /// Also accepted after a subcommand word, at any depth.
        global: bool,
    },
    Positional {
        name: String,
        double_dash: DoubleDash,
    },
    Subcommand,
}

/// A positional's relation to `--`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DoubleDash {
    /// Words fill it on either side of `--`.
    Optional,
    /// Only words after `--` fill it, and they all do.
    Required,
    /// Once it has a value, flags stop as if `--` had been typed.
    Automatic,
    /// A `--` reaching it is one of its values, and stops nothing.
    Preserve,
}

struct Field {
    ident: Ident,
    /// The name the documentation finds the field's type under, when it is
    /// not the type's own.
    spec: Option<String>,
    kind: Kind,
    role: Role,
    /// Splits each value of a `Vec` field.
    delimiter: Option<u8>,
    /// The only values accepted, checked before conversion.
    choices: Option<Vec<String>>,
    /// Environment variable consulted when the command line gave nothing.
    env: Option<String>,
    /// Value used when neither the command line nor the environment did.
    default: Option<String>,
    /// Selectors that may not be supplied together with this field, each with
    /// the message said in place of the default one.
    conflicts: Vec<(String, Option<String>)>,
    /// Selectors this field replaces, and that replace it: the last one given wins.
    overrides: Vec<String>,
    /// Selectors that must have a value when this field is supplied, each with
    /// the message said in place of the default one.
    requires: Vec<(String, Option<String>)>,
    /// Must end up with a value (for `Option` and `Vec` fields).
    required: bool,
    /// Required unless one of these selectors has a value.
    required_unless: Vec<String>,
    /// Required unless every one of these has a value.
    required_unless_all: Vec<String>,
    /// Required when a field was given, or given this value.
    required_ifs: Vec<(String, Option<String>)>,
    /// When this field has the value, the field named needs one.
    requires_ifs: Vec<(String, String)>,
    /// Given only alone.
    exclusive: bool,
    /// The struct-level group this field belongs to.
    group: Option<String>,
    /// The value of a flag given without one.
    default_missing: Option<String>,
    /// Help: first paragraph, whole text, section, hidden.
    help: String,
    long_help: String,
    heading: Option<String>,
    hide: bool,
    /// A flag's value placeholder.
    value_name: Option<String>,
    /// What the value is, for a shell that completes it: a variant of `Hint`.
    /// Unstated, it is what the value's type says.
    hint: Option<syn::Ident>,
    /// The function a shell's completion of the value asks.
    complete: Option<syn::Path>,
    /// What to know before using it, shown by `--help`.
    warning: String,
    /// Help and the pages leave the default out.
    hide_default_value: bool,
    /// Help and the pages leave the environment variable out.
    hide_env: bool,
    /// Where help lists it among the fields; otherwise its declaration's place.
    display_order: Option<usize>,
    /// The value is parsed with its type's `FromStr`.
    from_str: bool,
    /// What giving it does, for the documentation: `read`, `write` or
    /// `destructive`.
    effect: Option<String>,
    /// A negative number is a value: a flag's detached value, or a positional's word.
    negative_numbers: bool,
    /// A flag's detached value may be any word, flag-like or `--`.
    hyphen_values: bool,
    /// A flag's value must be attached.
    require_equals: bool,
    /// A short option's attached value keeps a leading `=` (GNU ld's `-L=dir`).
    keep_equals: bool,
    /// The long spelling that sets a `bool` false; the slot is then an
    /// `Option<bool>` until the end, so a default fills only what was not given.
    negate: Option<String>,
    /// Never spelled with one dash under `long_only`: `--omagic` (ld).
    two_dashes: bool,
    /// Words each occurrence takes (a `Vec` flag): `-platform_version macos 11.0 12.0`.
    values: usize,
    /// `values = a..` or `a..=b`: up to this many words in all (`usize::MAX`:
    /// no limit), the ones after the first only while they are not flag-like.
    values_max: Option<usize>,
    /// The word that ends a variable run of values, itself dropped.
    value_terminator: Option<String>,
    /// A function giving the value when nothing else did.
    default_fn: Option<syn::ExprPath>,
    /// The fields a computed default is made from, and the function or
    /// closure that makes it from them.
    default_from: Option<(Vec<Ident>, syn::Expr)>,
    /// What help says of `default_fn`'s value.
    default_note: Option<String>,
    /// `(selector, the value it must have, this field's default then)`.
    default_ifs: Vec<(String, Option<String>, String)>,
    /// Once this positional has a value, flags stop (whatever its `double_dash`).
    stop_flags: bool,
    /// The letter of its `+c` spelling, in a `plus_options` struct.
    plus: Option<char>,
    /// An `Option<bool>` with `short` and the same `plus` letter: `-c` is
    /// `Some(true)`, `+c` is `Some(false)`.
    tristate: bool,
    /// Its values are words of a vocabulary of their own, the field's type
    /// (`Args`): `-z now -z max-page-size=4096` (ld).
    keywords: bool,
    /// More letters it answers to, from a second `short` (`kill -l`/`-L`).
    short_aliases: Vec<char>,
    /// A short letter that always takes the rest of its word (`-lfoo`), so
    /// under `long_only` no long name starting with it is tried with one dash.
    prefix: bool,
}

impl Field {
    /// This field's entry in `help::Command::items`.
    fn help_item(&self) -> TokenStream2 {
        let short = opt_char(self.short());
        let long = opt_str(self.longs().first().copied());
        let aliases = self.longs().into_iter().skip(1);
        let negate = opt_str(self.negate.as_deref());
        let require_equals = self.require_equals;
        let global = self.is_global();
        let positional = self.is_positional();
        let trailing = self.double_dash() == DoubleDash::Required;
        let display = self.display();
        let value_name = if positional {
            opt_str(Some(&display))
        } else {
            opt_str(self.value_name.as_deref())
        };
        let (help, long_help) = (text(&self.help), text(&self.long_help));
        let heading = opt_str(self.heading.as_deref());
        let warning = text(&self.warning);
        let hide = self.hide;
        let more_shorts = self.short_aliases.iter();
        let plus = opt_char(self.plus);
        let optional_value = self.default_missing.is_some()
            || require_equals && matches!(self.kind, Kind::Optional(_));
        let values = self.values;
        let more_values = self.values_max.is_some();
        let hint = match (&self.hint, &self.kind, &self.role) {
            (Some(hint), _, _) => quote!(::winnow_args::complete::Hint::#hint),
            (None, Kind::Optional(ty) | Kind::Required(ty) | Kind::Many(ty), role)
                if !matches!(role, Role::Subcommand)
                    && !self.keywords
                    && !self.tristate
                    && !self.from_str =>
            {
                quote!(<#ty as ::winnow_args::FromArg>::HINT)
            }
            (None, _, _) => quote!(::winnow_args::complete::Hint::Nothing),
        };
        let complete = match &self.complete {
            Some(function) => quote!(::core::option::Option::Some(#function)),
            None => quote!(::core::option::Option::None),
        };
        let required = matches!(self.kind, Kind::Required(_))
            && self.default.is_none()
            && self.default_fn.is_none()
            && self.env.is_none()
            && self.default_from.is_none()
            && !self.keywords
            || self.required;
        let multiple = matches!(self.kind, Kind::Many(_) | Kind::Count(_));
        let default = opt_str(self.shown_default().as_deref());
        let env = opt_str(self.shown_env());
        // Declared `choices`, else whatever fixed set the value type has.
        let choices = match (&self.choices, &self.kind, &self.role) {
            (Some(choices), _, _) => quote!(&[#(#choices),*]),
            (None, _, Role::Subcommand) | (None, Kind::Switch | Kind::Count(_), _) => quote!(&[]),
            (None, _, _) if self.keywords || self.tristate || self.from_str => quote!(&[]),
            (None, Kind::Optional(ty) | Kind::Required(ty) | Kind::Many(ty), _) => {
                quote!(<#ty as ::winnow_args::FromArg>::CHOICES)
            }
        };
        quote! {
            ::winnow_args::help::Item {
                short: #short,
                long: #long,
                aliases: &[#(#aliases),*],
                negate: #negate,
                value_name: #value_name,
                help: #help,
                long_help: #long_help,
                heading: #heading,
                hide: #hide,
                positional: #positional,
                required: #required,
                multiple: #multiple,
                trailing: #trailing,
                default: #default,
                env: #env,
                choices: #choices,
                require_equals: #require_equals,
                global: #global,
                more_shorts: &[#(#more_shorts),*],
                plus: #plus,
                optional_value: #optional_value,
                values: #values,
                more_values: #more_values,
                hint: #hint,
                complete: #complete,
                warning: #warning,
            }
        }
    }

    /// How this flag takes a detached value.
    fn value_options(&self) -> TokenStream2 {
        let (negative_numbers, hyphen_values, require_equals, keep_equals) = (
            self.negative_numbers,
            self.hyphen_values,
            self.require_equals,
            self.keep_equals,
        );
        quote! {
            __wa::ValueOptions {
                negative_numbers: #negative_numbers,
                hyphen_values: #hyphen_values,
                require_equals: #require_equals,
                keep_equals: #keep_equals,
            }
        }
    }

    /// Reading this flag's value: `read_value`, or `read_value_or` its `default_missing`.
    fn read(&self) -> TokenStream2 {
        let (negative_numbers, hyphen_values, require_equals, keep_equals) = (
            self.negative_numbers,
            self.hyphen_values,
            self.require_equals,
            self.keep_equals,
        );
        let options = self.value_options();
        match &self.default_missing {
            None if !negative_numbers && !hyphen_values && !require_equals && !keep_equals => {
                quote!(__arg.read_value(__input)?)
            }
            None => quote!(__arg.read_value_with(__input, #options)?),
            Some(missing) => {
                let bytes = LitByteStr::new(missing.as_bytes(), Span::call_site());
                quote!(__arg.read_value_or_with(__input, #options, __wa::BStr::new(#bytes)))
            }
        }
    }

    /// Reject `value` (a `&BStr`) outside `choices`, reported by `error(cause)`.
    fn check(
        &self,
        value: TokenStream2,
        error: impl FnOnce(TokenStream2) -> TokenStream2,
    ) -> TokenStream2 {
        let Some(choices) = &self.choices else {
            return quote!();
        };
        let pattern = byte_patterns(choices);
        let listed = choices.iter().map(|c| LitStr::new(c, Span::call_site()));
        let cause = quote!(::winnow_args::ChoiceError { choices: &[#(#listed),*] });
        let error = error(cause);
        quote! {
            if !matches!(&**#value, #pattern) {
                return ::core::result::Result::Err(#error);
            }
        }
    }

    /// The type a value is converted to, and what makes it the field's: with
    /// `from_str`, `Parsed<T>` and its `.0`.
    fn parsed(&self, ty: &Type) -> (TokenStream2, TokenStream2) {
        if self.from_str {
            (quote!(::winnow_args::value::Parsed<#ty>), quote!(.0))
        } else {
            (quote!(#ty), quote!())
        }
    }

    /// A flag's value converted to `ty`, checked against `choices` first.
    fn flag_value(&self, ty: &Type, value: TokenStream2) -> TokenStream2 {
        let check = self.check(value.clone(), |cause| {
            quote!(__wa::Error::invalid_value(__arg.offset(), __arg.spelling(), #value, #cause))
        });
        if self.from_str {
            return quote!({ #check __arg.convert::<::winnow_args::value::Parsed<#ty>>(#value)?.0 });
        }
        quote!({ #check __arg.convert::<#ty>(#value)? })
    }

    /// `value` (a `&BStr` from the environment or a default) converted to `ty`;
    /// errors name it by `source`.
    fn source_value(&self, ty: &Type, value: TokenStream2, source: &str) -> TokenStream2 {
        let source = LitStr::new(source, Span::call_site());
        let check = self.check(
            value.clone(),
            |cause| quote!(__wa::Error::invalid_value(__input.offset(), #source, #value, #cause)),
        );
        let (ty, unwrap) = self.parsed(ty);
        quote!({
            #check
            <#ty as __wa::FromArg>::from_arg(#value).map_err(|cause| {
                __wa::Error::invalid_value(__input.offset(), #source, #value, cause)
            })? #unwrap
        })
    }

    /// Whether the field holds a value. Read before defaults this is "supplied"
    /// (command line or environment); after them it is "has a value".
    fn has(&self) -> TokenStream2 {
        let slot = slot(&self.ident);
        if self.keywords {
            return quote!((!#slot.is_empty()));
        }
        match &self.kind {
            Kind::Switch if self.negate.is_some() => {
                quote!((#slot == ::core::option::Option::Some(true)))
            }
            Kind::Switch => quote!(#slot),
            Kind::Count(_) => quote!((#slot != 0)),
            Kind::Optional(_) | Kind::Required(_) => quote!(#slot.is_some()),
            Kind::Many(_) => quote!((!#slot.is_empty())),
        }
    }

    /// Store `value` (a `&BStr`), converted, in the slot of a field that takes
    /// values; `source` names where it came from in an error.
    fn assign(&self, value: TokenStream2, source: &str) -> TokenStream2 {
        let slot = slot(&self.ident);
        match &self.kind {
            Kind::Optional(ty) | Kind::Required(ty) => {
                let value = self.source_value(ty, value, source);
                quote!(#slot = ::core::option::Option::Some(#value);)
            }
            Kind::Many(ty) => match self.delimiter {
                None => {
                    let value = self.source_value(ty, value, source);
                    quote!(#slot.push(#value);)
                }
                Some(d) => {
                    let piece = self.source_value(ty, quote!(__piece), source);
                    quote!(for __piece in __wa::split(#value, #d) { #slot.push(#piece); })
                }
            },
            Kind::Switch | Kind::Count(_) => unreachable!("a switch or a count takes no value"),
        }
    }

    /// The default help and the pages show, unless `hide_default_value`.
    fn shown_default(&self) -> Option<String> {
        if self.hide_default_value {
            return None;
        }
        let stated = self.default.clone().or_else(|| self.default_note.clone());
        stated.or_else(|| self.computed_default())
    }

    /// The environment variable help and the pages show, unless `hide_env`.
    fn shown_env(&self) -> Option<&str> {
        self.env.as_deref().filter(|_| !self.hide_env)
    }

    /// The type of the value a `default_from` is given for this field.
    fn given_type(&self) -> TokenStream2 {
        match &self.kind {
            Kind::Switch => quote!(bool),
            Kind::Count(ty) | Kind::Required(ty) => quote!(#ty),
            Kind::Optional(ty) => quote!(::core::option::Option<#ty>),
            Kind::Many(ty) => quote!(::std::vec::Vec<#ty>),
        }
    }

    /// The type of the value a `default_from` makes for this field.
    fn made_type(&self) -> TokenStream2 {
        match &self.kind {
            Kind::Switch => quote!(bool),
            Kind::Count(ty) | Kind::Required(ty) | Kind::Optional(ty) => quote!(#ty),
            Kind::Many(ty) => quote!(::std::vec::Vec<#ty>),
        }
    }

    /// What help says of a `default_from` with no `default_note`: the fields
    /// it is made from.
    fn computed_default(&self) -> Option<String> {
        let (names, _) = self.default_from.as_ref()?;
        let names: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
        Some(format!("from {}", names.join(", ")))
    }

    /// Whether the field was not given, for a fallback to fill it.
    fn unset(&self, displaced: bool) -> TokenStream2 {
        let slot = slot(&self.ident);
        if self.negate.is_some() {
            return quote!(#slot.is_none());
        }
        let has = self.has();
        let not_displaced = displaced.then(|| {
            let displaced = displaced_flag(&self.ident);
            quote!(&& !#displaced)
        });
        quote!(!#has #not_displaced)
    }

    /// Fill the field from its environment variable (`env`) or its default when
    /// the command line left it unset and no `overrides` displaced it.
    fn fallback(&self, env: bool, displaced: bool) -> TokenStream2 {
        let slot = slot(&self.ident);
        if self.negate.is_some() {
            return self.negatable_fallback(env);
        }
        let unset = self.unset(displaced);
        let assign = |value: TokenStream2, source: &str| self.assign(value, source);
        if !env {
            if let Some(function) = &self.default_fn {
                let set = match &self.kind {
                    Kind::Many(_) => quote!(#slot = #function();),
                    _ => quote!(#slot = ::core::option::Option::Some(#function());),
                };
                return quote!(if #unset { #set });
            }
            return self
                .default
                .as_ref()
                .map(|value| {
                    let bytes = LitByteStr::new(value.as_bytes(), Span::call_site());
                    let assign = assign(quote!(__wa::BStr::new(#bytes)), &self.display());
                    quote!(if #unset { #assign })
                })
                .unwrap_or_default();
        }
        self.env
            .as_ref()
            .map(|var| {
                let name = LitStr::new(var, Span::call_site());
                let apply = match &self.kind {
                    Kind::Switch => quote!(#slot = __wa::env::truthy(&__raw);),
                    Kind::Count(ty) => quote! {
                        if let ::core::option::Option::Some(::core::result::Result::Ok(__n)) =
                            __raw.to_str().map(str::parse::<#ty>)
                        {
                            #slot = __n;
                        }
                    },
                    _ => {
                        let assign = assign(quote!(__value), &format!("${var}"));
                        quote! {
                            let __value = __wa::BStr::new(__raw.as_encoded_bytes());
                            #assign
                        }
                    }
                };
                quote! {
                    if #unset {
                        if let ::core::option::Option::Some(__raw) = __wa::env::var(#name) {
                            #apply
                        }
                    }
                }
            })
            .unwrap_or_default()
    }

    /// A negatable switch's fallback: only when neither spelling was given.
    fn negatable_fallback(&self, env: bool) -> TokenStream2 {
        let slot = slot(&self.ident);
        let value = if env {
            let Some(var) = &self.env else {
                return quote!();
            };
            let name = LitStr::new(var, Span::call_site());
            quote! {
                if let ::core::option::Option::Some(__raw) = __wa::env::var(#name) {
                    #slot = ::core::option::Option::Some(__wa::env::truthy(&__raw));
                }
            }
        } else {
            match (&self.default_fn, &self.default) {
                (Some(function), _) => quote!(#slot = ::core::option::Option::Some(#function());),
                (None, Some(default)) => {
                    let default = default == "true";
                    quote!(#slot = ::core::option::Option::Some(#default);)
                }
                (None, None) => return quote!(),
            }
        };
        quote!(if #slot.is_none() { #value })
    }

    /// A positional word converted to `ty`, checked against `choices` first.
    fn word_value(&self, ty: &Type, word: TokenStream2, display: &LitStr) -> TokenStream2 {
        let check = self.check(
            quote!(#word.value),
            |cause| quote!(__wa::Error::invalid_value(#word.offset, #display, #word.value, #cause)),
        );
        let (ty, unwrap) = self.parsed(ty);
        quote!({ #check #word.convert::<#ty>(#display)? #unwrap })
    }

    /// How errors name the field: its long flag, else its short one, else its value name.
    fn display(&self) -> String {
        match &self.role {
            Role::Flag { long: Some(l), .. } => format!("--{l}"),
            Role::Flag { short: Some(c), .. } => format!("-{c}"),
            Role::Flag { .. } => match self.plus {
                Some(c) => format!("+{c}"),
                None => unreachable!("every flag has a name"),
            },
            Role::Positional { name, .. } => name.clone(),
            Role::Subcommand => "<COMMAND>".to_owned(),
        }
    }

    /// The pattern of every letter it answers to: `'l' | 'L'`.
    fn short_pattern(&self) -> Option<TokenStream2> {
        let first = self.short()?;
        let letters = std::iter::once(first)
            .chain(self.short_aliases.iter().copied())
            .map(|c| LitChar::new(c, Span::call_site()));
        Some(quote!(#(#letters)|*))
    }

    /// Every letter it answers to.
    fn shorts(&self) -> Vec<char> {
        self.short()
            .into_iter()
            .chain(self.short_aliases.iter().copied())
            .collect()
    }

    fn short(&self) -> Option<char> {
        match self.role {
            Role::Flag { short, .. } => short,
            Role::Positional { .. } | Role::Subcommand => None,
        }
    }

    /// Every long spelling: the name, then its aliases.
    fn longs(&self) -> Vec<&str> {
        match &self.role {
            Role::Flag { long, aliases, .. } => {
                long.iter().chain(aliases).map(String::as_str).collect()
            }
            Role::Positional { .. } | Role::Subcommand => Vec::new(),
        }
    }

    /// The pattern matching any long spelling, if there is one.
    fn long_pattern(&self) -> Option<TokenStream2> {
        let longs = self.longs();
        (!longs.is_empty()).then(|| byte_patterns(&longs))
    }

    /// Whether a flag reads a value (rather than being a switch or a count).
    fn takes_value(&self) -> bool {
        !self.tristate && !matches!(self.kind, Kind::Switch | Kind::Count(_))
    }

    fn is_positional(&self) -> bool {
        matches!(self.role, Role::Positional { .. })
    }

    fn is_global(&self) -> bool {
        matches!(self.role, Role::Flag { global: true, .. })
    }

    /// A positional's relation to `--`; `Optional` for anything else.
    fn double_dash(&self) -> DoubleDash {
        match self.role {
            Role::Positional { double_dash, .. } => double_dash,
            _ => DoubleDash::Optional,
        }
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Args` can only be derived for structs",
        ));
    };
    // A unit struct (`struct Cli;`) has no fields, and `Cli {}` builds it.
    let no_fields = syn::punctuated::Punctuated::new();
    let named = match &data.fields {
        Fields::Named(named) => &named.named,
        Fields::Unit => &no_fields,
        Fields::Unnamed(_) => {
            return Err(syn::Error::new(
                data.fields.span(),
                "`Args` needs a struct with named fields, or a unit struct",
            ));
        }
    };
    for f in named {
        role_alone(f)?;
    }
    let StructOptions {
        groups,
        restart_token,
        default_subcommand,
        default_subcommand_flags,
        default_subcommand_help,
        default_subcommand_on_empty,
        flatten_help,
        next_help_heading,
        headings,
        examples,
        effect,
        outputs,
        exit_codes,
        arg_required_else_help,
        validate,
        name: program_name,
        version,
        about,
        long_about,
        after_help,
        after_long_help,
        disable_help_flag,
        disable_help_short,
        disable_version_flag,
        disable_help_subcommand,
        unknown_flags_value,
        long_only,
        plus_options,
    } = struct_options(input)?;
    let mut fields = named
        .iter()
        .filter(|f| !is_skipped(f) && !is_sequence(f) && !is_unknown(f) && !is_flatten(f))
        .map(field)
        .collect::<syn::Result<Vec<_>>>()?;
    for field in fields.iter_mut().filter(|field| field.heading.is_none()) {
        field.heading.clone_from(&next_help_heading);
    }
    let fields = fields;
    let skipped: Vec<&Ident> = named
        .iter()
        .filter(|f| is_skipped(f) && !is_sequence(f) && !is_unknown(f) && !is_flatten(f))
        .filter_map(|f| f.ident.as_ref())
        .collect();
    // `#[arg(sequence)] items: Vec<T>`: occurrences of `T`'s flags and
    // positional, in order.
    let sequences: Vec<&syn::Field> = named.iter().filter(|f| is_sequence(f)).collect();
    if let Some(extra) = sequences.get(1) {
        return Err(syn::Error::new(
            extra.span(),
            "a struct has at most one `sequence` field",
        ));
    }
    // `#[arg(sequence, unknown)]`: unknown flags join the sequence, through
    // the enum's `#[arg(unknown)]` variant.
    let sequence_unknown = sequences.first().is_some_and(|f| has_flag(f, "unknown"));
    let sequence = vec_field(
        sequences.first().copied(),
        "a `sequence` field is a `Vec<T>` of an `Occurrence` enum",
    )?;
    let sequence_ty = sequence.as_ref().map(|(_, ty)| ty);
    // `#[arg(unknown)] unknown: Vec<T>`: flag-like words naming no flag.
    let unknowns: Vec<&syn::Field> = named.iter().filter(|f| is_unknown(f)).collect();
    if let Some(extra) = unknowns.get(1) {
        return Err(syn::Error::new(
            extra.span(),
            "a struct has at most one `unknown` field",
        ));
    }
    let unknown = vec_field(
        unknowns.first().copied(),
        "an `unknown` field is a `Vec<T>`",
    )?;
    // `#[arg(flatten)] common: T`: `T`'s flags, parsed as if declared here.
    let flattens: Vec<(Ident, Type)> = named
        .iter()
        .filter(|f| is_flatten(f))
        .map(|f| (f.ident.clone().expect("named field"), f.ty.clone()))
        .collect();
    if !flattens.is_empty() && !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "`flatten` in a generic struct is not supported yet",
        ));
    }
    check_duplicates(&fields)?;
    // A `double_dash = "required"` positional is outside the ordinary sequence:
    // every word after `--` goes to it, and no word before.
    let positionals: Vec<&Field> = fields
        .iter()
        .filter(|f| f.is_positional() && f.double_dash() != DoubleDash::Required)
        .collect();
    check_positional_order(&positionals)?;
    let trailing: Vec<&Field> = fields
        .iter()
        .filter(|f| f.double_dash() == DoubleDash::Required)
        .collect();
    if let Some(extra) = trailing.get(1) {
        return Err(syn::Error::new(
            extra.ident.span(),
            "a struct has at most one `double_dash = \"required\"` positional",
        ));
    }
    let trailing = trailing.first().copied();
    // The positionals a `--` fills while they are next, by position.
    let preserving: Vec<usize> = positionals
        .iter()
        .enumerate()
        .filter(|(_, f)| f.double_dash() == DoubleDash::Preserve)
        .map(|(i, _)| i)
        .collect();
    if let (Some(t), false) = (trailing, preserving.is_empty()) {
        return Err(syn::Error::new(
            t.ident.span(),
            "a `double_dash = \"preserve\"` positional keeps the `--` this one waits for",
        ));
    }
    let (doc_about, doc_long_about) = docs(&input.attrs);
    let about = about.unwrap_or(doc_about);
    let long_about = long_about.unwrap_or(doc_long_about);
    let (about_text, long_about_text) = (about, long_about);
    let (after_text, after_long_text) = (after_help, after_long_help);
    let (about, long_about) = (text(&about_text), text(&long_about_text));
    let (after_help, after_long_help) = (text(&after_text), text(&after_long_text));
    let rules = Rules::new(&fields, &groups)?;
    // The program's own check of the built struct.
    let validate = validate.map(|path| quote!(#path(&__built)?;));

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Each field's slot: its name, type and starting value.
    let slot_parts: Vec<(Ident, TokenStream2, TokenStream2)> = fields
        .iter()
        .map(|f| {
            let none = quote!(::core::option::Option::None);
            let new = quote!(::std::vec::Vec::new());
            let (ty, init) = match &f.kind {
                // Borrows the command line, so never in a `Flatten` slot struct.
                _ if f.keywords => (quote!(::std::vec::Vec<&__wa::BStr>), new),
                Kind::Switch if f.negate.is_some() => (quote!(::core::option::Option<bool>), none),
                Kind::Switch => (quote!(bool), quote!(false)),
                Kind::Count(ty) => (quote!(#ty), quote!(0)),
                Kind::Optional(ty) | Kind::Required(ty) => {
                    (quote!(::core::option::Option<#ty>), none)
                }
                Kind::Many(ty) => (quote!(::std::vec::Vec<#ty>), new),
            };
            (slot(&f.ident), ty, init)
        })
        .collect();
    let slots = slot_parts
        .iter()
        .map(|(ident, ty, init)| quote!(let mut #ident: #ty = #init;));

    // What storing one flag occurrence looks like; the same for both spellings.
    let store = |f: &Field| {
        let ident = slot(&f.ident);
        let displace = rules.displace(&fields, f);
        let stored = match &f.kind {
            _ if f.keywords => {
                let read = f.read();
                quote! {
                    let __value = #read;
                    #ident.push(__value);
                }
            }
            _ if f.tristate || f.negate.is_some() => quote! {
                __arg.check_switch()?;
                #ident = ::core::option::Option::Some(true);
            },
            Kind::Switch => quote! {
                __arg.check_switch()?;
                #ident = true;
            },
            Kind::Count(_) => quote! {
                __arg.check_switch()?;
                #ident = #ident.saturating_add(1);
            },
            Kind::Required(ty) => {
                let value = f.flag_value(ty, quote!(__value));
                let read = f.read();
                quote! {
                    let __value = #read;
                    #ident = ::core::option::Option::Some(#value);
                }
            }
            Kind::Optional(ty) => {
                let value = f.flag_value(ty, quote!(__value));
                match &f.default_missing {
                    // `require_equals` is getopt's `optional_argument`: a bare
                    // flag holds `None`. `default_missing` names what a bare
                    // flag holds instead.
                    Some(_) => {
                        let read = f.read();
                        quote! {
                            let __value = #read;
                            #ident = ::core::option::Option::Some(#value);
                        }
                    }
                    None if f.require_equals => {
                        let options = f.value_options();
                        quote! {
                            if let ::core::option::Option::Some(__value) =
                                __arg.read_value_opt_with(__input, #options)
                            {
                                #ident = ::core::option::Option::Some(#value);
                            }
                        }
                    }
                    None => {
                        let read = f.read();
                        quote! {
                            let __value = #read;
                            #ident = ::core::option::Option::Some(#value);
                        }
                    }
                }
            }
            Kind::Many(ty) => match f.delimiter {
                None => {
                    let value = f.flag_value(ty, quote!(__value));
                    let read = f.read();
                    let more = match f.values_max {
                        // `values = N`: the words after the first, whatever they look like.
                        None => (f.values > 1).then(|| {
                            let more = f.values - 1;
                            quote! {
                                for _ in 0..#more {
                                    let __value = __arg.read_next(__input)?;
                                    #ident.push(#value);
                                }
                            }
                        }),
                        // `values = a..=b`: more words until one is flag-like or the terminator.
                        Some(max) => {
                            let (min, options) = (f.values, f.value_options());
                            let terminator = match &f.value_terminator {
                                Some(word) => {
                                    let word = LitByteStr::new(word.as_bytes(), Span::call_site());
                                    quote!(::core::option::Option::Some(__wa::BStr::new(#word)))
                                }
                                None => quote!(::core::option::Option::None),
                            };
                            Some(quote! {
                                let mut __taken = 1usize;
                                while __taken < #max {
                                    let ::core::option::Option::Some(__value) =
                                        __arg.read_more(__input, #options, #terminator)
                                    else {
                                        break;
                                    };
                                    #ident.push(#value);
                                    __taken += 1;
                                }
                                __arg.check_values(__taken, #min)?;
                            })
                        }
                    };
                    quote! {
                        let __value = #read;
                        #ident.push(#value);
                        #more
                    }
                }
                Some(d) => {
                    let value = f.flag_value(ty, quote!(__piece));
                    let read = f.read();
                    quote! {
                        for __piece in __wa::split(#read, #d) {
                            #ident.push(#value);
                        }
                    }
                }
            },
        };
        quote!(#stored #displace)
    };

    let negated = |f: &Field| {
        let no = LitByteStr::new(f.negate.as_ref()?.as_bytes(), Span::call_site());
        let ident = slot(&f.ident);
        let displace = rules.displace(&fields, f);
        Some((
            quote!(#no),
            quote! {
                __arg.check_switch()?;
                #ident = ::core::option::Option::Some(false);
                #displace
            },
        ))
    };
    // A `+c` occurrence: a tristate's `Some(false)`, else what `-c` stores.
    let plus = |f: &Field| {
        let letter = LitChar::new(f.plus?, Span::call_site());
        let body = if f.tristate {
            let ident = slot(&f.ident);
            let displace = rules.displace(&fields, f);
            quote!(#ident = ::core::option::Option::Some(false); #displace)
        } else {
            store(f)
        };
        Some((quote!(#letter), body))
    };
    // The long (negations last), short and `+` arms of the fields `keep`
    // selects, each body ending in `tail`.
    type Part<'a> = &'a dyn Fn(&Field) -> Option<(TokenStream2, TokenStream2)>;
    let arms = |keep: &dyn Fn(&Field) -> bool, tail: TokenStream2| {
        let of = |part: Part<'_>| -> Vec<TokenStream2> {
            fields
                .iter()
                .filter(|f| keep(f))
                .filter_map(part)
                .map(|(pattern, body)| quote!(#pattern => { #body #tail }))
                .collect()
        };
        let mut long = of(&|f| Some((f.long_pattern()?, store(f))));
        long.extend(of(&negated));
        let short = of(&|f| Some((f.short_pattern()?, store(f))));
        (long, short, of(&plus))
    };
    let (long_arms, short_arms, plus_arms) = arms(&|_| true, quote!());

    let unexpected = quote!(return ::core::result::Result::Err(__arg.unexpected()));
    let subcommands: Vec<&Field> = fields
        .iter()
        .filter(|f| matches!(f.role, Role::Subcommand))
        .collect();
    if let Some(extra) = subcommands.get(1) {
        return Err(syn::Error::new(
            extra.ident.span(),
            "a struct has at most one subcommand field",
        ));
    }
    let subcommand = subcommands.first();
    // A word routes to a subcommand only before any positional is filled.
    let track_filled = subcommand.is_some() && !positionals.is_empty();
    let filled = track_filled.then(|| quote!(__filled = true;));

    // A word no ordinary positional takes: before `--`, it would have reached
    // the `double_dash = "required"` positional if there is one.
    let overflow = match trailing {
        Some(t) => {
            let display = LitStr::new(&t.display(), Span::call_site());
            quote! {
                return ::core::result::Result::Err(
                    __wa::Error::requires_double_dash(__word.offset, #display),
                )
            }
        }
        None => unexpected.clone(),
    };
    let positional_match = if positionals.is_empty() {
        quote!({ #overflow; })
    } else {
        let arms = positionals.iter().enumerate().map(|(i, f)| {
            let ident = slot(&f.ident);
            let display = LitStr::new(&f.display(), Span::call_site());
            let stop = (f.double_dash() == DoubleDash::Automatic || f.stop_flags)
                .then(|| quote!(__input.stop_flags();));
            // Flags may have stopped at the positional before this one. A
            // `--` is then a word, and is still not this one's first value.
            let empty = match &f.kind {
                Kind::Many(_) => quote!(#ident.is_empty()),
                _ => quote!(true),
            };
            let separator = (f.double_dash() == DoubleDash::Automatic).then(|| {
                quote! {
                    #i if !__word.after_separator && &**__word.value == b"--" && #empty => {
                        __input.separate();
                    }
                }
            });
            let arm = match &f.kind {
                Kind::Optional(ty) | Kind::Required(ty) => {
                    let value = f.word_value(ty, quote!(__word), &display);
                    quote! {
                        #i => {
                            #ident = ::core::option::Option::Some(#value);
                            __position += 1;
                            #filled
                            #stop
                        }
                    }
                }
                Kind::Many(ty) => {
                    let push = match f.delimiter {
                        None => {
                            let value = f.word_value(ty, quote!(__word), &display);
                            // Sized once, for at most every word left: the
                            // usual tail of operands then needs no regrowth.
                            quote! {
                                if #ident.capacity() == 0 {
                                    #ident.reserve_exact(__input.words_left() + 1);
                                }
                                #ident.push(#value);
                            }
                        }
                        Some(d) => {
                            let value = f.word_value(ty, quote!(__piece), &display);
                            quote! {
                                for __piece in __word.split(#d) {
                                    #ident.push(#value);
                                }
                            }
                        }
                    };
                    quote! {
                        #i => {
                            #push
                            #filled
                            #stop
                        }
                    }
                }
                Kind::Switch | Kind::Count(_) => {
                    unreachable!("rejected for positionals in `field`")
                }
            };
            quote!(#separator #arm)
        });
        quote! {
            match __position {
                #(#arms)*
                _ => { #overflow; }
            }
        }
    };
    let positional_match = match trailing {
        None => positional_match,
        Some(t) => {
            let ident = slot(&t.ident);
            let display = LitStr::new(&t.display(), Span::call_site());
            let store = match &t.kind {
                Kind::Many(ty) => match t.delimiter {
                    None => {
                        let value = t.word_value(ty, quote!(__word), &display);
                        quote!(#ident.push(#value);)
                    }
                    Some(d) => {
                        let value = t.word_value(ty, quote!(__piece), &display);
                        quote!(for __piece in __word.split(#d) { #ident.push(#value); })
                    }
                },
                Kind::Optional(ty) | Kind::Required(ty) => {
                    let value = t.word_value(ty, quote!(__word), &display);
                    quote! {
                        if #ident.is_some() { #unexpected; }
                        #ident = ::core::option::Option::Some(#value);
                    }
                }
                Kind::Switch | Kind::Count(_) => {
                    unreachable!("rejected for positionals in `field`")
                }
            };
            quote! {
                if __word.after_separator {
                    #store
                } else {
                    #positional_match
                }
            }
        }
    };
    let routes = subcommand.map(|f| {
        let ident = slot(&f.ident);
        let ty = subcommand_type(f);
        let not_filled = track_filled.then(|| quote!(!__filled &&));
        let (global_longs, global_shorts, global_plus) = arms(
            &Field::is_global,
            quote!(return ::core::result::Result::Ok(true);),
        );
        // The subcommand sees this struct's globals first, those of the structs
        // it flattens, then its ancestors'.
        let flat_globals = flattens.iter().map(|(ident, ty)| {
            let slot = format_ident!("__flat_{}", ident);
            quote! {
                if <#ty as __wa::Flatten>::bind_global(&mut #slot, __arg, __input)? {
                    return ::core::result::Result::Ok(true);
                }
            }
        });
        let flat_shorts = flattens
            .iter()
            .map(|(_, ty)| quote!(.or_else(|| <#ty as __wa::Flatten>::global_short(__c))));
        let (inherit, handler) = if fields.iter().any(Field::is_global) || !flattens.is_empty() {
            let shorts = fields.iter().filter(|f| f.is_global()).flat_map(|f| {
                let takes_value = f.takes_value();
                f.shorts().into_iter().map(move |c| {
                    let c = LitChar::new(c, Span::call_site());
                    quote!((#c, #takes_value))
                })
            });
            let setup = quote! {
                let mut __inherit = __wa::inherit(
                    &[#(#shorts),*],
                    |__c| ::core::option::Option::<bool>::None #(#flat_shorts)*,
                    __globals,
                    |__arg, __input, __parent| {
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#global_longs)*
                            _ => {}
                        },
                        __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                            #(#global_plus)*
                            _ => {}
                        },
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#global_shorts)*
                            _ => {}
                        },
                        _ => {}
                    }
                    #(#flat_globals)*
                    __parent.bind(__arg, __input)
                });
            };
            (setup, quote!(&mut __inherit))
        } else {
            (quote!(), quote!(&mut *__globals))
        };
        // `help a b`: the help of a subcommand, unless one is actually named `help`.
        let help_word = (!disable_help_subcommand).then(|| {
            quote! {
                if &**__word.value == b"help" {
                    return ::core::result::Result::Err(
                        __wa::help_word(<Self as ::winnow_args::Args>::HELP, __input),
                    );
                }
            }
        });
        // Any other word selects the default subcommand, which reads it again as its own.
        let fallback = default_subcommand.as_ref().map(|name| {
            let name = LitByteStr::new(name.as_bytes(), Span::call_site());
            quote! {
                let __sub = {
                    #inherit
                    *__input = __start;
                    <#ty as __wa::Subcommand>::parse_subcommand(#name, __input, #handler)?
                };
                #ident = ::core::option::Option::Some(__sub);
                break;
            }
        });
        // A flag this struct does not take selects it too, and is read again as its own.
        let flag_fallback = fallback.as_ref().filter(|_| default_subcommand_flags).map(
            |fallback| match track_filled {
                true => quote!(if __filled { #unexpected; } #fallback),
                false => fallback.clone(),
            },
        );
        let route = quote! {
            if #not_filled !__word.after_separator {
                if <#ty as __wa::Subcommand>::has(&**__word.value) {
                    let __sub = {
                        #inherit
                        <#ty as __wa::Subcommand>::parse_subcommand(
                            &**__word.value, __input, #handler,
                        )?
                    };
                    #ident = ::core::option::Option::Some(__sub);
                    break;
                }
                #help_word
                #fallback
            }
        };
        // A line that took no subcommand, no positional and no `--` runs the
        // default subcommand, with no words: usage's rule.
        let on_empty = default_subcommand
            .as_ref()
            .filter(|_| default_subcommand_on_empty)
            .map(|name| {
                let name = LitByteStr::new(name.as_bytes(), Span::call_site());
                let no_positional = track_filled.then(|| quote!(&&!__filled));
                quote! {
                    if #ident.is_none()
                        #no_positional
                        && __input.mode() != ::winnow_args::stream::Mode::Stopped
                    {
                        let __sub = {
                            #inherit
                            <#ty as __wa::Subcommand>::parse_subcommand(#name, __input, #handler)?
                        };
                        #ident = ::core::option::Option::Some(__sub);
                    }
                }
            });
        (route, flag_fallback, on_empty)
    });
    let (route, flag_fallback, on_empty) = match routes {
        Some((route, flag_fallback, on_empty)) => (Some(route), flag_fallback, on_empty),
        None => (None, None, None),
    };
    // A new invocation of the same command: positionals start over, flags keep theirs.
    let restart = restart_token.map(|token| {
        let token = LitByteStr::new(token.as_bytes(), Span::call_site());
        let clears = fields.iter().filter(|f| f.is_positional()).map(|f| {
            let slot = slot(&f.ident);
            match f.kind {
                Kind::Many(_) => quote!(#slot.clear();),
                _ => quote!(#slot = ::core::option::Option::None;),
            }
        });
        let position = (!positionals.is_empty()).then(|| quote!(__position = 0;));
        let filled = track_filled.then(|| quote!(__filled = false;));
        quote! {
            if &**__word.value == #token {
                #(#clears)*
                #position
                #filled
                __input.resume_flags();
                continue;
            }
        }
    });
    let sequence_word = sequence_ty.map(|ty| {
        quote! {
            if let ::core::option::Option::Some(__item) =
                <#ty as ::winnow_args::Occurrence>::from_word(&__word)?
            {
                __sequence.push(__item);
                continue;
            }
        }
    });
    let word_arm = quote! {
        __wa::Arg::Word(__word) => {
            #restart
            #route
            #sequence_word
            #positional_match
        }
    };
    let separator_arm = if preserving.is_empty() {
        quote!(__wa::Arg::Separator { .. } => {})
    } else {
        quote! {
            __wa::Arg::Separator { offset: __offset } => {
                if matches!(__position, #(#preserving)|*) {
                    __input.resume_flags();
                    let __word = __wa::Word {
                        value: __wa::BStr::new(b"--"),
                        offset: __offset,
                        after_separator: false,
                    };
                    #positional_match
                }
            }
        }
    };
    let position = (!positionals.is_empty()).then(|| quote!(let mut __position: usize = 0;));
    let filled_slot = track_filled.then(|| quote!(let mut __filled = false;));

    // A negative number is a word where the positional next to fill opts in,
    // unless it spells a declared digit short exactly (`-0` for fd's `--print0`).
    let opted: Vec<usize> = positionals
        .iter()
        .enumerate()
        .filter(|(_, f)| f.negative_numbers)
        .map(|(i, _)| i)
        .collect();
    let (unknown_slot, unknown_build) = match &unknown {
        Some((ident, ty)) => (
            quote!(let mut __unknown: ::std::vec::Vec<#ty> = ::std::vec::Vec::new();),
            quote!(#ident: __unknown,),
        ),
        None => (quote!(), quote!()),
    };
    let (sequence_slot, sequence_arg, sequence_build) = match &sequence {
        Some((ident, ty)) => (
            // Room for one item a word: a linker's line is mostly inputs.
            quote! {
                let mut __sequence: ::std::vec::Vec<#ty> =
                    ::std::vec::Vec::with_capacity(__input.words_left());
            },
            quote! {
                if let ::core::option::Option::Some(__item) =
                    <#ty as ::winnow_args::Occurrence>::from_arg(&__arg, __input)?
                {
                    __sequence.push(__item);
                } else
            },
            quote!(#ident: __sequence,),
        ),
        None => (quote!(), quote!(), quote!()),
    };
    // A word of several short flags (`-sS`), where the enum wants to know:
    // once its first letter is read, whoever took it, the `bundle` item goes
    // before the items of its letters.
    let bundle_start = sequence_ty.map(|_| {
        quote! {
            let __bundle_word = __input.front();
            let __bundle_start = __input.mode() == ::winnow_args::stream::Mode::Word;
            let __bundle_at = __sequence.len();
        }
    });
    let bundle_end = sequence_ty.map(|ty| {
        quote! {
            if <#ty as ::winnow_args::Occurrence>::BUNDLES
                && __bundle_start
                && matches!(__arg, __wa::Arg::Short(__flag) if !__flag.plus)
                && __input.mode() == ::winnow_args::stream::Mode::Bundle
            {
                let __word = __wa::Word {
                    value: __wa::BStr::new(__bundle_word),
                    offset: __arg.offset(),
                    after_separator: false,
                };
                if let ::core::option::Option::Some(__bundle) =
                    <#ty as ::winnow_args::Occurrence>::from_bundle(&__word)?
                {
                    __sequence.insert(__bundle_at, __bundle);
                }
            }
        }
    });
    // `#[arg(flatten)]` fields: their slots, the flags offered to them after
    // ours, and their values built at the end.
    let flat_slot = |ident: &Ident| format_ident!("__flat_{}", ident);
    let flatten_slots = flattens.iter().map(|(ident, ty)| {
        let slot = flat_slot(ident);
        quote! {
            let mut #slot = <<#ty as __wa::Flatten>::Slots as ::core::default::Default>::default();
        }
    });
    let flatten_arg: TokenStream2 = flattens
        .iter()
        .map(|(ident, ty)| {
            let slot = flat_slot(ident);
            quote!(if <#ty as __wa::Flatten>::bind(&mut #slot, &__arg, __input)? {} else)
        })
        .collect();
    let flatten_build = flattens.iter().map(|(ident, ty)| {
        let slot = flat_slot(ident);
        quote!(#ident: <#ty as __wa::Flatten>::finish(#slot, __input)?,)
    });
    let flatten_shorts: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| {
            quote! {
                __c if <#ty as __wa::Flatten>::short(__c).is_some() => {
                    <#ty as __wa::Flatten>::short(__c)
                }
            }
        })
        .collect();
    let flatten_longs: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_long(__name)))
        .collect();
    // Whether each of our letters takes a value, as `short` answers.
    let own_shorts: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let letters = f.short_pattern()?;
            let takes_value = f.takes_value();
            Some(quote!(#letters => ::core::option::Option::Some(#takes_value),))
        })
        .collect();
    // The long names one dash may spell under `long_only`, and the letters
    // that never start one.
    let own_longs: Vec<String> = fields
        .iter()
        .filter(|f| !f.two_dashes)
        .flat_map(|f| spellings(f).into_iter().map(str::to_owned))
        .collect();
    let prefixes: Vec<u8> = fields
        .iter()
        .filter(|f| f.prefix)
        .filter_map(|f| f.short().map(|c| c as u8))
        .collect();
    let flatten_prefixes = flattens.iter().map(|(_, ty)| {
        quote!(&& !__name.first().is_some_and(|__c| <#ty as __wa::Flatten>::is_prefix(*__c)))
    });
    let lexer = if plus_options {
        quote!(__wa::arg_plus)
    } else {
        quote!(__wa::arg)
    };
    if !plus_options && let Some(f) = fields.iter().find(|f| f.plus.is_some()) {
        return Err(syn::Error::new(
            f.ident.span(),
            "`plus` needs `#[arg(plus_options)]` on the struct",
        ));
    }
    // A `+x` this struct does not take selects the default subcommand as a flag does.
    let plus_unknown = match &flag_fallback {
        Some(fallback) => quote!(#fallback),
        None => quote!(#unexpected;),
    };
    let plus_match = plus_options.then(|| {
        quote! {
            __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                #(#plus_arms)*
                _ => #flatten_arg if !__globals.bind(&__arg, __input)? { #plus_unknown }
            },
        }
    });
    let lex = if opted.is_empty() {
        quote!(#lexer(__input)?)
    } else {
        let digits: Vec<LitByteStr> = fields
            .iter()
            .filter_map(|f| f.short().filter(char::is_ascii_digit))
            .map(|c| LitByteStr::new(format!("-{c}").as_bytes(), Span::call_site()))
            .collect();
        let not_a_short =
            (!digits.is_empty()).then(|| quote!(&&!matches!(__input.front(), #(#digits)|*)));
        quote! {
            match matches!(__position, #(#opted)|*) #not_a_short {
                true => match __wa::number(__input) {
                    ::core::result::Result::Ok(__word) => __wa::Arg::Word(__word),
                    ::core::result::Result::Err(_) => #lexer(__input)?,
                },
                false => #lexer(__input)?,
            }
        }
    };
    let start = default_subcommand
        .is_some()
        .then(|| quote!(let __start = *__input;));
    let help_on_empty = arg_required_else_help.then(|| {
        quote! {
            if __input.is_empty() {
                return ::core::result::Result::Err(
                    __wa::Error::bare_help(<Self as ::winnow_args::Args>::HELP),
                );
            }
        }
    });
    if default_subcommand.is_some() && subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand` needs an `#[arg(subcommand)]` field",
        ));
    }
    if default_subcommand_flags && default_subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand_flags` needs `default_subcommand`",
        ));
    }
    if default_subcommand_on_empty && default_subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand_on_empty` needs `default_subcommand`",
        ));
    }
    if default_subcommand_on_empty && arg_required_else_help {
        return Err(syn::Error::new(
            input.ident.span(),
            "an empty command line either asks for help or runs the default subcommand, not both",
        ));
    }
    if default_subcommand_help && default_subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand_help` needs `default_subcommand`",
        ));
    }
    if flatten_help && subcommand.is_none() {
        return Err(syn::Error::new(
            input.ident.span(),
            "`flatten_help` needs an `#[arg(subcommand)]` field",
        ));
    }
    if default_subcommand_flags && (unknown_flags_value || unknown.is_some() || sequence_unknown) {
        return Err(syn::Error::new(
            input.ident.span(),
            "`default_subcommand_flags` hands an unknown flag to the default subcommand: \
             this struct cannot keep unknown flags too",
        ));
    }
    // `-h`/`--help` and `-V`/`--version`, unless declared or disabled: arms after
    // the struct's own, so a CLI naming its own `--help` keeps it.
    let declares_long = |name: &str| fields.iter().any(|f| f.longs().contains(&name));
    let declares_short = |c: char| fields.iter().any(|f| f.shorts().contains(&c));
    let help = quote!(<Self as ::winnow_args::Args>::HELP);
    let help_long = (!disable_help_flag && !declares_long("help")).then(
        || quote!(b"help" => return ::core::result::Result::Err(__wa::Error::help(#help, true)),),
    );
    let help_short = (!disable_help_flag && !disable_help_short && !declares_short('h')).then(
        || quote!('h' => return ::core::result::Result::Err(__wa::Error::help(#help, false)),),
    );
    let with_version = version.is_some() && !disable_version_flag;
    let version_long = (with_version && !declares_long("version")).then(
        || quote!(b"version" => return ::core::result::Result::Err(__wa::Error::version(#help)),),
    );
    let version_short = (with_version && !declares_short('V'))
        .then(|| quote!('V' => return ::core::result::Result::Err(__wa::Error::version(#help)),));
    // The row help prints is `--help`'s: a struct with its own has none.
    let help_flag = help_long.is_some();
    let help_short_flag = help_short.is_some();
    let version_short_flag = version_short.is_some();
    // A flattened struct or a sequence enum may declare `-h`, `--help`, `-V`
    // or `--version` itself (ld's `-h SONAME`): the built-in ones are then
    // tried after them, not as arms before.
    let late = !flattens.is_empty() || sequence.is_some();
    let late_if = |on: bool, test: TokenStream2, error: TokenStream2| {
        (late && on).then(|| quote!(if #test { return ::core::result::Result::Err(#error); } else))
    };
    let late_long: TokenStream2 = [
        late_if(
            help_long.is_some(),
            quote!(__flag.name == b"help"),
            quote!(__wa::Error::help(#help, true)),
        ),
        late_if(
            version_long.is_some(),
            quote!(__flag.name == b"version"),
            quote!(__wa::Error::version(#help)),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let late_short: TokenStream2 = [
        late_if(
            help_short.is_some(),
            quote!(__flag.letter == 'h'),
            quote!(__wa::Error::help(#help, false)),
        ),
        late_if(
            version_short.is_some(),
            quote!(__flag.letter == 'V'),
            quote!(__wa::Error::version(#help)),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    let keep = |arm: Option<TokenStream2>| arm.filter(|_| !late);
    let (help_long, help_short) = (keep(help_long), keep(help_short));
    let (version_long, version_short) = (keep(version_long), keep(version_short));

    // `long_only`: `-name` is `--name` for the long names that may take one
    // dash, tried before the word is read as short letters.
    // `unknown_flags = "value"`: a flag-like word naming no flag goes to the
    // positionals whole, never to a subcommand. A bundle of several letters is
    // checked before any binds; a long word or a single letter once no arm or
    // global took it.
    // An `unknown` field takes those words instead of the positionals.
    // Only the first letter of a word reaches the short arm's sink: the bundle
    // check vouched for the letters of a longer one.
    let (lenient_bundle, save_token, unknown_flag) = if unknown_flags_value
        || unknown.is_some()
        || sequence_unknown
    {
        // With nothing to take a word, `positional_match` always returns.
        let next = (!positionals.is_empty() || trailing.is_some()).then(|| quote!(continue;));
        let (sink, sink_next) = match (&unknown, sequence_ty.filter(|_| sequence_unknown)) {
            (_, Some(ty)) => (
                quote! {
                    match <#ty as ::winnow_args::Occurrence>::from_unknown(&__word)? {
                        ::core::option::Option::Some(__item) => __sequence.push(__item),
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(__wa::Error::unknown_flag(
                                __word.offset,
                                ::std::string::String::from_utf8_lossy(&__word.value),
                            ));
                        }
                    }
                },
                quote!(continue;),
            ),
            (Some((_, ty)), None) => (
                quote!(__unknown.push(__word.convert::<#ty>("<UNKNOWN>")?);),
                quote!(continue;),
            ),
            (None, None) => (
                quote! {
                    let __arg = __wa::Arg::Word(__word);
                    #positional_match
                },
                quote!(#next),
            ),
        };
        let theirs = match sequence_ty {
            Some(ty) => quote! {
                __c => match <#ty as ::winnow_args::Occurrence>::short(__c) {
                    ::core::option::Option::None => __globals.short(__c),
                    __known => __known,
                },
            },
            None => quote!(_ => __globals.short(__c),),
        };
        // Known letters whether their arms come first or late.
        let builtin = help_short_flag
            .then(|| quote!('h' => ::core::option::Option::Some(false),))
            .into_iter()
            .chain(version_short_flag.then(|| quote!('V' => ::core::option::Option::Some(false),)));
        (
            quote! {
                if let ::core::option::Option::Some(__word) = __wa::unknown_bundle(__input, |__c| match __c {
                    #(#own_shorts)*
                    #(#builtin)*
                    #(#flatten_shorts)*
                    #theirs
                }) {
                    #sink
                    #sink_next
                }
            },
            quote!(let __token = __input.front();),
            quote! {
                let __word = __wa::Word {
                    value: __wa::BStr::new(__token),
                    offset: __flag.offset,
                    after_separator: false,
                };
                #sink
            },
        )
    } else {
        let unknown_flag = match &flag_fallback {
            Some(fallback) => quote!(#fallback),
            None => quote!(#unexpected;),
        };
        (quote!(), quote!(), unknown_flag)
    };
    let mut names = Vec::new();
    if long_only {
        names.clone_from(&own_longs);
        names.extend(help_long.is_some().then(|| "help".to_owned()));
        names.extend(version_long.is_some().then(|| "version".to_owned()));
    }
    let long_only_lex =
        long_only && (!names.is_empty() || sequence_ty.is_some() || !flattens.is_empty());
    let lex = if long_only_lex {
        let own = matches_name(&names);
        let (their_names, their_prefixes) = match sequence_ty {
            Some(ty) => (
                quote!(|| <#ty as ::winnow_args::Occurrence>::is_long(__name)),
                quote! {
                    && !__name.first().is_some_and(|__c| {
                        <#ty as ::winnow_args::Occurrence>::PREFIXES.contains(__c)
                    })
                },
            ),
            None => (quote!(), quote!()),
        };
        let not_prefixed = (!prefixes.is_empty()).then(
            || quote!(!matches!(__name.first(), ::core::option::Option::Some(#(#prefixes)|*)) &&),
        );
        quote! {
            match __wa::long_only(__input, |__name| {
                #not_prefixed (#own #their_names #(#flatten_longs)*) #their_prefixes #(#flatten_prefixes)*
            }) {
                ::core::option::Option::Some(__arg) => __arg,
                ::core::option::Option::None => {
                    #lenient_bundle
                    #lex
                }
            }
        }
    } else {
        lex
    };
    let lenient_bundle = if long_only_lex {
        quote!()
    } else {
        lenient_bundle
    };

    let help_version = match &version {
        Some(v) if with_version => quote!(::core::option::Option::Some(#v)),
        _ => quote!(::core::option::Option::None),
    };
    let help_items = in_help_order(&fields)
        .into_iter()
        .filter(|f| !matches!(f.role, Role::Subcommand))
        .map(|f| f.help_item());
    let (help_subcommands, subcommand_required) = match subcommand {
        Some(f) => {
            let ty = subcommand_type(f);
            (
                quote!(<#ty as ::winnow_args::Args>::HELP.subcommands),
                matches!(f.kind, Kind::Required(_)),
            )
        }
        None => (quote!(&[]), false),
    };
    let displaced = rules.displaced_slots(&fields);
    let env_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| f.fallback(true, rules.is_displaced(&fields, f)))
        .collect();
    let default_fallbacks: Vec<TokenStream2> = fields
        .iter()
        .map(|f| f.fallback(false, rules.is_displaced(&fields, f)))
        .collect();
    let exclusive = rules.exclusive(&fields, &groups);
    let supplied = rules.supplied(&fields);
    let default_ifs = rules.default_ifs(&fields);
    let default_froms = default_froms(&fields, &rules, !input.generics.params.is_empty())?;
    let required = rules.required(&fields, &groups);
    let build = fields.iter().map(|f| {
        let ident = &f.ident;
        let slot = slot(ident);
        match &f.kind {
            Kind::Required(ty) if f.keywords => {
                let display = LitStr::new(&f.display(), Span::call_site());
                quote!(#ident: __wa::keywords::<#ty>(&#slot, #display, __input.offset())?)
            }
            Kind::Switch if f.negate.is_some() => quote!(#ident: #slot.unwrap_or(false)),
            Kind::Switch | Kind::Count(_) | Kind::Optional(_) | Kind::Many(_) => {
                quote!(#ident: #slot)
            }
            Kind::Required(_) => {
                let display = LitStr::new(&f.display(), Span::call_site());
                let error = match f.role {
                    Role::Positional { .. } => {
                        quote!(__wa::Error::missing_argument(__input.offset(), #display))
                    }
                    Role::Flag { .. } => {
                        quote!(__wa::Error::missing_required(__input.offset(), #display))
                    }
                    Role::Subcommand => quote!(__wa::Error::missing_subcommand(__input.offset())),
                };
                quote! {
                    #ident: match #slot {
                        ::core::option::Option::Some(v) => v,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(#error);
                        }
                    }
                }
            }
        }
    });
    let build: Vec<TokenStream2> = build.collect();
    // Every field read once after building, so a flag accepted and ignored on
    // purpose (`pwd -L`) is not dead code in the user's crate. No runtime work
    // after optimization.
    let field_reads: Vec<TokenStream2> = fields
        .iter()
        .map(|f| &f.ident)
        .chain(skipped.iter().copied())
        .chain(sequence.iter().chain(&unknown).map(|(ident, _)| ident))
        .chain(flattens.iter().map(|(ident, _)| ident))
        .map(|ident| quote!(let _ = &__built.#ident;))
        .collect();
    let flatten_build: Vec<TokenStream2> = flatten_build.collect();
    // Flags only: no positionals, subcommand, `sequence`, `unknown`,
    // keywords or generics.
    let flattenable = positionals.is_empty()
        && trailing.is_none()
        && subcommand.is_none()
        && sequence.is_none()
        && unknown.is_none()
        && !unknown_flags_value
        && input.generics.params.is_empty()
        && !fields.iter().any(|f| f.keywords);
    let flatten_impl = flattenable.then(|| {
        let vis = &input.vis;
        let slots_name = format_ident!("__{}WinnowArgsSlots", name);
        let displaced_idents: Vec<Ident> = fields
            .iter()
            .filter(|f| rules.is_displaced(&fields, f))
            .map(|f| displaced_flag(&f.ident))
            .collect();
        let nested: Vec<(Ident, &Type)> = flattens
            .iter()
            .map(|(ident, ty)| (flat_slot(ident), ty))
            .collect();
        let decls = slot_parts.iter().map(|(ident, ty, _)| quote!(#ident: #ty,));
        let inits = slot_parts
            .iter()
            .map(|(ident, _, init)| quote!(#ident: #init,));
        let all: Vec<&Ident> = slot_parts
            .iter()
            .map(|(ident, _, _)| ident)
            .chain(&displaced_idents)
            .chain(nested.iter().map(|(n, _)| n))
            .collect();
        let names: Vec<TokenStream2> = all.iter().map(|ident| quote!(#ident,)).collect();
        let mutable: Vec<TokenStream2> = all.iter().map(|ident| quote!(mut #ident,)).collect();
        let displaced_decls = displaced_idents.iter().map(|d| quote!(#d: bool,));
        let displaced_inits = displaced_idents.iter().map(|d| quote!(#d: false,));
        let nested_decls = nested
            .iter()
            .map(|(n, ty)| quote!(#n: <#ty as ::winnow_args::__private::Flatten>::Slots,));
        let nested_inits = nested
            .iter()
            .map(|(n, _)| quote!(#n: ::core::default::Default::default(),));
        let (bind_long, bind_short, bind_plus) = arms(&|_| true, quote!(true));
        // The same for the `global` ones alone: what a subcommand of the
        // struct that flattens these is offered.
        let (global_long, global_short, global_plus) = arms(&Field::is_global, quote!(true));
        let global_nested: TokenStream2 = nested
            .iter()
            .map(|(n, ty)| {
                quote!(if <#ty as __wa::Flatten>::bind_global(&mut #n, &__arg, __input)? { true } else)
            })
            .collect();
        let global_letters = fields.iter().filter(|f| f.is_global()).filter_map(|f| {
            let letters = f.short_pattern()?;
            let takes_value = f.takes_value();
            Some(quote!(#letters => ::core::option::Option::Some(#takes_value),))
        });
        let nested_global_letters = nested
            .iter()
            .map(|(_, ty)| quote!(.or_else(|| <#ty as __wa::Flatten>::global_short(__c))));
        let bind_nested: TokenStream2 = nested
            .iter()
            .map(|(n, ty)| {
                quote!(if <#ty as __wa::Flatten>::bind(&mut #n, &__arg, __input)? { true } else)
            })
            .collect();
        // A `+x` word is offered to the `+` arms only, here and below.
        let plus = if plus_options {
            quote! {
                __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                    #(#bind_plus)*
                    _ => #bind_nested { false }
                },
            }
        } else {
            quote!(__wa::Arg::Short(__flag) if __flag.plus => #bind_nested { false },)
        };
        let own_longs = matches_name(&own_longs);
        let own_prefixes = if prefixes.is_empty() {
            quote!(false)
        } else {
            quote!(matches!(__c, #(#prefixes)|*))
        };
        let nested_prefixes = nested
            .iter()
            .map(|(_, ty)| quote!(|| <#ty as __wa::Flatten>::is_prefix(__c)));
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
            #vis struct #slots_name {
                #(#decls)*
                #(#displaced_decls)*
                #(#nested_decls)*
            }

            #[automatically_derived]
            impl ::core::default::Default for #slots_name {
                fn default() -> Self {
                    Self {
                        #(#inits)*
                        #(#displaced_inits)*
                        #(#nested_inits)*
                    }
                }
            }

            #[automatically_derived]
            #[allow(
                unused_mut,
                unused_variables,
                unreachable_code,
                clippy::all,
                clippy::pedantic,
                clippy::nursery,
                clippy::restriction
            )]
            impl ::winnow_args::__private::Flatten for #name {
                type Slots = #slots_name;

                fn bind<'__i>(
                    __slots: &mut #slots_name,
                    __arg: &::winnow_args::Arg<'__i>,
                    __input: &mut ::winnow_args::Argv<'__i>,
                ) -> ::core::result::Result<bool, ::winnow_args::Error> {
                    use ::winnow_args::__private as __wa;
                    let __arg = *__arg;
                    let #slots_name { #(#mutable)* } = ::core::mem::take(__slots);
                    let __bound = match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#bind_long)*
                            _ => #bind_nested { false }
                        },
                        #plus
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#bind_short)*
                            _ => #bind_nested { false }
                        },
                        _ => false,
                    };
                    *__slots = #slots_name { #(#names)* };
                    ::core::result::Result::Ok(__bound)
                }

                fn short(__c: char) -> ::core::option::Option<bool> {
                    use ::winnow_args::__private as __wa;
                    match __c {
                        #(#own_shorts)*
                        #(#flatten_shorts)*
                        _ => ::core::option::Option::None,
                    }
                }

                fn bind_global<'__i>(
                    __slots: &mut #slots_name,
                    __arg: &::winnow_args::Arg<'__i>,
                    __input: &mut ::winnow_args::Argv<'__i>,
                ) -> ::core::result::Result<bool, ::winnow_args::Error> {
                    use ::winnow_args::__private as __wa;
                    let __arg = *__arg;
                    let #slots_name { #(#mutable)* } = ::core::mem::take(__slots);
                    let __bound = match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#global_long)*
                            _ => #global_nested { false }
                        },
                        __wa::Arg::Short(__flag) if __flag.plus => match __flag.letter {
                            #(#global_plus)*
                            _ => #global_nested { false }
                        },
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#global_short)*
                            _ => #global_nested { false }
                        },
                        _ => false,
                    };
                    *__slots = #slots_name { #(#names)* };
                    ::core::result::Result::Ok(__bound)
                }

                fn global_short(__c: char) -> ::core::option::Option<bool> {
                    use ::winnow_args::__private as __wa;
                    let __own = match __c {
                        #(#global_letters)*
                        _ => ::core::option::Option::None,
                    };
                    __own #(#nested_global_letters)*
                }

                fn is_prefix(__c: u8) -> bool {
                    use ::winnow_args::__private as __wa;
                    #own_prefixes #(#nested_prefixes)*
                }

                fn is_long(__name: &[u8]) -> bool {
                    use ::winnow_args::__private as __wa;
                    #own_longs #(#flatten_longs)*
                }

                fn finish(
                    __slots: #slots_name,
                    __input: &::winnow_args::Argv<'_>,
                ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                    use ::winnow_args::__private as __wa;
                    let #slots_name { #(#mutable)* } = __slots;
                    #(#env_fallbacks)*
                    #exclusive
                    #supplied
                    #default_ifs
                    #(#default_fallbacks)*
                    #default_froms
                    #required
                    let __built = Self {
                        #(#build,)*
                        #(#skipped: ::core::default::Default::default(),)*
                        #(#flatten_build)*
                    };
                    #(#field_reads)*
                    #validate
                    ::core::result::Result::Ok(__built)
                }
            }
        }
    });
    let own_items = quote!(&[#(#help_items),*]);
    // A sequence enum with a positional variant takes every word.
    let positional_clash = sequence_ty
        .filter(|_| !positionals.is_empty() || trailing.is_some())
        .map(|ty| {
            quote! {
                const _: () = assert!(
                    !<#ty as ::winnow_args::Occurrence>::POSITIONAL,
                    "the sequence enum has a positional variant, which takes every word: this struct's positionals would never be filled",
                );
            }
        });
    // Ours, then the flattened structs', then the sequence enum's.
    let more: Vec<TokenStream2> = flattens
        .iter()
        .map(|(_, ty)| quote!(<#ty as ::winnow_args::Args>::HELP.items))
        .chain(
            sequence_ty
                .iter()
                .map(|ty| quote!(<#ty as ::winnow_args::Occurrence>::ITEMS)),
        )
        .collect();
    let items = if more.is_empty() {
        own_items
    } else {
        quote!({
            const __OWN: &[::winnow_args::help::Item] = #own_items;
            const _: () = assert!(
                !::winnow_args::help::items_clash(&[__OWN, #(#more),*]),
                "a flattened struct or the sequence enum declares a flag that this struct, or another of them, also declares",
            );
            #positional_clash
            const __N: usize = __OWN.len() #(+ #more.len())*;
            const __ALL: [::winnow_args::help::Item; __N] = ::winnow_args::__private::concat_items::<__N>(
                &[__OWN, #(#more),*],
            );
            &__ALL
        })
    };
    let help_default = opt_str(default_subcommand.as_deref());
    let help_examples = examples.iter().map(|[code, header, help]| {
        let (header, help) = (text(header), text(help));
        quote!(::winnow_args::help::Example { code: #code, header: #header, help: #help })
    });
    let help_examples: Vec<TokenStream2> = help_examples.collect();
    // Ours first: what this struct says of a heading is what is read.
    let own_headings = headings.iter().map(|(title, help)| {
        let help = text(help);
        quote!(::winnow_args::help::Heading { title: #title, help: #help })
    });
    let own_headings = quote!(&[#(#own_headings),*]);
    let help_headings = if flattens.is_empty() {
        own_headings
    } else {
        let more = flattens
            .iter()
            .map(|(_, ty)| quote!(<#ty as ::winnow_args::Args>::HELP.headings));
        let more: Vec<TokenStream2> = more.collect();
        quote!({
            const __OWN: &[::winnow_args::help::Heading] = #own_headings;
            const __N: usize = __OWN.len() #(+ #more.len())*;
            const __ALL: [::winnow_args::help::Heading; __N] =
                ::winnow_args::__private::concat_headings::<__N>(&[__OWN, #(#more),*]);
            &__ALL
        })
    };
    let mut spec_checks = Vec::new();
    let filed = filed(input)?;
    let identity = spec::identity(input.ident.span(), &filed);
    if spec::enabled() {
        spec_checks.extend(fields.iter().filter_map(|field| {
            spec_check(&input.generics, choices_type(field), field.spec.as_ref())
                .or_else(|| spec_check_stated(&input.generics, field))
        }));
        // The name stated on a `flatten` or `sequence` field, which `field`
        // does not read.
        let stated = |ident: &Ident| {
            let field = named.iter().find(|f| f.ident.as_ref() == Some(ident));
            field.map_or(Ok(None), |f| spec_name(&f.attrs))
        };
        let mut link = |kind: TokenStream2, stated: Option<String>, ty: &Type| {
            let name = spec_type(stated, ty)?;
            spec_checks.extend(spec_filed(&input.generics, kind, ty, &name));
            syn::Result::Ok(name)
        };
        for field in fields.iter().filter(|field| field.keywords) {
            if let Kind::Required(ty) = &field.kind {
                link(quote!(Args), field.spec.clone(), ty)?;
            }
        }
        let items = in_help_order(&fields)
            .into_iter()
            .filter(|field| !matches!(field.role, Role::Subcommand))
            .map(spec_item)
            .collect::<syn::Result<Vec<_>>>()?;
        let flatten = flattens
            .iter()
            .map(|(ident, ty)| link(quote!(Args), stated(ident)?, ty))
            .collect::<syn::Result<Vec<_>>>()?;
        let sequence = sequence
            .as_ref()
            .map(|(ident, ty)| link(quote!(Occurrence), stated(ident)?, ty))
            .transpose()?;
        let subcommand_ty = subcommand
            .map(|field| {
                link(
                    quote!(Subcommand),
                    field.spec.clone(),
                    subcommand_type(field),
                )
            })
            .transpose()?;
        spec::write_args(
            input.ident.span(),
            &spec::ArgsDoc {
                ident: filed.clone(),
                name: program_name.clone(),
                about: about_text,
                long_about: long_about_text,
                after_help: after_text,
                after_long_help: after_long_text,
                package_version: package_version(&version, with_version),
                items,
                flatten,
                sequence,
                subcommand: subcommand_ty,
                subcommand_required,
                help_flag,
                help_short: help_short_flag,
                long_only,
                unknown_flags_value,
                default_subcommand,
                examples,
                headings,
                effect,
                outputs,
                exit_codes,
            },
        )?;
    }

    Ok(quote! {
        #(#spec_checks)*
        impl #impl_generics ::winnow_args::Args for #name #ty_generics #where_clause {
            const SPEC: &'static str = #identity;

            // Compiled here and never inlined, so a program in another crate has
            // no copy of the help.
            #[inline(never)]
            fn help_data() -> &'static ::winnow_args::help::Command {
                <Self as ::winnow_args::Args>::HELP
            }

            const HELP: &'static ::winnow_args::help::Command = &::winnow_args::help::Command {
                name: #program_name,
                about: #about,
                long_about: #long_about,
                after_help: #after_help,
                after_long_help: #after_long_help,
                items: #items,
                subcommands: #help_subcommands,
                subcommand_required: #subcommand_required,
                help_flag: #help_flag,
                help_short: #help_short_flag,
                version: #help_version,
                default_subcommand: #help_default,
                default_subcommand_flags: #default_subcommand_flags,
                default_subcommand_help: #default_subcommand_help,
                flatten_help: #flatten_help,
                examples: &[#(#help_examples),*],
                headings: #help_headings,
            };

            fn parse_argv_with(
                __input: &mut ::winnow_args::Argv<'_>,
                __globals: &mut dyn ::winnow_args::Globals,
            ) -> ::core::result::Result<Self, ::winnow_args::Error> {
                use ::winnow_args::__private as __wa;
                #(#slots)*
                #sequence_slot
                #unknown_slot
                #(#flatten_slots)*
                #(#displaced)*
                #position
                #filled_slot
                #help_on_empty
                while !__input.is_empty() {
                    #start
                    #lenient_bundle
                    #save_token
                    #bundle_start
                    let __arg = #lex;
                    match __arg {
                        __wa::Arg::Long(__flag) => match __flag.name {
                            #(#long_arms)*
                            #help_long
                            #version_long
                            _ => #sequence_arg #flatten_arg #late_long if !__globals.bind(&__arg, __input)? { #unknown_flag },
                        },
                        #plus_match
                        __wa::Arg::Short(__flag) => match __flag.letter {
                            #(#short_arms)*
                            #help_short
                            #version_short
                            _ => #sequence_arg #flatten_arg #late_short if !__globals.bind(&__arg, __input)? { #unknown_flag },
                        },
                        #separator_arm
                        #word_arm
                    }
                    #bundle_end
                }
                #on_empty
                #(#env_fallbacks)*
                #exclusive
                #supplied
                #default_ifs
                #(#default_fallbacks)*
                #default_froms
                #required
                let __built = Self {
                    #(#build,)*
                    #(#skipped: ::core::default::Default::default(),)*
                    #sequence_build
                    #unknown_build
                    #(#flatten_build)*
                };
                #(#field_reads)*
                #validate
                ::core::result::Result::Ok(__built)
            }
        }

        #flatten_impl
    })
}

/// A struct-level `group("name", required, multiple)`.
struct Group {
    name: String,
    required: bool,
    multiple: bool,
    /// Said in place of the default message: `{0}` and `{1}` are two members
    /// given, or the group's name and its members when none was.
    message: Option<String>,
}

/// `#[arg(...)]` on the struct: options declared there rather than on a field.
#[derive(Default)]
struct StructOptions {
    groups: Vec<Group>,
    /// The word that starts a new invocation of this command.
    restart_token: Option<String>,
    /// The subcommand a word naming none selects.
    default_subcommand: Option<String>,
    /// A flag this struct does not take selects it too.
    default_subcommand_flags: bool,
    /// Its page follows this struct's.
    default_subcommand_help: bool,
    /// An empty command line runs it.
    default_subcommand_on_empty: bool,
    /// Each subcommand's usage, description and arguments replace the list.
    flatten_help: bool,
    /// The heading of the fields that name none.
    next_help_heading: Option<String>,
    /// What is said under a heading: its title and the paragraph.
    headings: Vec<(String, String)>,
    /// Command lines to try: the line, its header and what it does.
    examples: Vec<[String; 3]>,
    /// What running it does, for the documentation.
    effect: Option<String>,
    /// The forms its output takes, for the documentation.
    outputs: Vec<spec::OutputDoc>,
    /// What each exit status means, for the documentation.
    exit_codes: Vec<(u8, String)>,
    /// A bare invocation asks for help.
    arg_required_else_help: bool,
    /// Called with the built struct, before it is returned.
    validate: Option<syn::Path>,
    /// The program name for the usage line; `argv[0]` when empty.
    name: String,
    /// `-V`/`--version` text: a literal, or `CARGO_PKG_VERSION`.
    version: Option<TokenStream2>,
    /// Help text around the lists; `about`/`long_about` default to the doc comment.
    about: Option<String>,
    long_about: Option<String>,
    after_help: String,
    after_long_help: String,
    disable_help_flag: bool,
    /// Help is `--help` alone, `-h` left undeclared (bash's builtins).
    disable_help_short: bool,
    disable_version_flag: bool,
    disable_help_subcommand: bool,
    /// An unknown flag-like word is a positional value.
    unknown_flags_value: bool,
    /// GNU's `getopt_long_only`: a long name may be spelled with one dash.
    long_only: bool,
    /// `+abc` is a bundle of `+` options (`set +eu`), for fields with `plus`.
    plus_options: bool,
}

fn struct_options(input: &DeriveInput) -> syn::Result<StructOptions> {
    let mut options = StructOptions::default();
    for attr in input.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if skip_spec(&meta)? {
                return Ok(());
            }
            if meta.path.is_ident("restart_token") {
                options.restart_token = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand") {
                options.default_subcommand = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand_flags") {
                options.default_subcommand_flags = true;
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand_on_empty") {
                options.default_subcommand_on_empty = true;
                return Ok(());
            }
            if meta.path.is_ident("default_subcommand_help") {
                options.default_subcommand_help = true;
                return Ok(());
            }
            if meta.path.is_ident("flatten_help") {
                options.flatten_help = true;
                return Ok(());
            }
            if meta.path.is_ident("verbatim_doc_comment") {
                return Ok(());
            }
            if meta.path.is_ident("next_help_heading") {
                options.next_help_heading = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("heading") {
                let (title, [help]) = titled(&meta, ["help"])?;
                options.headings.push((title, help));
                return Ok(());
            }
            if meta.path.is_ident("effect") {
                options.effect = Some(effect_name(&meta)?);
                return Ok(());
            }
            if meta.path.is_ident("output") {
                let content;
                syn::parenthesized!(content in meta.input);
                let mut output = spec::OutputDoc {
                    name: content.parse::<LitStr>()?.value(),
                    default: false,
                    framing: String::new(),
                    help: String::new(),
                };
                while !content.is_empty() {
                    content.parse::<syn::Token![,]>()?;
                    if content.is_empty() {
                        break;
                    }
                    let key = content.parse::<Ident>()?;
                    if key == "default" {
                        output.default = true;
                        continue;
                    }
                    let slot = match key.to_string().as_str() {
                        "framing" => &mut output.framing,
                        "help" => &mut output.help,
                        _ => {
                            return Err(syn::Error::new(
                                key.span(),
                                "expected `default`, `framing` or `help`",
                            ));
                        }
                    };
                    content.parse::<syn::Token![=]>()?;
                    *slot = content.parse::<LitStr>()?.value();
                }
                options.outputs.push(output);
                return Ok(());
            }
            if meta.path.is_ident("exit_code") {
                let content;
                syn::parenthesized!(content in meta.input);
                let code = content.parse::<syn::LitInt>()?.base10_parse::<u8>()?;
                content.parse::<syn::Token![,]>()?;
                let help = content.parse::<LitStr>()?.value();
                options.exit_codes.push((code, help));
                return Ok(());
            }
            if meta.path.is_ident("example") {
                let example = if meta.input.peek(syn::Token![=]) {
                    [meta.value()?.parse::<LitStr>()?.value(), String::new(), String::new()]
                } else {
                    let (code, [header, help]) = titled(&meta, ["header", "help"])?;
                    [code, header, help]
                };
                options.examples.push(example);
                return Ok(());
            }
            if meta.path.is_ident("validate") {
                if options.validate.is_some() {
                    return Err(meta.error("`validate` is given twice"));
                }
                options.validate = Some(meta.value()?.parse::<syn::Path>()?);
                return Ok(());
            }
            if meta.path.is_ident("arg_required_else_help") {
                options.arg_required_else_help = true;
                return Ok(());
            }
            if meta.path.is_ident("long_only") {
                options.long_only = true;
                return Ok(());
            }
            if meta.path.is_ident("plus_options") {
                options.plus_options = true;
                return Ok(());
            }
            if meta.path.is_ident("unknown_flags") {
                let mode = meta.value()?.parse::<LitStr>()?;
                options.unknown_flags_value = match mode.value().as_str() {
                    "value" => true,
                    "error" => false,
                    _ => {
                        return Err(syn::Error::new(
                            mode.span(),
                            "expected \"value\" or \"error\"",
                        ));
                    }
                };
                return Ok(());
            }
            let text = |meta: &syn::meta::ParseNestedMeta<'_>| -> syn::Result<String> {
                Ok(meta.value()?.parse::<LitStr>()?.value())
            };
            if meta.path.is_ident("name") {
                options.name = text(&meta)?;
                return Ok(());
            }
            if meta.path.is_ident("version") {
                options.version = Some(if meta.input.peek(syn::Token![=]) {
                    let version = meta.value()?.parse::<LitStr>()?;
                    quote!(#version)
                } else {
                    quote!(::core::env!("CARGO_PKG_VERSION"))
                });
                return Ok(());
            }
            for (key, slot) in [
                ("about", &mut options.about),
                ("long_about", &mut options.long_about),
            ] {
                if meta.path.is_ident(key) {
                    *slot = Some(text(&meta)?);
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("after_help", &mut options.after_help),
                ("after_long_help", &mut options.after_long_help),
            ] {
                if meta.path.is_ident(key) {
                    *slot = text(&meta)?;
                    return Ok(());
                }
            }
            for (key, slot) in [
                ("disable_help_flag", &mut options.disable_help_flag),
                ("disable_help_short", &mut options.disable_help_short),
                ("disable_version_flag", &mut options.disable_version_flag),
                (
                    "disable_help_subcommand",
                    &mut options.disable_help_subcommand,
                ),
            ] {
                if meta.path.is_ident(key) {
                    *slot = true;
                    return Ok(());
                }
            }
            if !meta.path.is_ident("group") {
                return Err(meta.error(
                    "expected `group(\"name\", ...)`, `restart_token`, `default_subcommand`, \
                     `default_subcommand_flags`, `default_subcommand_help`, `default_subcommand_on_empty`, `arg_required_else_help`, `unknown_flags`, `long_only`, \
                     `plus_options`, `validate`, `name`, `version`, `about`, `long_about`, `verbatim_doc_comment`, `after_help`, `after_long_help`, `example`, \
                     `heading(\"title\", help = ...)`, `next_help_heading`, `flatten_help`, `effect`, `output(\"name\", ...)`, \
                     `exit_code(n, \"meaning\")` or a `disable_*` option",
                ));
            }
            let content;
            syn::parenthesized!(content in meta.input);
            let name = content.parse::<LitStr>()?.value();
            let (mut required, mut multiple, mut message) = (false, false, None);
            while !content.is_empty() {
                content.parse::<syn::Token![,]>()?;
                let flag = content.parse::<Ident>()?;
                match flag.to_string().as_str() {
                    "required" => required = true,
                    "multiple" => multiple = true,
                    "message" => {
                        content.parse::<syn::Token![=]>()?;
                        message = Some(content.parse::<LitStr>()?.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            flag.span(),
                            "expected `required`, `multiple` or `message`",
                        ));
                    }
                }
            }
            options.groups.push(Group {
                name,
                required,
                multiple,
                message,
            });
            Ok(())
        })?;
    }
    Ok(options)
}

/// `effect = "…"`: what a command or a flag does to what it works on.
fn effect_name(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let name = meta.value()?.parse::<LitStr>()?;
    match name.value().as_str() {
        "read" | "write" | "destructive" => Ok(name.value()),
        _ => Err(syn::Error::new(
            name.span(),
            "expected \"read\", \"write\" or \"destructive\"",
        )),
    }
}

/// `key("text", name = "…", …)`: the text, and what each of `names` is
/// given, empty when it is not.
fn titled<const N: usize>(
    meta: &syn::meta::ParseNestedMeta<'_>,
    names: [&str; N],
) -> syn::Result<(String, [String; N])> {
    let content;
    syn::parenthesized!(content in meta.input);
    let text = content.parse::<LitStr>()?.value();
    let mut given = [const { String::new() }; N];
    while !content.is_empty() {
        content.parse::<syn::Token![,]>()?;
        if content.is_empty() {
            break;
        }
        let key = content.parse::<Ident>()?;
        let Some(at) = names.iter().position(|name| key == name) else {
            let names = names.map(|name| format!("`{name}`")).join(" or ");
            return Err(syn::Error::new(key.span(), format!("expected {names}")));
        };
        content.parse::<syn::Token![=]>()?;
        given[at] = content.parse::<LitStr>()?.value();
    }
    Ok((text, given))
}

/// The relations between fields, resolved to field indices.
struct Rules {
    /// Pairs `(a, b)` that may not both be supplied, each unordered pair once,
    /// with the message `a` gives in place of the default one.
    conflicts: Vec<(usize, usize, Option<String>)>,
    /// For each field, the fields it displaces when bound (symmetric).
    beats: Vec<Vec<usize>>,
    /// `(field, target, message)`: `field` supplied needs `target` to have a value.
    requires: Vec<(usize, usize, Option<String>)>,
    /// `(field, others)`: `field` needs a value unless one of `others` has one.
    required_unless: Vec<(usize, Vec<usize>)>,
    /// `(field, others)`: `field` needs a value unless every one of `others` has one.
    required_unless_all: Vec<(usize, Vec<usize>)>,
    /// `(field, other, the value `other` must have)`: `field` needs a value when it holds.
    required_ifs: Vec<(usize, usize, Option<String>)>,
    /// `(field, its value, target)`: `target` needs a value when `field` was given that one.
    requires_ifs: Vec<(usize, String, usize)>,
    /// Fields given only alone.
    exclusives: Vec<usize>,
    /// Members of each struct-level group, in `groups` order.
    members: Vec<Vec<usize>>,
    /// `(field, other, the value `other` must have, the field's default then)`.
    default_ifs: Vec<(usize, usize, Option<String>, String)>,
}

impl Rules {
    fn new(fields: &[Field], groups: &[Group]) -> syn::Result<Self> {
        let resolve = |from: &Field, selector: &str| -> syn::Result<usize> {
            fields
                .iter()
                .position(
                    |f| match (selector.strip_prefix("--"), selector.strip_prefix('-')) {
                        (Some(long), _) => f.longs().contains(&long),
                        (None, Some(short)) => {
                            let mut chars = short.chars();
                            chars
                                .next()
                                .is_some_and(|c| chars.next().is_none() && f.shorts().contains(&c))
                        }
                        (None, None) => {
                            matches!(&f.role, Role::Positional { name, .. } if name == selector)
                                || f.ident == selector
                        }
                    },
                )
                .ok_or_else(|| {
                    // A rule is about a flag having a value, whichever way it was spelled.
                    let negated = selector
                        .strip_prefix("--")
                        .and_then(|long| fields.iter().find(|f| f.negate.as_deref() == Some(long)));
                    let message = match negated {
                        Some(f) => format!(
                            "`{selector}` is the negation of `{}`: name that flag instead",
                            f.display()
                        ),
                        None => format!("`{selector}` names no flag or positional of this struct"),
                    };
                    syn::Error::new(from.ident.span(), message)
                })
        };
        let mut rules = Rules {
            conflicts: Vec::new(),
            beats: vec![Vec::new(); fields.len()],
            requires: Vec::new(),
            required_unless: Vec::new(),
            required_unless_all: Vec::new(),
            required_ifs: Vec::new(),
            requires_ifs: Vec::new(),
            exclusives: Vec::new(),
            members: vec![Vec::new(); groups.len()],
            default_ifs: Vec::new(),
        };
        for (i, f) in fields.iter().enumerate() {
            for (selector, message) in &f.conflicts {
                let j = resolve(f, selector)?;
                let same = |&(a, b, _): &(usize, usize, _)| (a, b) == (i, j) || (a, b) == (j, i);
                match rules.conflicts.iter_mut().find(|pair| same(pair)) {
                    // Declared from both sides: a message says which comes first.
                    Some(pair) if message.is_some() && pair.2.is_none() => {
                        *pair = (i, j, message.clone());
                    }
                    Some(_) => {}
                    None if message.is_some() => rules.conflicts.push((i, j, message.clone())),
                    None => rules.conflicts.push((i.min(j), i.max(j), None)),
                }
            }
            for selector in &f.overrides {
                let j = resolve(f, selector)?;
                if !rules.beats[i].contains(&j) {
                    rules.beats[i].push(j);
                }
                if !rules.beats[j].contains(&i) {
                    rules.beats[j].push(i);
                }
            }
            for (selector, message) in &f.requires {
                rules
                    .requires
                    .push((i, resolve(f, selector)?, message.clone()));
            }
            // A condition on another field: given, or given a value it can compare.
            // `instead` is the spelling without the comparison.
            // A condition, or what a requirement depends on, is another field.
            let other_than_self = |selector: &str| {
                let other = resolve(f, selector)?;
                if other == i {
                    return Err(syn::Error::new(
                        f.ident.span(),
                        format!("`{selector}` is this field: name another one"),
                    ));
                }
                Ok(other)
            };
            let condition = |selector: &str, equals: &Option<String>, instead: &str| {
                let other = other_than_self(selector)?;
                let comparable = match &fields[other].kind {
                    Kind::Count(_) => false,
                    Kind::Switch => equals.as_deref().is_none_or(is_bool_word),
                    _ => !fields[other].keywords,
                };
                if equals.is_some() && !comparable {
                    return Err(syn::Error::new(
                        f.ident.span(),
                        format!("`{selector}` has no value to compare: use `{instead}`"),
                    ));
                }
                Ok(other)
            };
            for (selector, equals, value) in &f.default_ifs {
                let instead = format!("default_if(\"{selector}\", \"…\")");
                let other = condition(selector, equals, &instead)?;
                rules
                    .default_ifs
                    .push((i, other, equals.clone(), value.clone()));
            }
            for (selector, equals) in &f.required_ifs {
                let instead = format!("required_if(\"{selector}\")");
                let other = condition(selector, equals, &instead)?;
                rules.required_ifs.push((i, other, equals.clone()));
            }
            for (equals, selector) in &f.requires_ifs {
                let target = other_than_self(selector)?;
                let comparable = match &f.kind {
                    Kind::Count(_) => false,
                    Kind::Switch => is_bool_word(equals),
                    _ => !f.keywords,
                };
                if !comparable {
                    let message = match &f.kind {
                        Kind::Switch => {
                            format!("a switch's value is `true` or `false`, not `{equals}`")
                        }
                        _ => format!(
                            "this field has no value to compare: use `requires = \"{selector}\"`"
                        ),
                    };
                    return Err(syn::Error::new(f.ident.span(), message));
                }
                rules.requires_ifs.push((i, equals.clone(), target));
            }
            if f.exclusive {
                rules.exclusives.push(i);
            }
            if !f.required_unless_all.is_empty() {
                let others = f
                    .required_unless_all
                    .iter()
                    .map(|selector| other_than_self(selector))
                    .collect::<syn::Result<_>>()?;
                rules.required_unless_all.push((i, others));
            }
            if !f.required_unless.is_empty() {
                let others = f
                    .required_unless
                    .iter()
                    .map(|selector| other_than_self(selector))
                    .collect::<syn::Result<_>>()?;
                rules.required_unless.push((i, others));
            }
            if let Some(group) = &f.group {
                let g = groups
                    .iter()
                    .position(|g| &g.name == group)
                    .ok_or_else(|| {
                        syn::Error::new(
                            f.ident.span(),
                            format!("no `#[arg(group(\"{group}\"))]` on the struct"),
                        )
                    })?;
                rules.members[g].push(i);
            }
        }
        if let Some((g, _)) = groups
            .iter()
            .zip(&rules.members)
            .find(|(g, members)| g.required && members.is_empty())
        {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("group `{}` is required, but no field is in it", g.name),
            ));
        }
        Ok(rules)
    }

    /// Whether an `overrides` can unset `f`, so its fallbacks must check.
    fn is_displaced(&self, fields: &[Field], f: &Field) -> bool {
        !self.beats[index_of(fields, f)].is_empty()
    }

    /// `let mut __displaced_x = false;` for every field an `overrides` can unset.
    fn displaced_slots(&self, fields: &[Field]) -> Vec<TokenStream2> {
        fields
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.beats[*i].is_empty())
            .map(|(_, f)| {
                let displaced = displaced_flag(&f.ident);
                quote!(let mut #displaced = false;)
            })
            .collect()
    }

    /// After binding `f`: unset what it beats, and mark `f` as standing.
    fn displace(&self, fields: &[Field], f: &Field) -> TokenStream2 {
        let i = index_of(fields, f);
        if self.beats[i].is_empty() {
            return quote!();
        }
        let standing = displaced_flag(&f.ident);
        let clears = self.beats[i].iter().map(|&j| {
            let other = &fields[j];
            let slot = slot(&other.ident);
            let displaced = displaced_flag(&other.ident);
            let clear = match &other.kind {
                _ if other.keywords => quote!(#slot.clear();),
                Kind::Switch if other.negate.is_some() => {
                    quote!(#slot = ::core::option::Option::None;)
                }
                Kind::Switch => quote!(#slot = false;),
                Kind::Count(_) => quote!(#slot = 0;),
                Kind::Optional(_) | Kind::Required(_) => {
                    quote!(#slot = ::core::option::Option::None;)
                }
                Kind::Many(_) => quote!(#slot.clear();),
            };
            quote!(#clear #displaced = true;)
        });
        quote!(#standing = false; #(#clears)*)
    }

    /// Conflicts and at-most-one groups, judged on what was supplied.
    fn exclusive(&self, fields: &[Field], groups: &[Group]) -> TokenStream2 {
        let pairs = self.conflicts.iter().map(|(a, b, message)| {
            let (fa, fb) = (&fields[*a], &fields[*b]);
            let (has_a, has_b) = (fa.has(), fb.has());
            let (name_a, name_b) = (fa.display(), fb.display());
            let error = worded(
                quote!(__wa::Error::conflict(__input.offset(), #name_a, #name_b)),
                message.as_deref(),
            );
            quote! {
                if #has_a && #has_b {
                    return ::core::result::Result::Err(#error);
                }
            }
        });
        let at_most_one = groups.iter().zip(&self.members).filter(|(g, _)| !g.multiple).map(|(g, members)| {
            let checks = members.iter().map(|&m| {
                let has = fields[m].has();
                let name = fields[m].display();
                let error = worded(
                    quote!(__wa::Error::conflict(__input.offset(), __first, #name)),
                    g.message.as_deref(),
                );
                quote! {
                    if #has {
                        if let ::core::option::Option::Some(__first) = __first {
                            return ::core::result::Result::Err(#error);
                        }
                        __first = ::core::option::Option::Some(#name);
                    }
                }
            });
            quote! {{
                let mut __first: ::core::option::Option<&'static str> = ::core::option::Option::None;
                #(#checks)*
            }}
        });
        quote!(#(#pairs)* #(#at_most_one)*)
    }

    /// `default_if`: each condition judged on what was supplied, then the first
    /// that holds for a field still without a value gives it one.
    fn default_ifs(&self, fields: &[Field]) -> TokenStream2 {
        let (mut conditions, mut fills) = (Vec::new(), Vec::new());
        // `required_if` is judged on the same snapshot, and checked with the
        // other requirements.
        for (n, (_, other, equals)) in self.required_ifs.iter().enumerate() {
            let holds = format_ident!("__required_if_{n}");
            let condition = condition(&fields[*other], equals.as_deref());
            conditions.push(quote!(let #holds = #condition;));
        }
        for (n, (field, equals, _)) in self.requires_ifs.iter().enumerate() {
            let holds = format_ident!("__requires_if_{n}");
            let condition = condition(&fields[*field], Some(equals));
            conditions.push(quote!(let #holds = #condition;));
        }
        // An `exclusive` field given with any other, before defaults fill them.
        for &i in &self.exclusives {
            let (has, name) = (fields[i].has(), fields[i].display());
            let others = fields
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, other)| {
                    let (other_has, other_name) = (other.has(), other.display());
                    quote! {
                        if #other_has {
                            return ::core::result::Result::Err(
                                __wa::Error::conflict(__input.offset(), #name, #other_name),
                            );
                        }
                    }
                });
            conditions.push(quote!(if #has { #(#others)* }));
        }
        for (n, (field, other, equals, value)) in self.default_ifs.iter().enumerate() {
            let f = &fields[*field];
            let holds = format_ident!("__default_if_{n}");
            let condition = condition(&fields[*other], equals.as_deref());
            conditions.push(quote!(let #holds = #condition;));
            let unset = f.unset(self.is_displaced(fields, f));
            let slot = slot(&f.ident);
            let fill = match &f.kind {
                Kind::Switch => {
                    let value = value == "true";
                    match f.negate {
                        Some(_) => quote!(#slot = ::core::option::Option::Some(#value);),
                        None => quote!(#slot = #value;),
                    }
                }
                _ => {
                    let bytes = LitByteStr::new(value.as_bytes(), Span::call_site());
                    f.assign(quote!(__wa::BStr::new(#bytes)), &f.display())
                }
            };
            fills.push(quote!(if #holds && #unset { #fill }));
        }
        quote!(#(#conditions)* #(#fills)*)
    }

    /// Whether each `requires`-declaring field was supplied, before defaults fill it.
    fn supplied(&self, fields: &[Field]) -> TokenStream2 {
        let mut seen = Vec::new();
        let snapshots = self.requires.iter().filter_map(|&(i, _, _)| {
            if seen.contains(&i) {
                return None;
            }
            seen.push(i);
            let supplied = format_ident!("__supplied_{}", fields[i].ident);
            let has = fields[i].has();
            Some(quote!(let #supplied = #has;))
        });
        quote!(#(#snapshots)*)
    }

    /// `required`, `required_unless`, required groups and `requires`, judged on
    /// what has a value, defaults included.
    fn required(&self, fields: &[Field], groups: &[Group]) -> TokenStream2 {
        let missing = |f: &Field| {
            let name = f.display();
            match f.role {
                Role::Positional { .. } => {
                    quote!(__wa::Error::missing_argument(__input.offset(), #name))
                }
                _ => quote!(__wa::Error::missing_required(__input.offset(), #name)),
            }
        };
        // A flag another one overrode was given its answer: it is not missing.
        let displaced = |f: &Field| {
            self.is_displaced(fields, f).then(|| {
                let flag = displaced_flag(&f.ident);
                quote!(&& !#flag)
            })
        };
        // Every one left without a value is reported together, a field of a
        // type that is not an `Option` included: building it would fail.
        let conditional = |i: usize| {
            let mut unless = self.required_unless.iter().chain(&self.required_unless_all);
            unless.any(|(j, _)| *j == i)
        };
        let built = |i: usize, f: &Field| {
            matches!(f.kind, Kind::Required(_))
                && !f.keywords
                && !matches!(f.role, Role::Subcommand)
                && !conditional(i)
        };
        let plain = fields
            .iter()
            .enumerate()
            .filter(|&(i, f)| f.required || built(i, f));
        let plain = plain.map(|(_, f)| {
            let (has, displaced) = (f.has(), displaced(f));
            let error = missing(f);
            quote!(if !#has #displaced { __wa::missing(&mut __missing, #error); })
        });
        let unless = self.required_unless.iter().map(|(i, others)| {
            let (has, displaced) = (fields[*i].has(), displaced(&fields[*i]));
            let others = others.iter().map(|&j| fields[j].has());
            let error = missing(&fields[*i]);
            quote! {
                if !#has #displaced #(&& !#others)* {
                    __wa::missing(&mut __missing, #error);
                }
            }
        });
        let one_of = groups
            .iter()
            .zip(&self.members)
            .filter(|(g, _)| g.required)
            .map(|(g, members)| {
                let name = &g.name;
                let has = members.iter().map(|&m| fields[m].has());
                let names = members.iter().map(|&m| fields[m].display());
                let error = worded(
                    quote!(__wa::Error::missing_one_of(__input.offset(), #name, &[#(#names),*])),
                    g.message.as_deref(),
                );
                quote! {
                    if #(!#has)&&* {
                        return ::core::result::Result::Err(#error);
                    }
                }
            });
        let requires = self.requires.iter().map(|(i, j, message)| {
            let supplied = format_ident!("__supplied_{}", fields[*i].ident);
            let has = fields[*j].has();
            let (by, target) = (fields[*i].display(), fields[*j].display());
            let error = worded(
                quote!(__wa::Error::required_by(__input.offset(), #target, #by)),
                message.as_deref(),
            );
            quote! {
                if #supplied && !#has {
                    return ::core::result::Result::Err(#error);
                }
            }
        });
        let unless_all = self.required_unless_all.iter().map(|(i, others)| {
            let (has, displaced) = (fields[*i].has(), displaced(&fields[*i]));
            let others = others.iter().map(|&j| fields[j].has());
            let error = missing(&fields[*i]);
            quote! {
                if !#has #displaced && !(#(#others)&&*) {
                    __wa::missing(&mut __missing, #error);
                }
            }
        });
        let missing: Vec<TokenStream2> = plain.chain(unless).chain(unless_all).collect();
        let missing = (!missing.is_empty()).then(|| {
            quote! {
                let mut __missing = ::core::option::Option::None;
                #(#missing)*
                if let ::core::option::Option::Some(__error) = __missing {
                    return ::core::result::Result::Err(__error);
                }
            }
        });
        let required_ifs = self
            .required_ifs
            .iter()
            .enumerate()
            .map(|(n, (i, j, equals))| {
                let holds = format_ident!("__required_if_{n}");
                let (has, displaced) = (fields[*i].has(), displaced(&fields[*i]));
                let target = fields[*i].display();
                let by = match equals {
                    Some(equals) => format!("{}={equals}", fields[*j].display()),
                    None => fields[*j].display(),
                };
                quote! {
                    if #holds && !#has #displaced {
                        return ::core::result::Result::Err(
                            __wa::Error::required_by(__input.offset(), #target, #by),
                        );
                    }
                }
            });
        let requires_ifs = self
            .requires_ifs
            .iter()
            .enumerate()
            .map(|(n, (i, equals, j))| {
                let holds = format_ident!("__requires_if_{n}");
                let has = fields[*j].has();
                let target = fields[*j].display();
                let by = format!("{}={equals}", fields[*i].display());
                quote! {
                    if #holds && !#has {
                        return ::core::result::Result::Err(
                            __wa::Error::required_by(__input.offset(), #target, #by),
                        );
                    }
                }
            });
        quote!(#missing #(#required_ifs)* #(#requires_ifs)* #(#one_of)* #(#requires)*)
    }
}

/// The `default_from` fields, each computed once the fields it is made from
/// have their values, computed ones included: a field comes after those it
/// is made from, and a cycle is a compile error.
fn default_froms(fields: &[Field], rules: &Rules, generic: bool) -> syn::Result<TokenStream2> {
    let index = |f: &Field, name: &Ident| -> syn::Result<usize> {
        let i = fields
            .iter()
            .position(|other| other.ident == *name)
            .ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    format!("`{name}` is not a field of this struct"),
                )
            })?;
        let other = &fields[i];
        if other.ident == f.ident {
            return Err(syn::Error::new(
                name.span(),
                "a default cannot be made from its own field",
            ));
        }
        if other.keywords || matches!(other.role, Role::Subcommand) {
            return Err(syn::Error::new(
                name.span(),
                format!("`{name}` is a subcommand or `keywords`: a default is made from values"),
            ));
        }
        Ok(i)
    };
    let mut order: Vec<usize> = Vec::new();
    let mut visiting: Vec<usize> = Vec::new();
    fn visit(
        i: usize,
        fields: &[Field],
        index: &dyn Fn(&Field, &Ident) -> syn::Result<usize>,
        order: &mut Vec<usize>,
        visiting: &mut Vec<usize>,
    ) -> syn::Result<()> {
        let Some((names, _)) = &fields[i].default_from else {
            return Ok(());
        };
        if order.contains(&i) {
            return Ok(());
        }
        if visiting.contains(&i) {
            return Err(syn::Error::new(
                fields[i].ident.span(),
                "these `default_from` fields are made from each other",
            ));
        }
        visiting.push(i);
        for name in names {
            visit(index(&fields[i], name)?, fields, index, order, visiting)?;
        }
        visiting.pop();
        order.push(i);
        Ok(())
    }
    for i in 0..fields.len() {
        visit(i, fields, &index, &mut order, &mut visiting)?;
    }
    let mut out = Vec::new();
    for i in order {
        let f = &fields[i];
        let (names, function) = f.default_from.as_ref().expect("in the order only with one");
        let target = slot(&f.ident);
        let args: Vec<Ident> = (0..names.len())
            .map(|n| format_ident!("__from_{n}"))
            .collect();
        // Called through a function that states the types, so a closure needs
        // none of its own: `|input| input.with_extension("out")`. A generic
        // struct's types cannot be named there, and its closures state theirs.
        let value = if generic {
            quote!((#function)(#(#args),*))
        } else {
            let mut taken = Vec::new();
            for name in names {
                taken.push(fields[index(f, name)?].given_type());
            }
            let made = f.made_type();
            quote!({
                fn __typed<__F: ::core::ops::FnOnce(#(&#taken),*) -> #made>(f: __F) -> __F {
                    f
                }
                __typed(#function)(#(#args),*)
            })
        };
        let mut body = match &f.kind {
            Kind::Optional(_) | Kind::Required(_) => {
                quote!(#target = ::core::option::Option::Some(#value);)
            }
            Kind::Switch if f.negate.is_some() => {
                quote!(#target = ::core::option::Option::Some(#value);)
            }
            _ => quote!(#target = #value;),
        };
        // Each field as declared. A required one still without a value leaves
        // this one unset, and the requirements then report it.
        for (name, arg) in names.iter().zip(&args).rev() {
            let other = &fields[index(f, name)?];
            let source = slot(&other.ident);
            body = match &other.kind {
                Kind::Required(_) => {
                    quote!(if let ::core::option::Option::Some(#arg) = #source.as_ref() { #body })
                }
                Kind::Switch if other.negate.is_some() => {
                    quote!({ let #arg = &#source.unwrap_or(false); #body })
                }
                _ => quote!({ let #arg = &#source; #body }),
            };
        }
        let unset = f.unset(rules.is_displaced(fields, f));
        out.push(quote!(if #unset { #body }));
    }
    Ok(quote!(#(#out)*))
}

/// The fields in the order help lists them: by `display_order` where one is
/// given, else by their place among the fields, counted from 0. A field given
/// place `n` comes before the field that is at `n`.
fn in_help_order(fields: &[Field]) -> Vec<&Field> {
    let mut ordered: Vec<((usize, bool), &Field)> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| ((f.display_order.unwrap_or(i), f.display_order.is_none()), f))
        .collect();
    ordered.sort_by_key(|&(key, _)| key);
    ordered.into_iter().map(|(_, f)| f).collect()
}

/// Whether `other` has a value, or `equals` when it is given: the condition
/// of a `default_if` or a `required_if`, judged before the defaults.
fn condition(other: &Field, equals: Option<&str>) -> TokenStream2 {
    let other_slot = slot(&other.ident);
    let is = |ty: &Type, equals: &str| {
        let literal = LitByteStr::new(equals.as_bytes(), Span::call_site());
        let (ty, unwrap) = other.parsed(ty);
        quote! {
            |__v| <#ty as ::winnow_args::FromArg>::from_arg(__wa::BStr::new(#literal))
                .is_ok_and(|__literal| __literal #unwrap == *__v)
        }
    };
    match (equals, &other.kind) {
        (None, _) => other.has(),
        (Some(equals), Kind::Switch) => {
            let equals = equals == "true";
            match other.negate {
                Some(_) => quote!((#other_slot == ::core::option::Option::Some(#equals))),
                None => quote!((#other_slot == #equals)),
            }
        }
        (Some(equals), Kind::Optional(ty) | Kind::Required(ty)) => {
            let is = is(ty, equals);
            quote!(#other_slot.as_ref().is_some_and(#is))
        }
        (Some(equals), Kind::Many(ty)) => {
            let is = is(ty, equals);
            quote!(#other_slot.iter().any(#is))
        }
        (Some(_), Kind::Count(_)) => unreachable!("refused when the rules were read"),
    }
}

/// A `Vec<T>` field's name and `T`; `shape` is the error for any other type.
fn vec_field(f: Option<&syn::Field>, shape: &str) -> syn::Result<Option<(Ident, Type)>> {
    let Some(f) = f else {
        return Ok(None);
    };
    let ty = last_segment(&f.ty)
        .filter(|s| s.ident == "Vec")
        .and_then(inner)
        .ok_or_else(|| syn::Error::new(f.ty.span(), shape))?;
    Ok(Some((f.ident.clone().expect("named field"), ty.clone())))
}

/// The enum of a subcommand field.
fn subcommand_type(f: &Field) -> &Type {
    match &f.kind {
        Kind::Optional(ty) | Kind::Required(ty) => ty,
        _ => unreachable!("rejected for subcommands in `field`"),
    }
}

fn displaced_flag(ident: &Ident) -> Ident {
    format_ident!("__displaced_{}", ident)
}

fn index_of(fields: &[Field], f: &Field) -> usize {
    fields
        .iter()
        .position(|g| g.ident == f.ident)
        .expect("a field of this struct")
}

/// The variable a field's value is collected in. A leading `_` of the field's name is
/// spelled `u`, after a `slotu` that no other field's variable starts with, so the name
/// stays snake case (`_args` gives `__slotu_args`, not `__slot__args`).
fn slot(ident: &Ident) -> Ident {
    let name = ident.to_string();
    let name = name.trim_start_matches("r#");
    let bare = name.trim_start_matches('_');
    let underscores = name.len() - bare.len();
    if underscores == 0 {
        format_ident!("__slot_{}", bare)
    } else {
        format_ident!("__slot{}_{}", "u".repeat(underscores), bare)
    }
}

fn field(f: &syn::Field) -> syn::Result<Field> {
    let ident = f.ident.clone().expect("named field");
    let bare = ident.to_string().trim_start_matches("r#").to_owned();

    let mut short = None;
    let mut long = None;
    let mut positional = false;
    let mut count = false;
    let mut subcommand = false;
    let mut global = false;
    let mut alias = Vec::new();
    let mut delimiter = None;
    let mut choices = None;
    let mut env = None;
    let mut default = None;
    let mut double_dash = None;
    let mut default_missing = None;
    let mut negative_numbers = false;
    let mut hyphen_values = false;
    let mut require_equals = false;
    let mut keep_equals = false;
    let (mut two_dashes, mut prefix) = (false, false);
    let mut values = 1;
    let (mut values_max, mut value_terminator) = (None, None);
    let (mut default_fn, mut default_note, mut default_ifs) = (None, None, Vec::new());
    let mut default_from = None;
    let mut stop_flags = false;
    let mut plus: Option<char> = None;
    let mut keywords = false;
    let mut short_aliases = Vec::new();
    let mut negate: Option<Option<String>> = None;
    let (doc_help, doc_long_help) = docs(&f.attrs);
    let (mut help, mut long_help, mut heading, mut hide) = (None, None, None, false);
    let mut warning = String::new();
    let (mut hide_default_value, mut hide_env, mut display_order) = (false, false, None);
    let mut from_str = false;
    let mut effect = None;
    let (mut conflicts, mut overrides, mut requires) = (Vec::new(), Vec::new(), Vec::new());
    let (mut required, mut required_unless, mut group) = (false, Vec::new(), None);
    let (mut required_unless_all, mut required_ifs) = (Vec::new(), Vec::new());
    let (mut requires_ifs, mut exclusive) = (Vec::new(), false);
    let mut value_name = None;
    let (mut hint, mut complete) = (None, None);
    for attr in f.attrs.iter().filter(|a| is_ours(a)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("hint") {
                let name = meta.value()?.parse::<LitStr>()?;
                let variant = match name.value().as_str() {
                    "file" => "Files",
                    "dir" => "Dirs",
                    "command" => "Commands",
                    "none" => "Nothing",
                    _ => {
                        return Err(syn::Error::new(
                            name.span(),
                            "a hint is \"file\", \"dir\", \"command\" or \"none\"",
                        ));
                    }
                };
                hint = Some(syn::Ident::new(variant, name.span()));
            } else if meta.path.is_ident("complete") {
                complete = Some(meta.value()?.parse::<syn::Path>()?);
            } else if meta.path.is_ident("short") {
                let c = if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitChar>()?.value()
                } else {
                    bare.chars()
                        .next()
                        .ok_or_else(|| meta.error("cannot infer a short name"))?
                };
                // A second `short` is another letter, as `long` is another name.
                match short {
                    None => short = Some(c),
                    Some(_) => short_aliases.push(c),
                }
            } else if meta.path.is_ident("long") {
                let name = if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<LitStr>()?.value()
                } else {
                    bare.replace('_', "-")
                };
                // A second `long` is another spelling.
                match long {
                    None => long = Some(name),
                    Some(_) => alias.push(name),
                }
            } else if meta.path.is_ident("positional") {
                positional = true;
            } else if meta.path.is_ident("count") {
                count = true;
            } else if meta.path.is_ident("subcommand") {
                subcommand = true;
            } else if meta.path.is_ident("global") {
                global = true;
            } else if meta.path.is_ident("alias") {
                alias.extend(aliases(&meta)?);
            } else if meta.path.is_ident("help") {
                help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("long_help") {
                long_help = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("help_heading") {
                heading = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("hide") {
                hide = true;
            } else if meta.path.is_ident("from_str") {
                from_str = true;
            } else if meta.path.is_ident("hide_default_value") {
                hide_default_value = true;
            } else if meta.path.is_ident("hide_env") {
                hide_env = true;
            } else if meta.path.is_ident("display_order") {
                display_order = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse::<usize>()?);
            } else if meta.path.is_ident("warning") {
                warning = meta.value()?.parse::<LitStr>()?.value();
            } else if meta.path.is_ident("effect") {
                effect = Some(effect_name(&meta)?);
            } else if meta.path.is_ident("negate") {
                negate = Some(if meta.input.peek(syn::Token![=]) {
                    let name = meta.value()?.parse::<LitStr>()?.value();
                    // A leading `--` is not part of the name.
                    Some(name.strip_prefix("--").unwrap_or(&name).to_owned())
                } else {
                    None
                });
            } else if meta.path.is_ident("keywords") {
                keywords = true;
            } else if skip_spec(&meta)? || meta.path.is_ident("verbatim_doc_comment") {
            } else if meta.path.is_ident("plus") {
                plus = Some(meta.value()?.parse::<LitChar>()?.value());
            } else if meta.path.is_ident("stop_flags") {
                stop_flags = true;
            } else if meta.path.is_ident("values") {
                let count = meta.value()?.parse::<syn::Expr>()?;
                (values, values_max) = value_count(&count)?;
            } else if meta.path.is_ident("value_terminator") {
                value_terminator = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("default_from") {
                let content;
                syn::parenthesized!(content in meta.input);
                let mut list: Vec<syn::Expr> =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                let (Some(function), false) = (list.pop(), list.is_empty()) else {
                    return Err(meta.error(
                        "expected `default_from(field, …, function)`: the fields it is made from, then a function or a closure",
                    ));
                };
                let names = list
                    .iter()
                    .map(|name| match name {
                        syn::Expr::Path(path) if path.path.get_ident().is_some() => {
                            Ok(path.path.get_ident().expect("checked").clone())
                        }
                        other => Err(syn::Error::new(other.span(), "expected the name of a field")),
                    })
                    .collect::<syn::Result<Vec<_>>>()?;
                default_from = Some((names, function));
            } else if meta.path.is_ident("default_fn") {
                default_fn = Some(meta.value()?.parse::<syn::ExprPath>()?);
            } else if meta.path.is_ident("default_note") {
                default_note = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("default_if") {
                let content;
                syn::parenthesized!(content in meta.input);
                let list =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
                let mut words = list.iter().map(LitStr::value);
                default_ifs.push(match (words.next(), words.next(), words.next(), words.next()) {
                    (Some(selector), Some(value), None, _) => (selector, None, value),
                    (Some(selector), Some(equals), Some(value), None) => {
                        (selector, Some(equals), value)
                    }
                    _ => {
                        return Err(meta.error(
                            "expected `default_if(\"--flag\", \"value\")` or \
                             `default_if(\"--flag\", \"its value\", \"value\")`",
                        ));
                    }
                });
            } else if meta.path.is_ident("two_dashes") {
                two_dashes = true;
            } else if meta.path.is_ident("prefix") {
                prefix = true;
            } else if meta.path.is_ident("require_equals") {
                require_equals = true;
            } else if meta.path.is_ident("keep_equals") {
                keep_equals = true;
            } else if meta.path.is_ident("allow_hyphen_values") {
                hyphen_values = true;
            } else if meta.path.is_ident("allow_negative_numbers") {
                negative_numbers = true;
            } else if meta.path.is_ident("default_missing") {
                default_missing = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("double_dash") {
                let mode = meta.value()?.parse::<LitStr>()?;
                double_dash = Some(match mode.value().as_str() {
                    "required" => DoubleDash::Required,
                    "automatic" => DoubleDash::Automatic,
                    "optional" => DoubleDash::Optional,
                    "preserve" => DoubleDash::Preserve,
                    _ => {
                        return Err(syn::Error::new(
                            mode.span(),
                            "expected \"required\", \"automatic\", \"optional\" or \"preserve\"",
                        ));
                    }
                });
            } else if meta.path.is_ident("conflicts") {
                conflicts.extend(related(&meta)?);
            } else if meta.path.is_ident("overrides") {
                overrides.extend(aliases(&meta)?);
            } else if meta.path.is_ident("requires") {
                requires.extend(related(&meta)?);
            } else if meta.path.is_ident("required_unless")
                || meta.path.is_ident("required_unless_all")
            {
                let others = aliases(&meta)?;
                if others.is_empty() {
                    return Err(meta.error("name the fields it depends on"));
                }
                if meta.path.is_ident("required_unless") {
                    required_unless.extend(others);
                } else {
                    required_unless_all.extend(others);
                }
            } else if meta.path.is_ident("required_if") {
                let content;
                syn::parenthesized!(content in meta.input);
                let list =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
                let mut words = list.iter().map(LitStr::value);
                required_ifs.push(match (words.next(), words.next(), words.next()) {
                    (Some(selector), equals, None) => (selector, equals),
                    _ => {
                        return Err(meta.error(
                            "expected `required_if(\"--flag\")` or `required_if(\"--flag\", \"its value\")`",
                        ));
                    }
                });
            } else if meta.path.is_ident("requires_if") {
                let content;
                syn::parenthesized!(content in meta.input);
                let list =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
                let mut words = list.iter().map(LitStr::value);
                requires_ifs.push(match (words.next(), words.next(), words.next()) {
                    (Some(equals), Some(selector), None) => (equals, selector),
                    _ => {
                        return Err(meta.error(
                            "expected `requires_if(\"this field's value\", \"--flag\")`",
                        ));
                    }
                });
            } else if meta.path.is_ident("exclusive") {
                exclusive = true;
            } else if meta.path.is_ident("required") {
                required = true;
            } else if meta.path.is_ident("group") {
                group = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("env") {
                env = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("default") {
                default = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("choices") {
                let content;
                syn::parenthesized!(content in meta.input);
                let list =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(&content)?;
                choices = Some(list.iter().map(LitStr::value).collect());
            } else if meta.path.is_ident("delimiter") {
                let c = meta.value()?.parse::<LitChar>()?;
                if !c.value().is_ascii() {
                    return Err(syn::Error::new(c.span(), "a delimiter is one ASCII character"));
                }
                delimiter = Some(c.value() as u8);
            } else if meta.path.is_ident("value_name") {
                value_name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error(
                    "unknown `arg` option; expected one of `short`, `long`, `alias`, `global`, `count`, \
                     `positional`, `value_name`, `double_dash`, `subcommand`, `flatten`, `sequence`, `unknown`, `skip`, \
                     `delimiter`, `choices`, `env`, `default`, `default_missing`, `allow_negative_numbers`, \
                     `allow_hyphen_values`, `require_equals`, `keep_equals`, `negate`, `two_dashes`, `prefix`, `values`, \
                     `value_terminator`, `default_fn`, `default_from`, `default_note`, `default_if`, `from_str`, \
                     `stop_flags`, `plus`, `keywords`, `conflicts`, `overrides`, `requires`, `required`, `required_unless`, `required_unless_all`, `required_if`, `requires_if`, `exclusive`, \
                     `group`, `help`, `long_help`, `verbatim_doc_comment`, `help_heading`, `warning`, `effect`, `hide`, `hide_default_value`, `hide_env`, `display_order`, `hint`, `complete`",
                ));
            }
            Ok(())
        })?;
    }

    let error = |msg: String| Err(syn::Error::new(f.span(), msg));
    if delimiter.is_some() && (count || !matches!(kind(&f.ty), Kind::Many(_))) {
        return error("`delimiter` needs a `Vec<T>` field to put the pieces in".into());
    }
    let kind = if count {
        Kind::Count(f.ty.clone())
    } else {
        kind(&f.ty)
    };
    if default_missing.is_some()
        && (positional || subcommand || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error("`default_missing` is for flags that take a value".into());
    }
    if (hyphen_values || require_equals)
        && (positional || subcommand || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error(
            "`allow_hyphen_values` and `require_equals` are for flags that take a value".into(),
        );
    }
    if negative_numbers && (subcommand || matches!(kind, Kind::Switch | Kind::Count(_))) {
        return error("`allow_negative_numbers` is for fields that take a value".into());
    }
    let tristate =
        plus.is_some() && plus == short && matches!(&kind, Kind::Optional(ty) if is_bool(ty));
    if plus.is_some()
        && (positional || subcommand || !tristate && matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error(
            "`plus` is for a flag taking a value, or an `Option<bool>` with the same `short` letter".into(),
        );
    }
    if keywords
        && (positional
            || subcommand
            || !matches!(kind, Kind::Required(_))
            || env.is_some()
            || default.is_some()
            || default_missing.is_some()
            || choices.is_some()
            || delimiter.is_some())
    {
        return error(
            "`keywords` is for a flag whose type derives `Args`, without `env`, `default`, \
             `default_missing`, `choices` or `delimiter`"
                .into(),
        );
    }
    if stop_flags && !positional {
        return error("`stop_flags` is for positional fields".into());
    }
    if double_dash.is_some() && !positional {
        return error("`double_dash` is for positional fields".into());
    }
    if (values > 1 || values_max.is_some())
        && (positional
            || subcommand
            || !matches!(kind, Kind::Many(_))
            || delimiter.is_some()
            || default_missing.is_some())
    {
        return error(
            "`values` is for a `Vec<T>` flag, without `delimiter` or `default_missing`".into(),
        );
    }
    if values_max.is_some() && require_equals {
        return error("a range of `values` takes detached words: not with `require_equals`".into());
    }
    if value_terminator.is_some() && values_max.is_none() {
        return error("`value_terminator` ends a range of `values` (`values = 1..`)".into());
    }
    let valueless = matches!(kind, Kind::Switch | Kind::Count(_)) && negate.is_none();
    if default_fn.is_some() && (default.is_some() || subcommand || keywords || valueless) {
        return error(
            "`default_fn` is for a field that takes a value, or `negate`, without `default`".into(),
        );
    }
    if default_from.is_some()
        && (default.is_some() || default_fn.is_some() || subcommand || keywords || tristate)
    {
        return error(
            "`default_from` is for a field that takes a value, a switch or a count, without `default` or `default_fn`"
                .into(),
        );
    }
    if display_order.is_some() && positional {
        return error(
            "`display_order` is for a flag: positionals are listed in the order they are taken"
                .into(),
        );
    }
    if default_note.is_some() && default_fn.is_none() && default_from.is_none() {
        return error("`default_note` describes a `default_fn` or a `default_from`".into());
    }
    if !default_ifs.is_empty() && (subcommand || keywords || matches!(kind, Kind::Count(_))) {
        return error("`default_if` is for a switch or a field that takes a value".into());
    }
    if let (Kind::Switch, Some((.., value))) =
        (&kind, default_ifs.iter().find(|d| !is_bool_word(&d.2)))
    {
        return error(format!(
            "a switch's `default_if` value is `true` or `false`, not `{value}`"
        ));
    }
    if two_dashes && (long.is_none() && alias.is_empty() || positional || subcommand) {
        return error("`two_dashes` is for flags with a long name".into());
    }
    if prefix
        && (short.is_none_or(|c| !c.is_ascii())
            || positional
            || subcommand
            || matches!(kind, Kind::Switch | Kind::Count(_)))
    {
        return error("`prefix` is for a value-taking flag with an ASCII `short` letter".into());
    }
    let flag_value_name =
        (!positional && !subcommand && !matches!(kind, Kind::Switch | Kind::Count(_)))
            .then(|| value_name.clone().unwrap_or_else(|| bare.to_uppercase()));
    let role = if subcommand {
        if positional || count || global || short.is_some() || long.is_some() || !alias.is_empty() {
            return error("a subcommand field takes no other `arg` options".into());
        }
        if !matches!(kind, Kind::Optional(_) | Kind::Required(_)) {
            return error("a subcommand field is `E` or `Option<E>`".into());
        }
        Role::Subcommand
    } else if positional {
        if short.is_some() || long.is_some() || global || !alias.is_empty() {
            return error("a positional field has no `short`, `long`, `alias` or `global`".into());
        }
        if matches!(kind, Kind::Switch | Kind::Count(_)) {
            return error("a positional field cannot be `bool` or `count`".into());
        }
        Role::Positional {
            double_dash: double_dash.unwrap_or(DoubleDash::Optional),
            name: value_name.unwrap_or_else(|| bare.to_uppercase()),
        }
    } else {
        // Like bpaf: a flag with no names is `--field-name` (unless it is
        // spelled only `+c`).
        if short.is_none() && long.is_none() && plus.is_none() {
            long = Some(bare.replace('_', "-"));
        }
        for l in long.iter().chain(&alias) {
            if l.is_empty() || l.starts_with('-') || l.contains('=') {
                return error(format!("`{l}` is not a usable long name"));
            }
        }
        if let Some(c) = short
            && (c == '-' || c == '=')
        {
            return error(format!("`{c}` is not a usable short name"));
        }
        Role::Flag {
            short,
            long,
            aliases: alias,
            global,
        }
    };

    if choices.is_some()
        && (matches!(kind, Kind::Switch | Kind::Count(_)) || matches!(role, Role::Subcommand))
    {
        return error("`choices` needs a field that takes a value".into());
    }
    if (env.is_some() || default.is_some()) && matches!(role, Role::Subcommand) {
        return error("a subcommand field takes no `env` or `default`".into());
    }
    let negate = match negate {
        None => None,
        Some(_) if !matches!(kind, Kind::Switch) || !matches!(role, Role::Flag { .. }) => {
            return error("`negate` is for `bool` flags".into());
        }
        Some(Some(name)) => Some(name),
        Some(None) => match &role {
            Role::Flag { long: Some(l), .. } => Some(format!("no-{l}")),
            _ => return error("`negate` without a name needs a `long` to prefix".into()),
        },
    };
    if let Some(no) = &negate
        && (no.is_empty() || no.starts_with('-') || no.contains('='))
    {
        return error(format!("`{no}` is not a usable long name"));
    }
    match (&default, &negate) {
        (Some(d), Some(_)) if d != "true" && d != "false" => {
            return error("a negatable flag's `default` is \"true\" or \"false\"".into());
        }
        (Some(_), None) if matches!(kind, Kind::Switch | Kind::Count(_)) => {
            return error("`default` needs a field that takes a value, or `negate`".into());
        }
        _ => {}
    }
    if (hint.is_some() || complete.is_some())
        && (matches!(kind, Kind::Switch | Kind::Count(_))
            || !matches!(role, Role::Flag { .. } | Role::Positional { .. }))
    {
        return error("`hint` and `complete` are for a field that takes a value".into());
    }
    if hint.is_some() && complete.is_some() {
        return error("`complete` answers alone: `hint` beside it would never be asked".into());
    }
    let spec = spec_name(&f.attrs)?;
    if spec.is_some() && matches!(kind, Kind::Switch | Kind::Count(_)) {
        return error("`spec` names the type of a field's value, and this field has none".into());
    }
    Ok(Field {
        ident,
        spec,
        kind,
        role,
        delimiter,
        choices,
        env,
        default,
        conflicts,
        overrides,
        requires,
        required,
        required_unless,
        required_unless_all,
        required_ifs,
        requires_ifs,
        exclusive,
        group,
        default_missing,
        long_help: long_help.or_else(|| help.clone()).unwrap_or(doc_long_help),
        help: help.unwrap_or(doc_help),
        heading,
        hide,
        value_name: if tristate { None } else { flag_value_name },
        hint,
        complete,
        warning,
        hide_default_value,
        hide_env,
        display_order,
        from_str,
        effect,
        negative_numbers,
        hyphen_values,
        require_equals,
        keep_equals,
        negate,
        two_dashes,
        prefix,
        values,
        values_max,
        value_terminator,
        default_fn,
        default_from,
        default_note,
        default_ifs,
        stop_flags,
        plus,
        tristate,
        keywords,
        short_aliases,
    })
}

fn is_bool_word(word: &str) -> bool {
    matches!(word, "true" | "false")
}

/// `values = 3`, `values = 1..`, `values = 1..=3`: the least, and for a range
/// the most (`usize::MAX` when open).
fn value_count(count: &syn::Expr) -> syn::Result<(usize, Option<usize>)> {
    let number = |e: &syn::Expr| match e {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(n),
            ..
        }) => n.base10_parse::<usize>(),
        _ => Err(syn::Error::new(e.span(), "expected a number")),
    };
    let (min, max) = match count {
        syn::Expr::Range(range) => {
            let min = range.start.as_deref().map(number).transpose()?.unwrap_or(1);
            let max = match (&range.end, &range.limits) {
                (None, _) => usize::MAX,
                (Some(end), syn::RangeLimits::Closed(_)) => number(end)?,
                (Some(end), syn::RangeLimits::HalfOpen(_)) => number(end)?.saturating_sub(1),
            };
            (min, Some(max))
        }
        exact => (number(exact)?, None),
    };
    if min == 0 || max.is_some_and(|max| max < min) {
        return Err(syn::Error::new(
            count.span(),
            "`values` is at least 1, and a range does not end below its start",
        ));
    }
    Ok((min, max))
}

/// Every long spelling of `f`, its negation included.
fn spellings(f: &Field) -> Vec<&str> {
    let mut longs = f.longs();
    longs.extend(f.negate.as_deref());
    longs
}

fn check_duplicates(fields: &[Field]) -> syn::Result<()> {
    for (i, a) in fields.iter().enumerate() {
        for b in &fields[..i] {
            let theirs = b.shorts();
            let clash = a
                .shorts()
                .into_iter()
                .find(|c| theirs.contains(c))
                .map(|c| format!("-{c}"))
                .or_else(|| {
                    let longs = spellings(b);
                    spellings(a)
                        .into_iter()
                        .find(|l| longs.contains(l))
                        .map(|l| format!("--{l}"))
                });
            if let Some(name) = clash {
                return Err(syn::Error::new(
                    a.ident.span(),
                    format!("`{name}` is already used by `{}`", b.ident),
                ));
            }
        }
    }
    Ok(())
}

/// Required positionals come first, then optional ones, then at most one `Vec`,
/// last: otherwise which word fills which field would depend on lookahead.
fn check_positional_order(positionals: &[&Field]) -> syn::Result<()> {
    let rank = |f: &Field| match f.kind {
        Kind::Required(_) => 0,
        Kind::Optional(_) => 1,
        Kind::Many(_) | Kind::Switch | Kind::Count(_) => 2,
    };
    for pair in positionals.windows(2) {
        if rank(pair[1]) < rank(pair[0]) || rank(pair[0]) == 2 {
            return Err(syn::Error::new(
                pair[1].ident.span(),
                "positionals must be required, then optional, then one `Vec` last",
            ));
        }
    }
    Ok(())
}

fn last_segment(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last(),
        _ => None,
    }
}

fn inner(segment: &syn::PathSegment) -> Option<&Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => match args.args.first()? {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        },
        _ => None,
    }
}

fn kind(ty: &Type) -> Kind {
    if is_bool(ty) {
        return Kind::Switch;
    }
    if let Some(last) = last_segment(ty)
        && let Some(inner) = inner(last)
    {
        if last.ident == "Option" {
            return Kind::Optional(inner.clone());
        }
        if last.ident == "Vec" {
            return Kind::Many(inner.clone());
        }
    }
    Kind::Required(ty.clone())
}
