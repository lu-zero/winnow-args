//! `example -v/--verbose... -p/--path=PATH -I/--include=DIR... [FILE]... [use -g/--global [TOOL]...]`
//! in six spellings.

/// winnow-args, derived.
pub mod wa_derive {
    use std::path::PathBuf;

    #[derive(winnow_args::Args, Debug)]
    pub struct Cli {
        #[arg(short, long, count)]
        pub verbose: u8,
        #[arg(short, long)]
        pub path: Option<PathBuf>,
        #[arg(short = 'I', long)]
        pub include: Vec<PathBuf>,
        #[arg(positional, value_name = "FILE")]
        pub files: Vec<PathBuf>,
        #[arg(subcommand)]
        pub command: Option<Commands>,
    }

    #[derive(winnow_args::Subcommand, Debug)]
    pub enum Commands {
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
    use winnow_args::combinator::{Named, args, command, positional, short};
    use winnow_args::token::{Kind, kind};
    use winnow_args::{Argv, Error};

    #[derive(Debug, Default)]
    pub struct Cli {
        pub verbose: u8,
        pub path: Option<PathBuf>,
        pub include: Vec<PathBuf>,
        pub files: Vec<PathBuf>,
        pub command: Option<Commands>,
    }

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

    fn use_args(input: &mut Argv<'_>) -> Result<UseArgs, Error> {
        let mut u = UseArgs::default();
        let c = &mut u;
        args(dispatch! {kind;
            Kind::Long | Kind::Short => GLOBAL.switch().map(|()| c.global = true),
            Kind::Word => positional("TOOL").map(|t| c.tools.push(t)),
            Kind::Separator => fail,
        })
        .parse_next(input)?;
        Ok(u)
    }

    const VERBOSE: Named = short('v').long("verbose");
    const PATH: Named = short('p').long("path");
    const INCLUDE: Named = short('I').long("include");

    pub fn cli(input: &mut Argv<'_>) -> Result<Cli, Error> {
        let mut cli = Cli::default();
        let c = &mut cli;
        args(dispatch! {kind;
            Kind::Long | Kind::Short => alt((
                VERBOSE.switch().map(|()| c.verbose = c.verbose.saturating_add(1)),
                PATH.argument_as().map(|p| c.path = Some(p)),
                INCLUDE.argument_as().map(|i| c.include.push(i)),
            )),
            Kind::Word => alt((
                cond(c.files.is_empty(), command("use", use_args))
                    .verify_map(|found| found)
                    .map(|u| c.command = Some(Commands::Use(u))),
                positional("FILE").map(|f| c.files.push(f)),
            )),
            Kind::Separator => fail,
        })
        .parse_next(input)?;
        Ok(cli)
    }
}

/// winnow-args, combinators: one `dispatch!` matching flag names as patterns,
/// the same shape as the derive's `match`.
pub mod wa_disp {
    use winnow::combinator::{dispatch, fail};
    use winnow::prelude::*;
    use winnow_args::combinator::args;
    use winnow_args::token::{Arg, LongFlag, ShortFlag, arg};
    use winnow_args::{Argv, Error};

    pub use super::wa_comb::{Cli, Commands, UseArgs};

    fn use_args(input: &mut Argv<'_>) -> Result<UseArgs, Error> {
        let mut u = UseArgs::default();
        let c = &mut u;
        args(dispatch! {arg;
            a @ (Arg::Long(LongFlag { name: b"global", .. }) | Arg::Short(ShortFlag { letter: 'g', .. })) => {
                a.switch().map(|()| c.global = true)
            },
            Arg::Word(w) => w.value_as("TOOL").map(|t| c.tools.push(t)),
            _ => fail,
        })
        .parse_next(input)?;
        Ok(u)
    }

    pub fn cli(input: &mut Argv<'_>) -> Result<Cli, Error> {
        let mut cli = Cli::default();
        let c = &mut cli;
        args(dispatch! {arg;
            a @ (Arg::Long(LongFlag { name: b"verbose", .. }) | Arg::Short(ShortFlag { letter: 'v', .. })) => {
                a.switch().map(|()| c.verbose = c.verbose.saturating_add(1))
            },
            a @ (Arg::Long(LongFlag { name: b"path", .. }) | Arg::Short(ShortFlag { letter: 'p', .. })) => {
                a.value_as().map(|p| c.path = Some(p))
            },
            a @ (Arg::Long(LongFlag { name: b"include", .. }) | Arg::Short(ShortFlag { letter: 'I', .. })) => {
                a.value_as().map(|i| c.include.push(i))
            },
            Arg::Word(w) if c.files.is_empty() && !w.after_separator && *w.value == "use" => {
                use_args.map(|u| c.command = Some(Commands::Use(u)))
            },
            Arg::Word(w) => w.value_as("FILE").map(|f| c.files.push(f)),
            _ => fail,
        })
        .parse_next(input)?;
        Ok(cli)
    }
}

/// bpaf 0.10, derived.
pub mod bpaf010 {
    use std::path::PathBuf;

    use bpaf::Bpaf;

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options, generate(cli_p))]
    pub struct Cli {
        #[bpaf(short('v'), long("verbose"), req_flag(()), count)]
        pub verbose: usize,
        #[bpaf(short('p'), long("path"), argument("PATH"))]
        pub path: Option<PathBuf>,
        #[bpaf(short('I'), long("include"), argument("DIR"))]
        pub include: Vec<PathBuf>,
        // bpaf tries items in order: the command must come before the greedy
        // positional, or `use` is taken as a FILE.
        #[bpaf(external(commands_p), optional)]
        pub command: Option<Commands>,
        #[bpaf(positional("FILE"))]
        pub files: Vec<PathBuf>,
    }

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(generate(commands_p))]
    pub enum Commands {
        #[bpaf(command("use"))]
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
    pub struct Cli {
        #[arg(short, long, action = clap::ArgAction::Count)]
        pub verbose: u8,
        #[arg(short, long)]
        pub path: Option<PathBuf>,
        #[arg(short = 'I', long, value_name = "DIR")]
        pub include: Vec<PathBuf>,
        #[arg(value_name = "FILE")]
        pub files: Vec<PathBuf>,
        #[command(subcommand)]
        pub command: Option<Commands>,
    }

    #[derive(clap::Subcommand, Debug)]
    pub enum Commands {
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
    use usage_derive::{Args, Cli, Subcommands};

    #[derive(Cli)]
    #[usage(bin = "example", name = "example")]
    pub struct Cli {
        #[usage(long = "verbose", short = 'v', count)]
        pub verbose: u8,
        #[usage(long = "path", short = 'p', value_name = "PATH")]
        pub path: ::std::option::Option<::std::path::PathBuf>,
        #[usage(long = "include", short = 'I', value_name = "DIR", var)]
        pub include: ::std::vec::Vec<::std::path::PathBuf>,
        #[usage(arg, name = "FILE")]
        pub files: ::std::vec::Vec<::std::path::PathBuf>,
        #[usage(subcommand)]
        pub command: ::std::option::Option<Commands>,
    }

    #[derive(Subcommands)]
    pub enum Commands {
        #[usage(name = "use")]
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
