//! What 0.3 adds to a help page: the default subcommand, the global flags a
//! subcommand inherits, examples, what is said under a heading, subcommands
//! laid out in place of their list, and a flag's warning.

use winnow_args::{Args, Error, Subcommand, with_env};

/// A package manager
#[derive(Args, Debug)]
#[arg(
    name = "em",
    default_subcommand = "emerge",
    default_subcommand_flags,
    default_subcommand_help,
    example(
        "em -ua @world",
        header = "Update @world",
        help = "Rebuilds what changed."
    ),
    example("em query belongs /usr/bin/python", header = "Find a file's owner")
)]
struct Cli {
    /// When to paint
    #[arg(long, global, value_name = "WHEN")]
    color: Option<String>,
    #[arg(flatten)]
    roots: Roots,
    #[arg(subcommand)]
    applet: Option<Applet>,
}

#[derive(Args, Debug)]
#[arg(
    next_help_heading = "Roots",
    heading("Roots", help = "Which tree this invocation reads and writes.")
)]
struct Roots {
    /// The tree to merge into
    #[arg(long, global, value_name = "DIR")]
    root: Option<String>,
    /// Not shown
    #[arg(long, global, hide)]
    sysroot: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Applet {
    /// Merge packages
    Emerge(Emerge),
    /// Ask about packages
    Query(Query),
    /// Print the version
    Version,
}

#[derive(Args, Debug)]
#[arg(example = "em emerge firefox")]
struct Emerge {
    /// Only say what would be merged
    #[arg(short, long)]
    pretend: bool,
    /// Carry on after a failure
    #[arg(
        long,
        warning = "Exists for parity; a failed package must stop the run."
    )]
    keep_going: bool,
    /// What to merge
    #[arg(positional)]
    atoms: Vec<String>,
}

/// Ask about packages
#[derive(Args, Debug)]
#[arg(
    flatten_help,
    example = "em query depgraph zlib",
    example = "em query belongs /usr/bin/python"
)]
struct Query {
    /// Say more
    #[arg(short, long)]
    verbose: bool,
    #[arg(subcommand)]
    what: What,
}

#[derive(Subcommand, Debug)]
enum What {
    /// Show the dependency tree
    Depgraph(Depgraph),
    /// Find the package that owns a file
    Belongs(Belongs),
    /// Not listed
    #[arg(hide)]
    Debug,
}

#[derive(Args, Debug)]
struct Depgraph {
    /// How to print it
    #[arg(long, choices("tree", "json"), default = "tree")]
    format: String,
    /// The packages to resolve
    #[arg(positional, required)]
    atom: Vec<String>,
}

#[derive(Args, Debug)]
struct Belongs {
    /// The file to look for
    #[arg(positional)]
    path: String,
}

fn help(line: &[&str]) -> String {
    let error: Error = Cli::parse_from(line).unwrap_err();
    with_env(&[("COLUMNS", "80")], || error.render_help("em")).expect("a help request")
}

fn completed(words: &[&str]) -> Vec<String> {
    let completions = winnow_args::complete::complete(Cli::HELP, words);
    completions
        .candidates
        .into_iter()
        .map(|c| c.value)
        .collect()
}

#[test]
fn the_default_subcommand_is_marked_and_its_page_follows() {
    let page = help(&["-h"]);
    assert!(
        page.contains("  emerge   Merge packages (default)\n"),
        "{page}"
    );
    let (root, default) = page
        .split_once("\nDefault command: emerge\n  It runs when no command is named.\n\n")
        .expect("the default command's page");
    assert!(root.contains("--color <WHEN>"), "{root}");
    assert!(
        default.starts_with("Usage: em emerge [OPTIONS] [ATOMS]...\n"),
        "{default}"
    );
    assert!(default.contains("-p, --pretend"), "{default}");
    // What it inherits is the root's, listed above.
    assert!(!default.contains("--color"), "{default}");
    assert!(!default.contains("Global options"), "{default}");
}

#[test]
fn a_subcommand_lists_the_global_flags_it_inherits() {
    let page = help(&["emerge", "-h"]);
    let (_, globals) = page
        .split_once("\nGlobal options:\n")
        .expect("the inherited flags");
    assert!(
        globals.starts_with(
            "      --color <WHEN>  When to paint\n      --root <DIR>    The tree to merge into\n"
        ),
        "{page}"
    );
    // A hidden one stays hidden, and a flag that is not global is not inherited.
    assert!(!page.contains("--sysroot"), "{page}");
    let page = help(&["query", "depgraph", "-h"]);
    assert!(page.contains("\nGlobal options:\n"), "{page}");
    assert!(!page.contains("--verbose"), "{page}");
    // The root inherits nothing.
    assert!(!help(&["query", "-h"]).is_empty());
    assert!(
        !help(&["-h"])
            .split("Default command")
            .next()
            .unwrap()
            .contains("Global options")
    );
    // `help emerge` is the same page.
    assert_eq!(help(&["help", "emerge"]), help(&["emerge", "--help"]));
}

#[test]
fn a_unit_subcommand_lists_them_too() {
    let page = help(&["version", "-h"]);
    assert!(
        page.contains("\nGlobal options:\n      --color <WHEN>"),
        "{page}"
    );
}

#[test]
fn examples_follow_the_lists() {
    let short = help(&["-h"]);
    assert!(
        short.contains(
            "\nExamples:\n  Update @world:\n    $ em -ua @world\n\n  Find a file's owner:\n    $ em query belongs /usr/bin/python\n"
        ),
        "{short}"
    );
    // What a line does is said on the long page.
    let long = help(&["--help"]);
    assert!(
        long.contains("  Update @world:\n    Rebuilds what changed.\n    $ em -ua @world\n"),
        "{long}"
    );
    // Lines with nothing said of them stand together.
    let page = help(&["query", "-h"]);
    assert!(
        page.ends_with(
            "\nExamples:\n  $ em query depgraph zlib\n  $ em query belongs /usr/bin/python\n"
        ),
        "{page}"
    );
}

#[test]
fn a_heading_has_its_paragraph_on_the_long_page() {
    let short = help(&["-h"]);
    assert!(
        short.contains("\nRoots:\n      --root <DIR>  The tree to merge into\n"),
        "{short}"
    );
    let long = help(&["--help"]);
    assert!(
        long.contains(
            "\nRoots:\n  Which tree this invocation reads and writes.\n\n      --root <DIR>\n"
        ),
        "{long}"
    );
}

#[test]
fn flatten_help_lays_the_subcommands_out() {
    let page = help(&["query", "-h"]);
    assert!(
        page.contains(
            "Usage: em query depgraph [OPTIONS] <ATOM>...\n       em query belongs [OPTIONS] <PATH>\n\n\
             em query depgraph:\n  Show the dependency tree\n  <ATOM>...              The packages to resolve\n"
        ),
        "{page}"
    );
    assert!(
        page.contains("em query belongs:\n  Find the package that owns a file\n  <PATH>  The file to look for\n\nOptions:\n"),
        "{page}"
    );
    assert!(!page.contains("Commands:"), "{page}");
    assert!(!page.contains("debug"), "{page}");
    let long = help(&["query", "--help"]);
    assert!(
        long.contains("em query belongs:\n  Find the package that owns a file\n\n  <PATH>\n          The file to look for\n\nOptions:\n"),
        "{long}"
    );
}

#[test]
fn a_warning_is_on_the_long_page() {
    let long = help(&["emerge", "--help"]);
    assert!(
        long.contains(
            "      --keep-going\n          Carry on after a failure\n          Warning: Exists for parity; a failed package must stop the run.\n"
        ),
        "{long}"
    );
    assert!(!help(&["emerge", "-h"]).contains("Warning"));
}

#[test]
fn completion_offers_what_the_default_subcommand_takes() {
    // Its flags, where a flag selects it.
    let flags = completed(&["-"]);
    for flag in [
        "--color",
        "--root",
        "--pretend",
        "-p",
        "--keep-going",
        "--help",
    ] {
        assert!(flags.iter().any(|f| f == flag), "{flag}: {flags:?}");
    }
    assert_eq!(
        flags.iter().filter(|f| *f == "--help").count(),
        1,
        "{flags:?}"
    );
    assert_eq!(completed(&["--pr"]), ["--pretend"]);
    // Once it is selected, only its own and the global ones.
    let flags = completed(&["-p", "-"]);
    assert!(flags.iter().any(|f| f == "--keep-going"), "{flags:?}");
    assert!(flags.iter().any(|f| f == "--color"), "{flags:?}");
    // A word that names no subcommand selects it as well.
    let flags = completed(&["firefox", "--k"]);
    assert_eq!(flags, ["--keep-going"]);
    // A subcommand named is still itself.
    assert_eq!(completed(&["query", "dep"]), ["depgraph"]);
    assert_eq!(completed(&["que"]), ["query"]);
}

#[test]
fn a_hand_written_command_ends_in_empty() {
    use winnow_args::help::{Command, Item, render_width};
    static ITEMS: &[Item] = &[Item {
        long: Some("force"),
        help: "Do it anyway",
        ..Item::EMPTY
    }];
    static COMMAND: Command = Command {
        about: "A tool",
        items: ITEMS,
        ..Command::EMPTY
    };
    assert_eq!(
        render_width(&COMMAND, &["tool"], false, 80),
        "A tool\n\nUsage: tool [OPTIONS]\n\nOptions:\n      --force  Do it anyway\n"
    );
}

/// Flags two commands on the way both flatten are one flag, and text of
/// several paragraphs keeps them apart.
mod nested {
    use winnow_args::{Args, Subcommand, with_env};

    #[derive(Args, Debug)]
    struct Common {
        /// Say more
        #[arg(short, long, global)]
        verbose: bool,
    }

    #[derive(Args, Debug)]
    #[arg(
        name = "tool",
        heading("Roots", help = "First.\n\nSecond."),
        example("tool mid leaf", header = "Go down", help = "One.\n\nTwo.")
    )]
    struct Tool {
        #[arg(flatten)]
        common: Common,
        /// Where to work
        #[arg(short = 'C', long, global, help_heading = "Roots")]
        dir: Option<String>,
        #[arg(subcommand)]
        command: Top,
    }

    #[derive(Subcommand, Debug)]
    enum Top {
        Mid(Mid),
    }

    /// The middle
    #[derive(Args, Debug)]
    #[arg(flatten_help)]
    struct Mid {
        #[arg(flatten)]
        common: Common,
        #[arg(subcommand)]
        sub: Bottom,
    }

    #[derive(Subcommand, Debug)]
    enum Bottom {
        /// The leaf
        Leaf(Leaf),
        Unit,
    }

    /// The leaf
    ///
    /// It has more to say.
    ///
    /// And more.
    #[derive(Args, Debug)]
    struct Leaf {
        /// Its own letter, another flag's
        #[arg(short = 'C', long)]
        chdir: bool,
    }

    fn help(line: &[&str]) -> String {
        let error = Tool::parse_from(line).unwrap_err();
        with_env(&[("COLUMNS", "80")], || error.render_help("tool")).expect("a help request")
    }

    #[test]
    fn a_global_two_commands_flatten_is_listed_once() {
        for line in [
            &["mid", "leaf", "--help"][..],
            &["mid", "unit", "-h"],
            &["help", "mid", "leaf"],
        ] {
            let page = help(line);
            assert_eq!(page.matches("--verbose").count(), 1, "{line:?}\n{page}");
        }
        // A flag that shares only a letter with one of the command's own is
        // still taken by its long name, and still listed.
        let page = help(&["mid", "leaf", "-h"]);
        assert!(page.contains("--chdir"), "{page}");
        assert!(page.contains("-C, --dir <DIR>"), "{page}");
        assert!(Tool::parse_from(["mid", "leaf", "--dir", "x", "-C"]).is_ok());
    }

    #[test]
    fn paragraphs_stay_apart() {
        let page = help(&["--help"]);
        assert!(
            page.contains("\nRoots:\n  First.\n\n  Second.\n\n  -C, --dir <DIR>\n"),
            "{page}"
        );
        assert!(
            page.contains("  Go down:\n    One.\n\n    Two.\n    $ tool mid leaf\n"),
            "{page}"
        );
        let page = help(&["mid", "--help"]);
        assert!(
            page.contains("tool mid leaf:\n  The leaf\n\n  It has more to say.\n\n  And more.\n\n"),
            "{page}"
        );
        for line in page.lines() {
            assert_eq!(line, line.trim_end(), "{page:?}");
        }
    }
}

/// `hide_default_value`, `hide_env` and `display_order`.
mod hidden_and_ordered {
    use winnow_args::{Args, with_env};

    #[derive(Args, Debug)]
    #[arg(name = "deploy")]
    struct Deploy {
        /// The region
        #[arg(long, default = "eu-west-1", env = "DEPLOY_REGION")]
        region: String,
        /// The token
        #[arg(
            long,
            env = "DEPLOY_TOKEN",
            hide_env,
            default = "dev",
            hide_default_value
        )]
        token: String,
        /// Listed first
        #[arg(long, display_order = 0)]
        yes: bool,
    }

    fn help() -> String {
        let error = Deploy::parse_from(["-h"]).unwrap_err();
        with_env(&[("COLUMNS", "100")], || error.render_help("deploy")).unwrap()
    }

    #[test]
    fn hidden_details_stay_out_of_help_and_still_work() {
        let page = help();
        assert!(
            page.contains("[env: DEPLOY_REGION] [default: eu-west-1]"),
            "{page}"
        );
        assert!(!page.contains("DEPLOY_TOKEN"), "{page}");
        assert!(!page.contains("dev"), "{page}");
        let deploy = with_env(&[("DEPLOY_TOKEN", "t")], || {
            Deploy::parse_from::<[&str; 0], &str>([])
        })
        .unwrap();
        assert_eq!(deploy.token, "t");
        assert_eq!(
            Deploy::parse_from::<[&str; 0], &str>([]).unwrap().token,
            "dev"
        );
    }

    #[test]
    fn display_order_moves_an_entry() {
        let page = help();
        let yes = page.find("--yes").unwrap();
        assert!(yes < page.find("--region").unwrap(), "{page}");
        // Parsing is unchanged.
        assert!(Deploy::parse_from(["--yes"]).unwrap().yes);
    }
}
