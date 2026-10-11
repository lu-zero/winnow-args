//! Global flags: declared on a parent, accepted after its subcommand word at any depth.

use winnow_args::{Args, Error, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq, Default)]
struct Root {
    #[arg(short, long, count, global)]
    verbose: u8,
    #[arg(short = 'C', long, global)]
    cd: Option<String>,
    #[arg(short)]
    local: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run(RunArgs),
    Ls,
    Tool(ToolArgs),
}

#[derive(Args, Debug, PartialEq)]
struct RunArgs {
    /// Redeclares the root's `-C`: below `run`, this one binds.
    #[arg(short = 'C')]
    cores: Option<u32>,
    #[arg(positional)]
    task: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
struct ToolArgs {
    #[arg(subcommand)]
    command: Option<ToolCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum ToolCommand {
    Install(InstallArgs),
}

#[derive(Args, Debug, PartialEq)]
struct InstallArgs {
    #[arg(short)]
    force: bool,
}

fn parse(line: &[&str]) -> Result<Root, Error> {
    Root::parse_from(line)
}

fn ok(line: &[&str]) -> Root {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

fn run(cores: Option<u32>, task: &[&str]) -> Option<Command> {
    Some(Command::Run(RunArgs {
        cores,
        task: task.iter().map(|t| t.to_string()).collect(),
    }))
}

#[test]
fn a_global_binds_on_either_side_of_the_subcommand() {
    let root = ok(&["-v", "run", "-v", "x", "--verbose"]);
    assert_eq!((root.verbose, root.command), (3, run(None, &["x"])));
    assert_eq!(ok(&["run", "--cd=/a"]).cd.as_deref(), Some("/a"));
    assert_eq!(ok(&["ls", "-vv"]).verbose, 2);
}

#[test]
fn a_redeclared_name_binds_to_the_subcommand() {
    let root = ok(&["run", "-C", "3"]);
    assert_eq!((root.cd, root.command), (None, run(Some(3), &[])));
    // The long name is not redeclared, so it still reaches the root.
    assert_eq!(ok(&["run", "--cd", "/a"]).cd.as_deref(), Some("/a"));
}

#[test]
fn globals_reach_any_depth_even_inside_a_bundle() {
    let root = ok(&["tool", "install", "-vfC", "/d"]);
    assert_eq!((root.verbose, root.cd.as_deref()), (1, Some("/d")));
    assert_eq!(
        root.command,
        Some(Command::Tool(ToolArgs {
            command: Some(ToolCommand::Install(InstallArgs { force: true })),
        }))
    );
}

#[test]
fn only_globals_are_inherited() {
    let e = parse(&["run", "-l"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-l")));
    assert!(ok(&["-l", "run"]).local);
    let e = parse(&["ls", "x"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnexpectedArg, Some("x")));
    // After `--` a global's spelling is just a word.
    let root = ok(&["run", "--", "-v"]);
    assert_eq!((root.verbose, root.command), (0, run(None, &["-v"])));
}

/// Groups of flags that several commands share, some of them global.
mod flattened {
    use winnow_args::{Args, ErrorKind, Subcommand};

    #[derive(Args, Debug, Default, PartialEq)]
    struct VerboseArg {
        #[arg(short, long, count, global)]
        verbose: u8,
    }

    /// A group that holds another, and a flag that is not global.
    #[derive(Args, Debug, Default, PartialEq)]
    struct Topology {
        #[arg(long, global, value_name = "DIR")]
        prefix: Option<String>,
        #[arg(long, short = 'T', global)]
        target: Option<String>,
        #[arg(long)]
        local: bool,
        #[arg(flatten)]
        verbose_arg: VerboseArg,
    }

    #[derive(Args, Debug)]
    struct Cli {
        #[arg(long, global, default = "auto")]
        color: String,
        #[arg(flatten)]
        topology: Topology,
        #[arg(subcommand)]
        applet: Applet,
    }

    #[derive(Subcommand, Debug, PartialEq)]
    enum Applet {
        Maint(Maint),
        Emerge(Emerge),
    }

    #[derive(Args, Debug, PartialEq)]
    struct Maint {
        #[arg(subcommand)]
        command: MaintCommand,
    }

    #[derive(Subcommand, Debug, PartialEq)]
    enum MaintCommand {
        World(World),
    }

    #[derive(Args, Debug, PartialEq)]
    struct World {
        #[arg(long)]
        fix: bool,
    }

    /// A command that flattens the same group itself.
    #[derive(Args, Debug, PartialEq)]
    struct Emerge {
        #[arg(flatten)]
        verbose_arg: VerboseArg,
        #[arg(positional)]
        atoms: Vec<String>,
    }

    fn ok(line: &[&str]) -> Cli {
        Cli::parse_from(line).unwrap()
    }

    #[test]
    fn a_flattened_struct_s_global_flags_reach_the_subcommands() {
        let cli = ok(&["maint", "world", "-v", "--fix"]);
        assert_eq!(cli.topology.verbose_arg.verbose, 1);
        assert_eq!(
            cli.applet,
            Applet::Maint(Maint {
                command: MaintCommand::World(World { fix: true })
            })
        );
        // At every depth, each occurrence counted once.
        let cli = ok(&[
            "-v", "maint", "-v", "world", "-vv", "--prefix", "/p", "-Tx86",
        ]);
        let topology = cli.topology;
        assert_eq!(topology.verbose_arg.verbose, 4);
        assert_eq!(topology.prefix.as_deref(), Some("/p"));
        assert_eq!(topology.target.as_deref(), Some("x86"));
        // Beside the struct's own global, and attached or not.
        let cli = ok(&["maint", "world", "--prefix=/q", "--color", "never"]);
        assert_eq!(
            (cli.color.as_str(), cli.topology.prefix.as_deref()),
            ("never", Some("/q"))
        );
    }

    #[test]
    fn their_letters_bundle_and_one_may_take_a_value() {
        let cli = ok(&["maint", "world", "-vT", "arm"]);
        assert_eq!(cli.topology.verbose_arg.verbose, 1);
        assert_eq!(cli.topology.target.as_deref(), Some("arm"));
        let cli = ok(&["maint", "world", "-vvTarm"]);
        assert_eq!(cli.topology.verbose_arg.verbose, 2);
        assert_eq!(cli.topology.target.as_deref(), Some("arm"));
    }

    #[test]
    fn only_the_global_ones_do() {
        let e = Cli::parse_from(["maint", "world", "--local"]).unwrap_err();
        assert_eq!(
            (e.kind(), e.token()),
            (ErrorKind::UnknownFlag, Some("--local"))
        );
        assert!(ok(&["--local", "maint", "world"]).topology.local);
    }

    #[test]
    fn completion_and_tips_know_them_below() {
        let offered: Vec<String> =
            winnow_args::complete::complete(Cli::HELP, &["maint", "world", "--"])
                .candidates
                .into_iter()
                .map(|c| c.value)
                .collect();
        for flag in ["--fix", "--prefix", "--target", "--verbose", "--color"] {
            assert!(offered.iter().any(|o| o == flag), "{flag} in {offered:?}");
        }
        assert!(!offered.iter().any(|o| o == "--local"), "{offered:?}");
        let e = Cli::parse_from(["maint", "world", "--prefx", "/p"]).unwrap_err();
        assert_eq!(e.similar(), Some("--prefix"));
    }

    #[test]
    fn a_command_that_flattens_the_group_itself_keeps_its_own() {
        let cli = ok(&["-v", "emerge", "-vv", "x"]);
        assert_eq!(cli.topology.verbose_arg.verbose, 1);
        let Applet::Emerge(emerge) = cli.applet else {
            panic!("not an emerge");
        };
        assert_eq!((emerge.verbose_arg.verbose, emerge.atoms.len()), (2, 1));
        // What it does not declare is still the root's.
        let cli = ok(&["emerge", "--prefix", "/r", "x"]);
        assert_eq!(cli.topology.prefix.as_deref(), Some("/r"));
    }
}
