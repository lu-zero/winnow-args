//! Running the selected command.
//!
//! Parsing gives a tree of types: a struct of the program's flags, an enum of
//! its subcommands, and a struct for each of those. The program finds the
//! selected variant and calls the code that carries it out. [`Run`] is that
//! call. `#[arg(run)]` on a `Subcommand` enum writes the `match`.
//!
//! Each command implements the trait:
//!
#![cfg_attr(feature = "derive", doc = "```")]
#![cfg_attr(not(feature = "derive"), doc = "```ignore")]
//! use winnow_args::run::Run;
//! use winnow_args::{Args, Subcommand};
//!
//! #[derive(Args)]
//! struct Cli {
//!     #[arg(short, long, global)]
//!     verbose: bool,
//!     #[arg(subcommand)]
//!     command: Command,
//! }
//!
//! #[derive(Subcommand)]
//! #[arg(run)]
//! enum Command {
//!     Add(Add),
//!     Remove(Remove),
//! }
//!
//! #[derive(Args)]
//! struct Add {
//!     #[arg(positional)]
//!     name: String,
//! }
//!
//! #[derive(Args)]
//! struct Remove {
//!     #[arg(positional)]
//!     name: String,
//! }
//!
//! /// What every command is handed: here, the flags of the program.
//! struct Context {
//!     verbose: bool,
//! }
//!
//! impl Run<&Context> for Add {
//!     type Output = String;
//!
//!     fn run(self, context: &Context) -> String {
//!         match context.verbose {
//!             true => format!("adding {}, at length", self.name),
//!             false => format!("adding {}", self.name),
//!         }
//!     }
//! }
//!
//! impl Run<&Context> for Remove {
//!     type Output = String;
//!
//!     fn run(self, _: &Context) -> String {
//!         format!("removing {}", self.name)
//!     }
//! }
//!
//! let cli = Cli::parse_from(["-v", "add", "x"])?;
//! let context = Context { verbose: cli.verbose };
//! assert_eq!(cli.command.run(&context), "adding x, at length");
//! # Ok::<(), winnow_args::Error>(())
//! ```
//!
//! The context is whatever the commands share: a reference to the parsed
//! program, a connection, nothing at all (`()`, the default). The enum
//! implements the trait for every context all of its commands take, so a
//! struct that holds a nested enum of subcommands implements it by calling
//! `self.command.run(context)`, and the nesting runs all the way down.
//!
//! [`RunAsync`] and `#[arg(run_async)]` are the same for commands that are
//! awaited.
//!
//! None of this is on the path of a parse: it is a `match` in the program,
//! written by the derive rather than by hand.

/// A command that can be carried out, given `C`.
pub trait Run<C = ()> {
    /// What running the command produces.
    type Output;

    /// Carry out the command.
    fn run(self, context: C) -> Self::Output;
}

/// A command that is awaited: [`Run`], as a future.
///
/// An implementation writes `async fn run_async(self, context: C)`. The
/// signature here is `-> impl Future` so that the trait implies no `Send`
/// bound either way: whether a command's future is `Send` is decided by what
/// the command does.
pub trait RunAsync<C = ()> {
    /// What running the command produces, once awaited.
    type Output;

    /// Carry out the command.
    fn run_async(self, context: C) -> impl core::future::Future<Output = Self::Output>;
}
