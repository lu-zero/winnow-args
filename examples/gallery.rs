//! Every item help renders, once each. Unlike `help.rs`, this is not meant
//! to look like a real tool: each description says which item it shows, so
//! that its pages are a reference for the rendering of each item.
//!
//!     cargo run --example gallery -- -h            # short help
//!     cargo run --example gallery -- --help        # long help
//!     cargo run --example gallery -- run --help    # a subcommand's help
//!     cargo run --example gallery -- shapes --help # flattened subcommands
//!     cargo run --example gallery -- brief --help  # `brief_help`: the short page
//!     cargo run --example gallery -- -V            # version
//!
//! Its pages are built with
//! `cargo run -p xtask-docs -- gen docs --example gallery`.

use std::path::PathBuf;

use winnow_args::{Args, Subcommand, ValueEnum, report};

/// A description with `code`, *emphasis* and **strong** marks
///
/// The long description. `--help` shows it whole, and `-h` only its first
/// paragraph.
#[derive(Args, Debug)]
#[arg(
    name = "gallery",
    version = "1.0.0",
    after_help = "The text after the lists, on `-h`.",
    after_long_help = "The text after the lists, on `--help`.",
    default_subcommand = "run",
    default_subcommand_help,
    plus_options,
    heading("Values", help = "The paragraph under a heading, on `--help` only."),
    example = "gallery -vx --jobs 2 run <INPUT> -o out.txt -- echo 'a b' # a comment",
    example = "gallery --json unit | jq .",
    example(
        "gallery shapes circle --radius 2",
        header = "An example with a `header`",
        help = "And a *help* line, on `--help` only."
    ),
    effect = "read",
    output("text", default, help = "The output form"),
    exit_code(0, "an exit status")
)]
struct Gallery {
    /// A count, `-vv` is two, and `global`
    #[arg(short, long, count, global)]
    verbose: u8,
    /// Two short letters
    #[arg(short = 'q', short = 's', long)]
    quiet: bool,
    /// An optional value, attached with `=`
    #[arg(long, default_missing = "always", require_equals)]
    color: Option<When>,
    /// A `negate` flag, with a default
    #[arg(long, negate, default = "true")]
    progress: bool,
    /// A `plus` flag: `-x` sets it, `+x` clears it
    #[arg(short = 'x', plus = 'x')]
    trace: Option<bool>,
    /// A flag with a `warning`
    #[arg(long, warning = "The warning, on `--help` only.")]
    unsafe_mode: bool,
    /// A `deprecated` flag
    #[arg(
        long,
        deprecated = "the reason, in the long help and the warning",
        deprecated_remove_at = "2.0"
    )]
    old: bool,
    /// Choices that say what they mean, one a line on `--help`
    #[arg(long)]
    speed: Option<Speed>,
    /// Choices written on the field, with what each means
    #[arg(long, choices("name" = "The first choice's meaning", "size"))]
    sort: Option<String>,
    /// Choices that help leaves out (`hide_choices`)
    #[arg(long, choices("a", "b"), hide_choices)]
    mode: Option<String>,
    /// A doc comment the pages show, where help shows a constant
    #[arg(long, help = HELP_FROM_A_CONSTANT)]
    made: bool,
    /// A flag listed first by `display_order`
    #[arg(long, display_order = 0)]
    first: bool,
    /// An `env` variable and a default
    #[arg(
        short,
        long,
        env = "GALLERY_JOBS",
        default = "4",
        help_heading = "Values"
    )]
    jobs: u32,
    /// A default and an `env` variable that help leaves out
    #[arg(
        long,
        env = "GALLERY_LEVEL",
        default = "3",
        hide_default_value,
        hide_env,
        help_heading = "Values"
    )]
    level: u32,
    /// Two values each time
    #[arg(long, values = 2, value_name = "N", help_heading = "Values")]
    point: Vec<i32>,
    /// One value or more each time
    #[arg(long, values = 1.., help_heading = "Values")]
    tag: Vec<String>,
    #[arg(flatten)]
    output: Output,
    #[arg(subcommand)]
    command: Option<Command>,
}

/// Flags of a flattened struct, under its own heading.
#[derive(Args, Debug)]
#[arg(
    next_help_heading = "Output",
    heading("Output", help = "The paragraph of a flattened struct's heading.")
)]
struct Output {
    /// A flattened flag that is `global`
    #[arg(long, global)]
    json: bool,
}

/// Possible values
#[derive(ValueEnum, Debug)]
enum When {
    Auto,
    Always,
    Never,
}

/// Possible values that say what they mean
#[derive(ValueEnum, Debug)]
enum Speed {
    /// A variant's doc comment
    Fast,
    /// Another
    Slow,
    /// A variant that says nothing is listed bare
    #[arg(help = "")]
    Steady,
}

/// A flag's help made by a constant, which may differ by target.
const HELP_FROM_A_CONSTANT: &str = if cfg!(unix) {
    "Help from a constant (`help = …`), here on unix"
} else {
    "Help from a constant (`help = …`)"
};

// Read only through `Debug`.
#[allow(dead_code)]
#[derive(Subcommand, Debug)]
enum Command {
    /// A subcommand that holds a struct, with an alias
    #[arg(alias = "r")]
    Run(Run),
    /// A subcommand whose subcommands are flattened into its page
    #[arg(flatten_help, example = "gallery shapes square 3")]
    Shapes {
        #[arg(subcommand)]
        shape: Shape,
    },
    /// A unit subcommand
    Unit,
    /// A `deprecated` subcommand
    #[arg(deprecated = "use `unit`")]
    Old,
    /// A subcommand whose `--help` is the short page (`brief_help`)
    Brief(Brief),
}

/// A command with `brief_help`: `--help` shows `-h`'s page.
///
/// This paragraph is on neither.
#[derive(Args, Debug)]
#[arg(brief_help)]
struct Brief {
    /// A flag
    ///
    /// Its long text, not on `--help`.
    #[arg(long)]
    flag: bool,
}

/// The default subcommand's page.
///
/// What its description adds to its line in the list, under the default
/// command's heading on `--help`.
#[derive(Args, Debug)]
struct Run {
    /// A required positional
    #[arg(positional)]
    input: PathBuf,
    /// A default made from another field
    #[arg(short, long, default_from(input, |input| input.with_extension("out")))]
    output: Option<PathBuf>,
    /// A default from a function, with a note
    #[arg(long, default_fn = cores, default_note = "one per core")]
    threads: usize,
    /// The words after `--`
    #[arg(positional, double_dash = "required")]
    command: Vec<String>,
}

fn cores() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

// Read only through `Debug`.
#[allow(dead_code)]
#[derive(Subcommand, Debug)]
enum Shape {
    /// A variant with fields
    Circle {
        /// A required flag
        #[arg(long)]
        radius: f64,
    },
    /// A variant with a positional
    Square {
        /// An optional positional
        #[arg(positional)]
        side: Option<f64>,
    },
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Some(answer) = Gallery::completion_request(&args) {
        print!("{answer}");
        return;
    }
    let parsed = Gallery::parse_from(&args);
    // What was deprecated: `Gallery::parse()` prints these itself.
    for warning in Gallery::warnings() {
        eprint!("{}", warning.render());
    }
    match parsed {
        Ok(gallery) => println!("{gallery:#?}"),
        // `-h`, `--help`, `help …` and `-V` end up here too.
        Err(e) => std::process::exit(report(&e, "gallery")),
    }
}
