//! `example -v/--verbose... -p/--path=PATH [FILE]...` in five spellings.

/// winnow-args, derived.
pub mod wa_derive {
    use std::path::PathBuf;

    #[derive(winnow_args::Args, Debug)]
    pub struct Cli {
        #[arg(short, long, count)]
        pub verbose: u8,
        #[arg(short, long)]
        pub path: Option<PathBuf>,
        #[arg(positional, value_name = "FILE")]
        pub files: Vec<PathBuf>,
    }
}

/// winnow-args, combinators.
pub mod wa_comb {
    use std::path::PathBuf;

    use winnow::combinator::alt;
    use winnow::prelude::*;
    use winnow_args::combinator::{Named, args, positional, short};
    use winnow_args::{Argv, Error};

    #[derive(Debug)]
    pub struct Cli {
        pub verbose: u8,
        pub path: Option<PathBuf>,
        pub files: Vec<PathBuf>,
    }

    const VERBOSE: Named = short('v').long("verbose");
    const PATH: Named = short('p').long("path");

    pub fn cli(input: &mut Argv<'_>) -> Result<Cli, Error> {
        let mut verbose = 0u8;
        let mut path = None;
        let mut files = Vec::new();
        args(alt((
            VERBOSE
                .switch()
                .map(|()| verbose = verbose.saturating_add(1)),
            PATH.argument_as::<PathBuf>().map(|p| path = Some(p)),
            positional::<PathBuf>("FILE").map(|f| files.push(f)),
        )))
        .parse_next(input)?;
        Ok(Cli {
            verbose,
            path,
            files,
        })
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
        #[bpaf(positional("FILE"))]
        pub files: Vec<PathBuf>,
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
        #[arg(value_name = "FILE")]
        pub files: Vec<PathBuf>,
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
    use usage_derive::Cli;

    #[derive(Cli)]
    #[usage(bin = "example", name = "example")]
    pub struct Cli {
        #[usage(long = "verbose", short = 'v', count)]
        pub verbose: u8,
        #[usage(long = "path", short = 'p', value_name = "PATH")]
        pub path: ::std::option::Option<::std::path::PathBuf>,
        #[usage(arg, name = "FILE")]
        pub files: ::std::vec::Vec<::std::path::PathBuf>,
    }
}
