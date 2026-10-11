//! `#[arg(run)]` and `#[arg(run_async)]`: the enum of subcommands runs the
//! command it holds.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use winnow_args::run::{Run, RunAsync};
use winnow_args::{Args, Subcommand};

/// What the commands share.
struct Shared {
    verbose: bool,
}

#[derive(Args, Debug)]
struct Cli {
    #[arg(short, long, global)]
    verbose: bool,
    #[arg(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[arg(run)]
enum Command {
    Add(Add),
    /// Held in a box, and run all the same.
    Remove(Box<Remove>),
    Remote(Remote),
}

#[derive(Args, Debug)]
struct Add {
    #[arg(positional)]
    name: String,
}

#[derive(Args, Debug)]
struct Remove {
    #[arg(positional)]
    name: String,
}

/// A command with commands of its own.
#[derive(Args, Debug)]
struct Remote {
    #[arg(subcommand)]
    command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
#[arg(run)]
enum RemoteCommand {
    Prune(Prune),
}

#[derive(Args, Debug)]
struct Prune {
    #[arg(long)]
    dry_run: bool,
}

impl Run<&Shared> for Add {
    type Output = String;

    fn run(self, shared: &Shared) -> String {
        format!("add {} {}", self.name, shared.verbose)
    }
}

impl Run<&Shared> for Remove {
    type Output = String;

    fn run(self, _: &Shared) -> String {
        format!("remove {}", self.name)
    }
}

impl<'a> Run<&'a Shared> for Remote {
    type Output = String;

    fn run(self, shared: &'a Shared) -> String {
        format!("remote {}", self.command.run(shared))
    }
}

impl Run<&Shared> for Prune {
    type Output = String;

    fn run(self, _: &Shared) -> String {
        format!("prune {}", self.dry_run)
    }
}

fn ran(line: &[&str]) -> String {
    let cli = Cli::parse_from(line).unwrap();
    let shared = Shared {
        verbose: cli.verbose,
    };
    cli.command.run(&shared)
}

#[test]
fn the_enum_runs_the_command_it_holds() {
    assert_eq!(ran(&["add", "x"]), "add x false");
    assert_eq!(ran(&["-v", "add", "x"]), "add x true");
    assert_eq!(ran(&["remove", "y"]), "remove y");
    // A command's own subcommands are run by the same call, one level down.
    assert_eq!(ran(&["remote", "prune", "--dry-run"]), "remote prune true");
}

/// Commands that take nothing: the context is `()`.
#[derive(Subcommand, Debug)]
#[arg(run)]
enum Plain {
    One(One),
    Two(Two),
}

#[derive(Args, Debug)]
struct One {}

#[derive(Args, Debug)]
struct Two {
    #[arg(long)]
    twice: bool,
}

impl Run for One {
    type Output = u8;

    fn run(self, (): ()) -> u8 {
        1
    }
}

impl Run for Two {
    type Output = u8;

    fn run(self, (): ()) -> u8 {
        match self.twice {
            true => 4,
            false => 2,
        }
    }
}

#[derive(Args, Debug)]
struct PlainCli {
    #[arg(subcommand)]
    command: Plain,
}

#[test]
fn a_command_may_take_nothing() {
    let run = |line: &[&str]| PlainCli::parse_from(line).unwrap().command.run(());
    assert_eq!(run(&["one"]), 1);
    assert_eq!(run(&["two", "--twice"]), 4);
}

#[derive(Subcommand, Debug)]
#[arg(run_async)]
enum Awaited {
    Fetch(Fetch),
    Push(Box<Push>),
}

#[derive(Args, Debug)]
struct Fetch {
    #[arg(positional)]
    from: String,
}

#[derive(Args, Debug)]
struct Push {
    #[arg(long)]
    force: bool,
}

impl RunAsync<&Shared> for Fetch {
    type Output = Result<String, String>;

    async fn run_async(self, shared: &Shared) -> Self::Output {
        std::future::ready(()).await;
        Ok(format!("fetch {} {}", self.from, shared.verbose))
    }
}

impl RunAsync<&Shared> for Push {
    type Output = Result<String, String>;

    async fn run_async(self, _: &Shared) -> Self::Output {
        match self.force {
            true => Ok("push".to_owned()),
            false => Err("refused".to_owned()),
        }
    }
}

#[derive(Args, Debug)]
struct AwaitedCli {
    #[arg(subcommand)]
    command: Awaited,
}

/// These futures never wait on anything: one poll finishes them.
fn finish<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("a future that waits on nothing was pending"),
    }
}

#[test]
fn an_awaited_command_is_awaited() {
    let shared = Shared { verbose: true };
    let run = |line: &[&str]| {
        let cli = AwaitedCli::parse_from(line).unwrap();
        finish(cli.command.run_async(&shared))
    };
    assert_eq!(
        run(&["fetch", "origin"]),
        Ok("fetch origin true".to_owned())
    );
    assert_eq!(run(&["push"]), Err("refused".to_owned()));
    assert_eq!(run(&["push", "--force"]), Ok("push".to_owned()));
}

/// Two words for one command: both variants hold its type, one in a box.
#[derive(Subcommand, Debug)]
#[arg(run)]
enum Twice {
    Add(Add),
    #[arg(name = "plus")]
    AddToo(Box<Add>),
}

#[derive(Args, Debug)]
struct TwiceCli {
    #[arg(subcommand)]
    command: Twice,
}

#[test]
fn two_variants_may_hold_one_type() {
    let shared = Shared { verbose: false };
    let run = |line: &[&str]| TwiceCli::parse_from(line).unwrap().command.run(&shared);
    assert_eq!(run(&["add", "x"]), "add x false");
    assert_eq!(run(&["plus", "y"]), "add y false");
}

/// An enum with parameters of its own, a default and a `where` among them.
#[derive(Subcommand, Debug)]
#[arg(run)]
enum Generic<T: Default + std::fmt::Debug + 'static = u8>
where
    T: Clone,
{
    Hold(Holder<T>),
}

#[derive(Debug)]
struct Holder<T>(T);

impl<T: Default + 'static> Args for Holder<T> {
    const HELP: &'static winnow_args::help::Command = <One as Args>::HELP;

    fn parse_argv_with(
        _: &mut winnow_args::Argv<'_>,
        _: &mut dyn winnow_args::Globals,
    ) -> Result<Self, winnow_args::Error> {
        Ok(Holder(T::default()))
    }
}

impl<T: std::fmt::Debug> Run for Holder<T> {
    type Output = String;

    fn run(self, (): ()) -> String {
        format!("{:?}", self.0)
    }
}

#[derive(Args, Debug)]
struct GenericCli {
    #[arg(subcommand)]
    command: Generic,
}

#[test]
fn the_enum_s_own_parameters_are_kept() {
    let cli = GenericCli::parse_from(["hold"]).unwrap();
    assert_eq!(cli.command.run(()), "0");
}
