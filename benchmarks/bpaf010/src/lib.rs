//! The bench's CLIs in bpaf 0.10, which is not released: outside the
//! workspace, so that it going out of sync breaks nothing, and built only by
//! `BPAF010=1 just perf`.

/// mise, as usage's bpaf shadow declares it: the source compiles with 0.9 and 0.10.
#[cfg(not(doctest))]
#[path = "../../shadows/mise-bpaf/src/shadow.rs"]
pub mod mise;

use std::path::PathBuf;

use bpaf::Bpaf;

/// bpaf has no declarative constraints: a guard over the parsed struct.
pub fn cli_p() -> bpaf::OptionParser<Cli> {
    use bpaf::Parser as _;
    cli_inner()
        .guard(
            |c| !(c.quiet && c.verbose > 0),
            "--quiet cannot be used with --verbose",
        )
        .guard(|c| !(c.json && c.toml), "--json cannot be used with --toml")
        .guard(|c| !c.strict || c.json, "--strict requires --json")
        .to_options()
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(cli_inner))]
pub struct Cli {
    #[bpaf(external(verbose_p))]
    pub verbose: usize,
    #[bpaf(short('q'), long("quiet"), switch)]
    pub quiet: bool,
    #[bpaf(long("json"), switch)]
    pub json: bool,
    #[bpaf(long("toml"), switch)]
    pub toml: bool,
    #[bpaf(long("strict"), switch)]
    pub strict: bool,
    #[bpaf(external(write_p))]
    pub write: Option<String>,
    #[bpaf(external(offset_p))]
    pub offset: Option<i32>,
    // bpaf 0.10 has no hyphen values, and pairing `literal("--args")` with
    // `any` breaks `optional()`: a plain argument, which refuses `-destroy`.
    // The agreement test skips bpaf on such lines.
    #[bpaf(long("args"), argument("ARGS"))]
    pub args: Option<String>,
    #[bpaf(external(inspect_p))]
    pub inspect: Option<String>,
    #[bpaf(external(cache_p))]
    pub cache: bool,
    #[bpaf(short('p'), long("path"), long("dir"), argument("PATH"))]
    pub path: Option<PathBuf>,
    #[bpaf(external(include_p))]
    pub include: Vec<PathBuf>,
    #[bpaf(long("color"), argument("COLOR"))]
    pub color: Option<Color>,
    #[bpaf(
        short('j'),
        long("jobs"),
        env("EXAMPLE_JOBS"),
        argument("JOBS"),
        fallback(4)
    )]
    pub jobs: u32,
    // bpaf tries items in order: the command must come before the greedy
    // positional, or `use` is taken as a FILE.
    #[bpaf(external(commands_p), optional)]
    pub command: Option<Commands>,
    #[bpaf(positional("FILE"))]
    pub files: Vec<PathBuf>,
    // bpaf 0.10 cannot keep `FILE` before `--`: a strict positional errors on a
    // plain word rather than missing it, so `FILE` also takes the words after
    // `--` and this stays empty. The agreement test skips bpaf on such lines.
    #[bpaf(positional("CMD"), strict, many)]
    pub cmd: Vec<String>,
}

/// bpaf's derive has no `default_missing`; `on_missing_value` is the combinator.
fn write_p() -> impl bpaf::Parser<Output = Option<String>> {
    use bpaf::Parser as _;
    bpaf::short('w')
        .long("write")
        .argument::<String>("PATH")
        .on_missing_value(|| Ok("./bin/mise".into()))
        .optional()
}

/// `adjacent` refuses a detached value: `require_equals`. It reports
/// `--inspect a` as not adjacent rather than missing, so `on_missing_value`
/// never fires; a bare `--inspect` flag is the other branch.
fn inspect_p() -> impl bpaf::Parser<Output = Option<String>> {
    use bpaf::Parser as _;
    let port = bpaf::long("inspect").argument::<String>("PORT").adjacent();
    let bare = bpaf::long("inspect").req_flag(String::from("9229"));
    bpaf::construct!([port, bare]).optional()
}

/// `--cache` / `--no-cache`, the last one given winning.
fn cache_p() -> impl bpaf::Parser<Output = bool> {
    use bpaf::Parser as _;
    let yes = bpaf::long("cache").req_flag(true);
    let no = bpaf::long("no-cache").req_flag(false);
    bpaf::construct!([yes, no]).last().fallback(true)
}

/// bpaf's derive has no negative numbers; `negative_lit` is the combinator.
fn offset_p() -> impl bpaf::Parser<Output = Option<i32>> {
    use bpaf::Parser as _;
    bpaf::long("offset")
        .argument::<i32>("N")
        .negative_lit()
        .optional()
}

/// bpaf has no value enums: a `FromStr` does the matching.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl std::str::FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(format!("expected one of auto, always, never, got {s}")),
        }
    }
}

/// bpaf has no value delimiter: split each occurrence afterwards.
fn include_p() -> impl bpaf::Parser<Output = Vec<PathBuf>> {
    use bpaf::Parser as _;
    bpaf::short('I')
        .long("include")
        .argument::<String>("DIR")
        .many()
        .map(|all| {
            all.iter()
                .flat_map(|v| v.split(','))
                .map(PathBuf::from)
                .collect()
        })
}

/// bpaf's derive has no `global`; the combinator does.
fn verbose_p() -> impl bpaf::Parser<Output = usize> {
    use bpaf::Parser as _;
    bpaf::short('v')
        .long("verbose")
        .req_flag(())
        .count()
        .global()
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(commands_p))]
pub enum Commands {
    #[bpaf(command("use"), short('u'))]
    Use(#[bpaf(external(useargs_p))] UseArgs),
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(useargs_p))]
pub struct UseArgs {
    #[bpaf(short('g'), long("global"), switch)]
    pub global: bool,
    #[bpaf(positional("TOOL"))]
    pub tools: Vec<String>,
}

impl Cli {
    /// Whether caching is on, however this framework holds it.
    pub fn cache(&self) -> bool {
        self.cache
    }
}
