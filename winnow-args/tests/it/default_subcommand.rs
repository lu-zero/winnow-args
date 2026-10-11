//! `default_subcommand` and `arg_required_else_help`. The first follows usage's
//! corpus, `09-default-subcommand.json`; each test names the vector it mirrors.

use winnow_args::{Args, Error, ErrorKind, Subcommand};

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::parse_from(line)
}

#[derive(Args, Debug, PartialEq)]
#[arg(default_subcommand = "run")]
struct Ex {
    #[arg(long)]
    verbose: bool,
    #[arg(positional, value_name = "ROOT_TASK")]
    root_task: Option<String>,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Command {
    Run(RunArgs),
    Install,
}

#[derive(Args, Debug, PartialEq)]
struct RunArgs {
    #[arg(positional)]
    task: Option<String>,
    #[arg(subcommand)]
    command: Option<RunCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum RunCommand {
    Lint,
}

fn run(task: Option<&str>, command: Option<RunCommand>) -> Option<Command> {
    Some(Command::Run(RunArgs {
        task: task.map(Into::into),
        command,
    }))
}

#[test]
fn default_route_and_default_beats_the_root_own_arg() {
    let ex: Ex = parse(&["build"]).unwrap();
    assert_eq!((ex.root_task, ex.command), (None, run(Some("build"), None)));
}

#[test]
fn default_not_used_when_named() {
    assert_eq!(
        parse::<Ex>(&["install"]).unwrap().command,
        Some(Command::Install)
    );
}

#[test]
fn default_word_reexamined() {
    // The word that selected the default is read again by it, as its subcommand.
    assert_eq!(
        parse::<Ex>(&["lint"]).unwrap().command,
        run(None, Some(RunCommand::Lint))
    );
}

#[test]
fn default_flag_before_the_word() {
    let ex: Ex = parse(&["--verbose", "build"]).unwrap();
    assert_eq!((ex.verbose, ex.command), (true, run(Some("build"), None)));
}

#[test]
fn default_after_separator() {
    let ex: Ex = parse(&["--", "build"]).unwrap();
    assert_eq!((ex.root_task.as_deref(), ex.command), (Some("build"), None));
}

#[test]
fn option_off_keeps_root() {
    // `17-default-subcommand-on-empty.json`: an empty line selects nothing.
    assert_eq!(parse::<Ex>(&[]).unwrap().command, None);
}

#[test]
fn default_not_taken_by_an_unknown_flag() {
    // usage's default is lenient, handing `--wat` to ROOT_TASK; winnow-args is
    // strict (see the checklist), so it is an unknown flag. Either way, the default
    // subcommand does not take it.
    assert_eq!(
        parse::<Ex>(&["--wat"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
    assert_eq!(
        parse::<Ex>(&["-x"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[derive(Args, Debug, PartialEq)]
#[arg(default_subcommand = "r")]
struct ByAlias {
    #[arg(subcommand)]
    command: Option<AliasCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum AliasCommand {
    #[arg(alias = "r")]
    Run(RunArgs),
}

#[test]
fn default_named_by_an_alias() {
    assert_eq!(
        parse::<ByAlias>(&["build"]).unwrap().command,
        Some(AliasCommand::Run(RunArgs {
            task: Some("build".into()),
            command: None
        }))
    );
}

#[derive(Args, Debug)]
#[arg(default_subcommand = "ls")]
struct RootOnly {
    #[arg(subcommand)]
    command: Option<RootOnlyCommand>,
}

#[derive(Subcommand, Debug)]
enum RootOnlyCommand {
    Ls,
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
struct ConfigArgs {
    #[arg(subcommand)]
    command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Ls(LsArgs),
}

#[derive(Args, Debug)]
struct LsArgs {
    #[arg(positional)]
    what: Option<String>,
}

#[test]
fn default_is_declared_for_the_root() {
    // `config` declares no default, so `zzz` is just unexpected there.
    let e = parse::<RootOnly>(&["config", "zzz"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token()),
        (ErrorKind::UnexpectedArg, Some("zzz"))
    );
    assert!(matches!(
        parse::<RootOnly>(&["config", "ls", "x"]).unwrap().command,
        Some(RootOnlyCommand::Config(ConfigArgs {
            command: Some(ConfigCommand::Ls(LsArgs { what: Some(_) }))
        }))
    ));
}

#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct Help {
    #[arg(short, long, global)]
    verbose: bool,
    #[arg(subcommand)]
    command: Option<HelpCommand>,
}

#[derive(Subcommand, Debug)]
enum HelpCommand {
    Run(RunHelp),
}

#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct RunHelp {
    #[arg(positional)]
    task: Option<String>,
}

#[test]
fn a_bare_invocation_asks_for_help() {
    assert_eq!(
        parse::<Help>(&[]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    // A global flag before the subcommand is not the subcommand's argument.
    assert_eq!(
        parse::<Help>(&["-v", "run"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    let help: Help = parse(&["-v", "run", "x"]).unwrap();
    assert!(help.verbose);
    assert!(matches!(
        help.command,
        Some(HelpCommand::Run(RunHelp { task: Some(_) }))
    ));
}

/// A program whose usual command is its default one: `em -pu @world`.
#[derive(Args, Debug, PartialEq)]
#[arg(default_subcommand = "emerge", default_subcommand_flags, version = "1")]
struct Em {
    #[arg(long, global, default = "auto")]
    color: String,
    #[arg(short, long, global, count)]
    verbose: u8,
    #[arg(subcommand)]
    applet: Option<Applet>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Applet {
    Emerge(Emerge),
    Query(Query),
}

#[derive(Args, Debug, PartialEq, Default)]
struct Emerge {
    #[arg(short, long)]
    pretend: bool,
    #[arg(short, long)]
    update: bool,
    #[arg(long)]
    jobs: Option<u32>,
    #[arg(positional)]
    atoms: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
struct Query {
    #[arg(positional)]
    what: Vec<String>,
}

fn emerge(line: &[&str]) -> (Em, Emerge) {
    let mut em: Em = parse(line).unwrap();
    match em.applet.take() {
        Some(Applet::Emerge(emerge)) => (em, emerge),
        other => panic!("{line:?} is not an emerge: {other:?}"),
    }
}

#[test]
fn a_flag_the_root_does_not_take_selects_the_default_subcommand() {
    let (_, e) = emerge(&["-pu", "@world"]);
    assert_eq!(
        (e.pretend, e.update, &e.atoms[..]),
        (true, true, &["@world".to_owned()][..])
    );
    let (_, e) = emerge(&["--jobs", "4", "pkg"]);
    assert_eq!((e.jobs, e.atoms.len()), (Some(4), 1));
    let (_, e) = emerge(&["--jobs=4"]);
    assert_eq!(e.jobs, Some(4));
    // A flag alone is enough.
    assert!(emerge(&["-p"]).1.pretend);
}

#[test]
fn the_root_s_own_flags_come_before_or_among_them() {
    let (em, e) = emerge(&["--color=never", "-p", "x"]);
    assert_eq!((em.color.as_str(), e.pretend), ("never", true));
    // After the default subcommand is selected, a global is still the root's.
    let (em, e) = emerge(&["--pretend", "--color", "never", "y"]);
    assert_eq!(
        (em.color.as_str(), e.pretend, e.atoms.len()),
        ("never", true, 1)
    );
    // One word of letters of both: each is read once.
    let (em, e) = emerge(&["-vp", "x"]);
    assert_eq!((em.verbose, e.pretend), (1, true));
    let (em, e) = emerge(&["-pv", "x"]);
    assert_eq!((em.verbose, e.pretend), (1, true));
    let (em, e) = emerge(&["-vpv"]);
    assert_eq!((em.verbose, e.pretend), (2, true));
}

#[test]
fn every_later_word_is_the_default_subcommand_s() {
    // `query` after a flag is a package named query, not the applet.
    let (_, e) = emerge(&["-p", "query", "a"]);
    assert_eq!(e.atoms, ["query", "a"]);
    // Named first, an applet keeps its own flags and takes none of emerge's.
    let em: Em = parse(&["query", "a"]).unwrap();
    assert_eq!(
        em.applet,
        Some(Applet::Query(Query {
            what: vec!["a".into()]
        }))
    );
    let e = parse::<Em>(&["query", "-p"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnknownFlag);
}

#[test]
fn a_flag_nothing_takes_is_reported_as_the_default_subcommand_s() {
    let e = parse::<Em>(&["--pretnd", "x"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnknownFlag);
    assert_eq!(e.similar(), Some("--pretend"));
    let e = parse::<Em>(&["--bogus"]).unwrap_err();
    assert_eq!((e.kind(), e.similar()), (ErrorKind::UnknownFlag, None));
    // The root's own requests are still the root's.
    assert_eq!(
        parse::<Em>(&["--help"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    assert_eq!(
        parse::<Em>(&["-V"]).unwrap_err().kind(),
        ErrorKind::VersionRequested
    );
    // With no word at all, nothing is selected.
    assert_eq!(parse::<Em>(&[]).unwrap().applet, None);
}

#[test]
fn a_flag_after_the_one_that_selected_it_is_the_subcommand_s_too() {
    // A global with a value, in the word that hands the line over.
    let (em, e) = emerge(&["-pv", "--color=never"]);
    assert_eq!(
        (em.color.as_str(), em.verbose, e.pretend),
        ("never", 1, true)
    );
    // `--jobs` is emerge's, attached or not, wherever it comes.
    let (_, e) = emerge(&["-p", "--jobs", "2", "x"]);
    assert_eq!((e.pretend, e.jobs), (true, Some(2)));
}

/// `+x` options select the default subcommand as `-x` ones do.
#[derive(Args, Debug, PartialEq)]
#[arg(plus_options, default_subcommand = "set", default_subcommand_flags)]
struct Shell {
    #[arg(short, global)]
    quiet: bool,
    /// The root's alone: it goes before what selects the subcommand.
    #[arg(short)]
    login: bool,
    #[arg(subcommand)]
    command: Option<ShellCommand>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum ShellCommand {
    Set(SetArgs),
}

#[derive(Args, Debug, PartialEq)]
#[arg(plus_options)]
struct SetArgs {
    #[arg(short = 'x', plus = 'x')]
    trace: Option<bool>,
}

#[test]
fn a_plus_option_selects_it_and_the_root_s_own_flags_go_first() {
    let set = |trace| Some(ShellCommand::Set(SetArgs { trace }));
    assert_eq!(parse::<Shell>(&["+x"]).unwrap().command, set(Some(false)));
    assert_eq!(parse::<Shell>(&["-x"]).unwrap().command, set(Some(true)));
    let shell: Shell = parse(&["-lx"]).unwrap();
    assert_eq!((shell.login, shell.command), (true, set(Some(true))));
    // After it, a flag of the root that is not global is unknown to the subcommand.
    let e = parse::<Shell>(&["-xl"]).unwrap_err();
    assert_eq!((e.kind(), e.token()), (ErrorKind::UnknownFlag, Some("-l")));
    // A global one is taken wherever it is.
    let shell: Shell = parse(&["+x", "-q"]).unwrap();
    assert_eq!((shell.quiet, shell.command), (true, set(Some(false))));
}
