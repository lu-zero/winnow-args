//! Field attributes used where they mean nothing.
use winnow_args::Args;

#[derive(Args)]
struct Unknown {
    #[arg(long, hdie)]
    verbose: bool,
}

#[derive(Args)]
struct DelimiterOnOne {
    #[arg(long, delimiter = ',')]
    path: String,
}

#[derive(Args)]
struct DefaultOnASwitch {
    #[arg(long, default = "true")]
    verbose: bool,
}

#[derive(Args)]
struct MissingDefaultOnASwitch {
    #[arg(long, default_missing = "x")]
    verbose: bool,
}

#[derive(Args)]
struct StopFlagsOnAFlag {
    #[arg(long, stop_flags)]
    path: String,
}

#[derive(Args)]
struct NegateOnAValue {
    #[arg(long, negate)]
    path: String,
}

#[derive(Args)]
struct PlusWithoutPlusOptions {
    #[arg(short = 'x', plus = 'x')]
    trace: Option<bool>,
}

#[derive(Args)]
struct HintOnASwitch {
    #[arg(long, hint = "dir")]
    verbose: bool,
}

#[derive(Args)]
struct HintOfNoKind {
    #[arg(long, hint = "folder")]
    path: String,
}

#[derive(Args)]
struct Tuple(bool);

fn names(_typed: &str) -> Vec<winnow_args::complete::Candidate> {
    Vec::new()
}

#[derive(Args)]
struct HintBesideAFunction {
    #[arg(long, hint = "dir", complete = names)]
    path: String,
}

#[derive(Args)]
struct FunctionOnACount {
    #[arg(short, count, complete = names)]
    verbose: u8,
}

#[derive(Args)]
struct HiddenChoicesOfNone {
    #[arg(long, hide_choices)]
    verbose: bool,
}

const HELP: &str = "help";

#[derive(Args)]
struct ConstantHelpBesideLongHelp {
    #[arg(long, help = HELP, long_help = "more")]
    flag: bool,
}

#[derive(Args)]
struct DeprecatedEnvOfNone {
    #[arg(long, deprecated_env = "OLD")]
    jobs: Option<u32>,
}

#[derive(winnow_args::Subcommand)]
enum Command {
    Run,
}

#[derive(Args)]
struct DeprecatedSubcommandField {
    #[arg(subcommand, deprecated)]
    command: Option<Command>,
}

#[derive(winnow_args::ValueEnum)]
enum DeprecatedValue {
    Fast,
    #[arg(deprecated)]
    Slow,
}

#[derive(Args)]
struct MaybeValuePositional {
    #[arg(positional)]
    name: Option<Option<String>>,
}

#[derive(Args)]
struct MaybeValueDefaultMissing {
    #[arg(long, default_missing = "always")]
    color: Option<Option<String>>,
}

#[derive(Args)]
struct GroupedPositional {
    #[arg(positional)]
    words: Vec<Vec<String>>,
}

#[derive(Args)]
struct EachWithDelimiter {
    #[arg(long, delimiter = ',')]
    define: Vec<Option<String>>,
}

#[derive(Args)]
struct ParseWithOnASwitch {
    #[arg(long, parse_with = str::parse::<bool>)]
    verbose: bool,
}

#[derive(Args)]
#[arg(plus_options)]
struct TristateWithValues {
    #[arg(short = 'x', plus = 'x', values = 2)]
    trace: Vec<bool>,
}

#[derive(Args)]
struct KeywordsOnEach {
    #[arg(long, keywords)]
    define: Vec<Option<String>>,
}

#[derive(Args)]
#[arg(plus_options)]
struct EachWithPlus {
    #[arg(short = 'x', plus = 'x')]
    trace: Vec<Option<bool>>,
}

#[derive(Args)]
#[arg(plus_options)]
struct SignedWithoutPlus {
    #[arg(short = 'o')]
    options: Vec<(bool, String)>,
}

#[derive(Args)]
struct DetachedWithEquals {
    #[arg(short = 'o', require_equals, detached_only)]
    option: Option<String>,
}

#[derive(Args)]
struct HeadingFromNowhere {
    #[arg(long, help_heading = env!("WINNOW_ARGS_NO_SUCH_VARIABLE"))]
    option: Option<String>,
}

#[derive(Args)]
struct HeadingFromNowhereSaid {
    #[arg(long, help_heading = env!("WINNOW_ARGS_NO_SUCH_VARIABLE", "set it in build.rs"))]
    option: Option<String>,
}

const HEADING: &str = "Output";

#[derive(Args)]
struct HeadingFromAConstant {
    #[arg(long, help_heading = HEADING)]
    option: Option<String>,
}

/// Joined by rustc only.
#[derive(Args)]
struct HelpFromConcat {
    #[doc = concat!("A ", "flag.")]
    #[arg(long)]
    flag: bool,
}

fn main() {}
