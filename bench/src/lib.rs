//! `example -v/--verbose -p/--path=PATH` in four spellings.

/// winnow-args, derived.
pub mod wa_derive {
    use std::path::PathBuf;

    #[derive(winnow_args::Args, Debug)]
    pub struct Cli {
        #[arg(short, long)]
        pub verbose: bool,
        #[arg(short, long)]
        pub path: Option<PathBuf>,
    }
}

/// winnow-args, combinators.
pub mod wa_comb {
    use std::path::PathBuf;

    use winnow::combinator::alt;
    use winnow::prelude::*;
    use winnow_args::combinator::{Named, args, short};
    use winnow_args::{Argv, Error};

    #[derive(Debug)]
    pub struct Cli {
        pub verbose: bool,
        pub path: Option<PathBuf>,
    }

    const VERBOSE: Named = short('v').long("verbose");
    const PATH: Named = short('p').long("path");

    pub fn cli(input: &mut Argv<'_>) -> Result<Cli, Error> {
        let mut verbose = false;
        let mut path = None;
        args(alt((
            VERBOSE.switch().map(|()| verbose = true),
            PATH.argument_as::<PathBuf>().map(|p| path = Some(p)),
        )))
        .parse_next(input)?;
        Ok(Cli { verbose, path })
    }
}

/// bpaf 0.10, derived.
pub mod bpaf010 {
    use std::path::PathBuf;

    use bpaf::Bpaf;

    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options, generate(cli_p))]
    pub struct Cli {
        #[bpaf(short('v'), long("verbose"), switch)]
        pub verbose: bool,
        #[bpaf(short('p'), long("path"), argument("PATH"))]
        pub path: Option<PathBuf>,
    }
}

/// clap 4, derived.
pub mod clap4 {
    use std::path::PathBuf;

    #[derive(clap::Parser, Debug)]
    pub struct Cli {
        #[arg(short, long)]
        pub verbose: bool,
        #[arg(short, long)]
        pub path: Option<PathBuf>,
    }
}

/// Repeat count from the environment; 1 when unset.
pub fn parse_n() -> usize {
    std::env::var("PARSE_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

/// usage (usage-derive over usage-argv), as `../usage/benches/shadows/mise` uses it.
pub mod usage {
    use usage_derive::Cli;

    #[derive(Cli)]
    #[usage(bin = "example", name = "example")]
    pub struct Cli {
        #[usage(long = "verbose", short = 'v')]
        pub verbose: bool,
        #[usage(long = "path", short = 'p', value_name = "PATH")]
        pub path: ::std::option::Option<::std::path::PathBuf>,
    }
}
