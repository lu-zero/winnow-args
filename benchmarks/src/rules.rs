//! The rules CLI: `export` with conditional requirements, a conditional default
//! and a check across fields, the same in each framework that has them built
//! in. Its lines are `rules-argv.txt`, compared by `SUITE=rules just perf`.
//!
//! - `--schema` is required when `--format` is `json`, `--endpoint` when
//!   `--upload` is given;
//! - `--target` is required unless both `--dir` and `--name` are given;
//! - `--level` is 9 for `json` and 6 otherwise;
//! - `--from` is at most `--to`, checked once the command line is parsed.
//!
//! bpaf has none of these rules built in, so it is not measured here.

/// What `--from` and `--to` must satisfy, shared by every framework.
fn in_order(from: u32, to: u32) -> bool {
    from <= to
}

/// winnow-args, derived.
pub mod wa {
    use winnow_args::Error;

    #[derive(winnow_args::Args, Debug)]
    #[arg(name = "export", validate = check)]
    pub struct Cli {
        #[arg(long, env = "EXPORT_FORMAT")]
        pub format: Option<Format>,
        #[arg(long, required_if("--format", "json"))]
        pub schema: Option<String>,
        #[arg(long)]
        pub upload: bool,
        #[arg(long, required_if("--upload"))]
        pub endpoint: Option<String>,
        #[arg(long, default_if("--format", "json", "9"), default = "6")]
        pub level: u32,
        #[arg(long)]
        pub dir: Option<String>,
        #[arg(long)]
        pub name: Option<String>,
        #[arg(long, required_unless_all("--dir", "--name"))]
        pub target: Option<String>,
        #[arg(long, default = "0")]
        pub from: u32,
        #[arg(long, default = "100")]
        pub to: u32,
    }

    #[derive(winnow_args::ValueEnum, Debug, Clone, Copy, PartialEq)]
    pub enum Format {
        Text,
        Json,
    }

    fn check(cli: &Cli) -> Result<(), Error> {
        if super::in_order(cli.from, cli.to) {
            Ok(())
        } else {
            Err(Error::invalid("`--from` is after `--to`"))
        }
    }
}

/// clap 4, derived. clap has no hook on the built value: the check runs after
/// the parse, as a clap program writes it.
pub mod clap4 {
    use clap::Parser as _;

    #[derive(clap::Parser, Debug)]
    #[command(name = "export")]
    pub struct Cli {
        #[arg(long, env = "EXPORT_FORMAT", value_enum)]
        pub format: Option<Format>,
        #[arg(long, required_if_eq("format", "json"))]
        pub schema: Option<String>,
        #[arg(long)]
        pub upload: bool,
        #[arg(long, required_if_eq("upload", "true"))]
        pub endpoint: Option<String>,
        #[arg(
            long,
            default_value_if("format", "json", Some("9")),
            default_value = "6"
        )]
        pub level: u32,
        #[arg(long)]
        pub dir: Option<String>,
        #[arg(long)]
        pub name: Option<String>,
        #[arg(long, required_unless_present_all(["dir", "name"]))]
        pub target: Option<String>,
        #[arg(long, default_value = "0")]
        pub from: u32,
        #[arg(long, default_value = "100")]
        pub to: u32,
    }

    #[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq)]
    pub enum Format {
        Text,
        Json,
    }

    /// Parse `argv`, program name first, and check the result.
    pub fn parse(argv: &[std::ffi::OsString]) -> Option<Cli> {
        Cli::try_parse_from(argv)
            .ok()
            .filter(|cli| super::in_order(cli.from, cli.to))
    }
}

/// usage, derived, with its check as `try_into`.
pub mod usage {
    use usage_derive::{Cli as UsageCli, ValueEnum};

    #[derive(UsageCli)]
    #[usage(bin = "export", name = "export", try_into = Checked)]
    pub struct Cli {
        #[usage(long = "format", env = "EXPORT_FORMAT", value_enum)]
        pub format: ::std::option::Option<Format>,
        #[usage(long = "schema", required_if_eq("--format", "json"))]
        pub schema: ::std::option::Option<::std::string::String>,
        #[usage(long = "upload")]
        pub upload: bool,
        #[usage(long = "endpoint", required_if("--upload"))]
        pub endpoint: ::std::option::Option<::std::string::String>,
        #[usage(long = "level", default_if("--format", "json", "9"), default = "6")]
        pub level: u32,
        #[usage(long = "dir")]
        pub dir: ::std::option::Option<::std::string::String>,
        #[usage(long = "name")]
        pub name: ::std::option::Option<::std::string::String>,
        #[usage(long = "target", required_unless_all = ["--dir", "--name"])]
        pub target: ::std::option::Option<::std::string::String>,
        #[usage(long = "from", default = "0")]
        pub from: u32,
        #[usage(long = "to", default = "100")]
        pub to: u32,
    }

    #[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
    pub enum Format {
        Text,
        Json,
    }

    /// The parsed command line, once checked.
    pub struct Checked(pub Cli);

    impl TryFrom<Cli> for Checked {
        type Error = usage_rs::ValidationError;

        fn try_from(cli: Cli) -> Result<Self, Self::Error> {
            if super::in_order(cli.from, cli.to) {
                Ok(Self(cli))
            } else {
                Err(usage_rs::ValidationError::field("--from").reason("after `--to`"))
            }
        }
    }
}
