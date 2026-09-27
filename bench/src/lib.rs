//! `example -v/--verbose... -p/--path=PATH --color=WHEN -j/--jobs=N -q/--quiet --json|--toml --strict -w/--write[=PATH] -I/--include=DIR[,DIR]... [FILE]... [-- CMD...] [use -g/--global [TOOL]...]`
//! in six spellings.

/// winnow-args, derived.
pub mod wa_derive {
    use std::path::PathBuf;

    #[derive(winnow_args::Args, Debug)]
    #[arg(group("output"))]
    pub struct Cli {
        #[arg(short, long, count, global)]
        pub verbose: u8,
        #[arg(short, long, conflicts = "--verbose")]
        pub quiet: bool,
        #[arg(long, group = "output")]
        pub json: bool,
        #[arg(long, group = "output")]
        pub toml: bool,
        #[arg(long, requires = "--json")]
        pub strict: bool,
        #[arg(short, long, value_optional, default_missing = "./bin/mise")]
        pub write: Option<String>,
        #[arg(short, long, alias = "dir")]
        pub path: Option<PathBuf>,
        #[arg(short = 'I', long, delimiter = ',')]
        pub include: Vec<PathBuf>,
        #[arg(long)]
        pub color: Option<Color>,
        #[arg(short, long, env = "EXAMPLE_JOBS", default = "4")]
        pub jobs: Option<u32>,
        #[arg(positional, value_name = "FILE")]
        pub files: Vec<PathBuf>,
        #[arg(positional, value_name = "CMD", double_dash = "required")]
        pub cmd: Vec<String>,
        #[arg(subcommand)]
        pub command: Option<Commands>,
    }

    #[derive(winnow_args::ValueEnum, Debug, Clone, Copy)]
    pub enum Color {
        Auto,
        Always,
        Never,
    }

    #[derive(winnow_args::Subcommand, Debug)]
    pub enum Commands {
        #[arg(alias = "u")]
        Use(UseArgs),
    }

    #[derive(winnow_args::Args, Debug)]
    pub struct UseArgs {
        #[arg(short, long)]
        pub global: bool,
        #[arg(positional, value_name = "TOOL")]
        pub tools: Vec<String>,
    }
}

/// winnow-args, combinators: `dispatch!` on the item kind, so words skip the flags.
pub mod wa_comb {
    use std::path::PathBuf;

    use winnow::combinator::{alt, cond, dispatch, fail};
    use winnow::prelude::*;
    use winnow_args::combinator::{Named, args, command, long, positional, short};
    use winnow_args::token::{self, Kind, Word, kind};
    use winnow_args::{Argv, Error};

    #[derive(Debug, Default)]
    pub struct Cli {
        pub verbose: u8,
        pub quiet: bool,
        pub json: bool,
        pub toml: bool,
        pub strict: bool,
        pub write: Option<String>,
        pub path: Option<PathBuf>,
        pub include: Vec<PathBuf>,
        pub color: Option<Color>,
        pub jobs: Option<u32>,
        pub files: Vec<PathBuf>,
        pub cmd: Vec<String>,
        pub command: Option<Commands>,
    }

    pub use super::wa_derive::Color;

    #[derive(Debug)]
    pub enum Commands {
        Use(UseArgs),
    }

    #[derive(Debug, Default)]
    pub struct UseArgs {
        pub global: bool,
        pub tools: Vec<String>,
    }

    const GLOBAL: Named = short('g').long("global");

    /// `use`, also accepting the parent's global flags through `globals`.
    fn use_args<'i, G>(mut globals: G) -> impl Parser<Argv<'i>, UseArgs, Error>
    where
        G: Parser<Argv<'i>, (), Error>,
    {
        move |input: &mut Argv<'i>| {
            let mut u = UseArgs::default();
            let (c, g) = (&mut u, &mut globals);
            args(dispatch! {kind;
                Kind::Long | Kind::Short => alt((
                    GLOBAL.switch().map(|()| c.global = true),
                    g.by_ref(),
                )),
                Kind::Word => positional("TOOL").map(|t| c.tools.push(t)),
                Kind::Separator => fail,
            })
            .parse_next(input)?;
            Ok(u)
        }
    }

    const VERBOSE: Named = short('v').long("verbose");
    const PATH: Named<2> = short('p').longs(["path", "dir"]);
    const INCLUDE: Named = short('I').long("include");
    const COLOR: Named = long("color");
    const JOBS: Named = short('j').long("jobs");

    const QUIET: Named = short('q').long("quiet");
    const JSON: Named = long("json");
    const TOML: Named = long("toml");
    const STRICT: Named = long("strict");
    const WRITE: Named = short('w').long("write");

    /// What the derive does after its loop: `--jobs` fallbacks, then the constraints.
    pub fn finish(cli: &mut Cli, input: &Argv<'_>) -> Result<(), Error> {
        if cli.jobs.is_none() {
            cli.jobs = Some(winnow_args::env::value("EXAMPLE_JOBS")?.unwrap_or(4));
        }
        let at = input.offset();
        if cli.quiet && cli.verbose > 0 {
            return Err(Error::conflict(at, "--quiet", "--verbose"));
        }
        if cli.json && cli.toml {
            return Err(Error::conflict(at, "--json", "--toml"));
        }
        if cli.strict && !cli.json {
            return Err(Error::required_by(at, "--json", "--strict"));
        }
        Ok(())
    }

    pub fn cli(input: &mut Argv<'_>) -> Result<Cli, Error> {
        let mut cli = Cli::default();
        let c = &mut cli;
        args(dispatch! {kind;
            // winnow's `alt` takes at most 9 parsers, so the flags are split in two.
            Kind::Long | Kind::Short => alt((
                alt((
                    VERBOSE.switch().map(|()| c.verbose = c.verbose.saturating_add(1)),
                    PATH.argument_as().map(|p| c.path = Some(p)),
                    INCLUDE
                        .arguments_as(b',')
                        .map(|i: Vec<PathBuf>| c.include.extend(i)),
                    COLOR.argument_as().map(|w| c.color = Some(w)),
                    JOBS.argument_as().map(|j| c.jobs = Some(j)),
                )),
                alt((
                    QUIET.switch().map(|()| c.quiet = true),
                    JSON.switch().map(|()| c.json = true),
                    TOML.switch().map(|()| c.toml = true),
                    STRICT.switch().map(|()| c.strict = true),
                    WRITE.argument_or("./bin/mise").map(|w| c.write = Some(w)),
                )),
            )),
            Kind::Word => alt((
                // `CMD` takes every word after `--`, and only those.
                |input: &mut Argv<'_>| {
                    let word = token::word.verify(|w: &Word<'_>| w.after_separator).parse_next(input)?;
                    c.cmd.push(word.convert("CMD")?);
                    Ok(())
                },
                cond(
                    c.files.is_empty(),
                    command(
                        ["use", "u"],
                        use_args(VERBOSE.switch().map(|()| c.verbose = c.verbose.saturating_add(1))),
                    ),
                )
                .verify_map(|found| found)
                .map(|u| c.command = Some(Commands::Use(u))),
                positional("FILE").map(|f| c.files.push(f)),
            )),
            Kind::Separator => fail,
        })
        .parse_next(input)?;
        finish(&mut cli, input)?;
        Ok(cli)
    }
}

/// winnow-args, combinators: one `dispatch!` matching flag names as patterns,
/// the same shape as the derive's `match`.
pub mod wa_disp {
    use winnow::combinator::{dispatch, fail};
    use winnow::prelude::*;
    use winnow::stream::BStr;
    use winnow_args::combinator::args;
    use winnow_args::token::{Arg, LongFlag, ShortFlag, arg, split};
    use winnow_args::{Argv, Error, Globals, globals};

    pub use super::wa_comb::{Cli, Commands, UseArgs};

    /// `use`; flags it does not know are offered to the parent's `globals`.
    fn use_args<'i>(input: &mut Argv<'i>, globals: &mut dyn Globals) -> Result<UseArgs, Error> {
        let mut u = UseArgs::default();
        let c = &mut u;
        args(dispatch! {arg;
            a @ (Arg::Long(LongFlag { name: b"global", .. }) | Arg::Short(ShortFlag { letter: 'g', .. })) => {
                a.switch().map(|()| c.global = true)
            },
            a @ (Arg::Long(_) | Arg::Short(_)) => |input: &mut Argv<'i>| {
                if globals.bind(&a, input)? { Ok(()) } else { Err(a.unexpected()) }
            },
            Arg::Word(w) => w.value_as("TOOL").map(|t| c.tools.push(t)),
            _ => fail,
        })
        .parse_next(input)?;
        Ok(u)
    }

    pub fn cli<'i>(input: &mut Argv<'i>) -> Result<Cli, Error> {
        let mut cli = Cli::default();
        let c = &mut cli;
        args(dispatch! {arg;
            a @ (Arg::Long(LongFlag { name: b"verbose", .. }) | Arg::Short(ShortFlag { letter: 'v', .. })) => {
                a.switch().map(|()| c.verbose = c.verbose.saturating_add(1))
            },
            a @ (Arg::Long(LongFlag { name: b"path" | b"dir", .. }) | Arg::Short(ShortFlag { letter: 'p', .. })) => {
                a.value_as().map(|p| c.path = Some(p))
            },
            a @ (Arg::Long(LongFlag { name: b"include", .. }) | Arg::Short(ShortFlag { letter: 'I', .. })) => {
                |input: &mut Argv<'i>| {
                    for piece in split(a.read_value(input)?, b',') {
                        c.include.push(a.convert(piece)?);
                    }
                    Ok(())
                }
            },
            a @ Arg::Long(LongFlag { name: b"color", .. }) => a.value_as().map(|w| c.color = Some(w)),
            a @ (Arg::Long(LongFlag { name: b"jobs", .. }) | Arg::Short(ShortFlag { letter: 'j', .. })) => {
                a.value_as().map(|j| c.jobs = Some(j))
            },
            a @ (Arg::Long(LongFlag { name: b"quiet", .. }) | Arg::Short(ShortFlag { letter: 'q', .. })) => {
                a.switch().map(|()| c.quiet = true)
            },
            a @ Arg::Long(LongFlag { name: b"json", .. }) => a.switch().map(|()| c.json = true),
            a @ Arg::Long(LongFlag { name: b"toml", .. }) => a.switch().map(|()| c.toml = true),
            a @ Arg::Long(LongFlag { name: b"strict", .. }) => a.switch().map(|()| c.strict = true),
            a @ (Arg::Long(LongFlag { name: b"write", .. }) | Arg::Short(ShortFlag { letter: 'w', .. })) => {
                a.value_or(BStr::new("./bin/mise")).map(|w| c.write = Some(w))
            },
            Arg::Word(w) if w.after_separator => {
                |_: &mut Argv<'i>| {
                    c.cmd.push(w.convert("CMD")?);
                    Ok(())
                }
            },
            Arg::Word(w) if c.files.is_empty() && !w.after_separator && (*w.value == "use" || *w.value == "u") => {
                |input: &mut Argv<'i>| {
                    let verbose = &mut c.verbose;
                    let mut inherit = globals(|a, _| match a {
                        Arg::Long(LongFlag { name: b"verbose", .. }) | Arg::Short(ShortFlag { letter: 'v', .. }) => {
                            a.check_switch()?;
                            *verbose = verbose.saturating_add(1);
                            Ok(true)
                        }
                        _ => Ok(false),
                    });
                    let u = use_args(input, &mut inherit)?;
                    c.command = Some(Commands::Use(u));
                    Ok(())
                }
            },
            Arg::Word(w) => w.value_as("FILE").map(|f| c.files.push(f)),
            _ => fail,
        })
        .parse_next(input)?;
        super::wa_comb::finish(&mut cli, input)?;
        Ok(cli)
    }
}

/// bpaf 0.10, derived.
pub mod bpaf010 {
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
}

/// clap 4, derived.
pub mod clap4 {
    use std::path::PathBuf;

    #[derive(clap::Parser, Debug)]
    #[command(group(clap::ArgGroup::new("output").multiple(false)))]
    pub struct Cli {
        #[arg(short, long, action = clap::ArgAction::Count, global = true)]
        pub verbose: u8,
        #[arg(short, long, conflicts_with = "verbose")]
        pub quiet: bool,
        #[arg(long, group = "output")]
        pub json: bool,
        #[arg(long, group = "output")]
        pub toml: bool,
        #[arg(long, requires = "json")]
        pub strict: bool,
        #[arg(short, long, num_args = 0..=1, default_missing_value = "./bin/mise")]
        pub write: Option<String>,
        #[arg(short, long, alias = "dir")]
        pub path: Option<PathBuf>,
        #[arg(short = 'I', long, value_name = "DIR", value_delimiter = ',')]
        pub include: Vec<PathBuf>,
        #[arg(long, value_enum)]
        pub color: Option<Color>,
        #[arg(short, long, env = "EXAMPLE_JOBS", default_value = "4")]
        pub jobs: Option<u32>,
        #[arg(value_name = "FILE")]
        pub files: Vec<PathBuf>,
        #[arg(value_name = "CMD", last = true)]
        pub cmd: Vec<String>,
        #[command(subcommand)]
        pub command: Option<Commands>,
    }

    #[derive(clap::ValueEnum, Debug, Clone, Copy)]
    pub enum Color {
        Auto,
        Always,
        Never,
    }

    #[derive(clap::Subcommand, Debug)]
    pub enum Commands {
        #[command(alias = "u")]
        Use(UseArgs),
    }

    #[derive(clap::Args, Debug)]
    pub struct UseArgs {
        #[arg(short, long)]
        pub global: bool,
        #[arg(value_name = "TOOL")]
        pub tools: Vec<String>,
    }
}

/// Parse `PARSE_N` times (default 1) and print how many parses accepted the line.
///
/// With `PARSE_TIME` set, a second line gives the wall time of the first parse
/// in nanoseconds: the cold cost a CLI actually pays, once, in a fresh process.
pub fn run(mut parse: impl FnMut() -> bool) {
    let n: usize = std::env::var("PARSE_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let timed = std::env::var_os("PARSE_TIME").is_some();
    let mut seen = 0usize;
    let mut first = None;
    for i in 0..n {
        let start = timed.then(std::time::Instant::now);
        seen += usize::from(parse());
        if i == 0 {
            first = start.map(|s| s.elapsed());
        }
    }
    println!("{seen}");
    if let Some(elapsed) = first {
        println!("{}", elapsed.as_nanos());
    }
}

/// usage (usage-derive over usage-argv), as `../usage/benches/shadows/mise` uses it.
pub mod usage {
    use usage_derive::{Args, Cli, Subcommands, ValueEnum};

    #[derive(Cli)]
    #[usage(bin = "example", name = "example", group("output"))]
    pub struct Cli {
        #[usage(long = "verbose", short = 'v', count, global)]
        pub verbose: u8,
        #[usage(long = "quiet", short = 'q', conflicts("--verbose"))]
        pub quiet: bool,
        #[usage(long = "json", group = "output")]
        pub json: bool,
        #[usage(long = "toml", group = "output")]
        pub toml: bool,
        #[usage(long = "strict", requires("--json"))]
        pub strict: bool,
        #[usage(
            long = "write",
            short = 'w',
            value_optional,
            default_missing = "./bin/mise"
        )]
        pub write: ::std::option::Option<::std::string::String>,
        #[usage(long = "path", short = 'p', alias = "dir", value_name = "PATH")]
        pub path: ::std::option::Option<::std::path::PathBuf>,
        #[usage(
            long = "include",
            short = 'I',
            value_name = "DIR",
            var,
            delimiter = ','
        )]
        pub include: ::std::vec::Vec<::std::path::PathBuf>,
        #[usage(long = "color", value_enum)]
        pub color: ::std::option::Option<Color>,
        #[usage(
            long = "jobs",
            short = 'j',
            value_name = "JOBS",
            env = "EXAMPLE_JOBS",
            default = "4"
        )]
        pub jobs: ::std::option::Option<u32>,
        #[usage(arg, name = "FILE")]
        pub files: ::std::vec::Vec<::std::path::PathBuf>,
        #[usage(arg, name = "CMD", double_dash = "required")]
        pub cmd: ::std::vec::Vec<::std::string::String>,
        #[usage(subcommand)]
        pub command: ::std::option::Option<Commands>,
    }

    #[derive(ValueEnum, Debug, Clone, Copy)]
    pub enum Color {
        #[usage(name = "auto")]
        Auto,
        #[usage(name = "always")]
        Always,
        #[usage(name = "never")]
        Never,
    }

    #[derive(Subcommands)]
    pub enum Commands {
        #[usage(name = "use", alias = "u")]
        Use(Box<UseArgs>),
    }

    #[derive(Args)]
    pub struct UseArgs {
        #[usage(long = "global", short = 'g')]
        pub global: bool,
        #[usage(arg, name = "TOOL")]
        pub tools: ::std::vec::Vec<::std::string::String>,
    }
}
