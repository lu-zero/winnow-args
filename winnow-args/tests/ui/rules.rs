//! Selectors and the rules that use them.
use winnow_args::Args;

#[derive(Args)]
struct NoSuchFlag {
    #[arg(long, conflicts("--quiet"))]
    verbose: bool,
}

#[derive(Args)]
struct TheNegation {
    #[arg(long, negate)]
    cache: bool,
    #[arg(long, requires("--no-cache"))]
    fresh: bool,
}

#[derive(Args)]
struct NoSuchGroup {
    #[arg(long, group = "output")]
    json: bool,
}

#[derive(Args)]
struct DefaultIfArity {
    #[arg(long, default_if("--json"))]
    color: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DefaultIfOnACount {
    #[arg(short, count, default_if("--json", "1"))]
    verbose: u8,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DefaultIfComparesACount {
    #[arg(long, default_if("-v", "2", "never"))]
    color: Option<String>,
    #[arg(short, count)]
    verbose: u8,
}

#[derive(Args)]
struct SwitchDefaultIf {
    #[arg(long, default_if("--json", "yes"))]
    quiet: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RequiredIfCount {
    #[arg(short, count)]
    verbose: u8,
    #[arg(long, required_if("-v", "2"))]
    log: Option<String>,
}

#[derive(Args)]
struct RequiredIfNothing {
    #[arg(long, required_if("--nope"))]
    log: Option<String>,
}

#[derive(Args)]
struct RequiredIfItself {
    #[arg(long, required_if("--log", "x"))]
    log: Option<String>,
}

#[derive(Args)]
struct RequiredUnlessNothing {
    #[arg(long, required_unless_all())]
    log: Option<String>,
}

#[derive(Args)]
#[arg(validate = check, validate = check)]
struct ValidateTwice {
    #[arg(long)]
    log: Option<String>,
}

fn check(_: &ValidateTwice) -> Result<(), winnow_args::Error> {
    Ok(())
}

#[derive(Args)]
struct DefaultFromNothing {
    #[arg(long, default_from(nope, |n: &String| n.clone()))]
    out: String,
}

#[derive(Args)]
struct DefaultFromEachOther {
    #[arg(long, default_from(b, |b: &String| b.clone()))]
    a: String,
    #[arg(long, default_from(a, |a: &String| a.clone()))]
    b: String,
}

#[derive(Args)]
struct DefaultFromAndDefault {
    #[arg(long)]
    a: String,
    #[arg(long, default = "x", default_from(a, |a: &String| a.clone()))]
    b: String,
}

#[derive(Args)]
struct RequiresIfSwitch {
    #[arg(long, requires_if("yes", "--log"))]
    verbose: bool,
    #[arg(long)]
    log: Option<String>,
}

#[derive(Args)]
#[arg(group("output"))]
struct RequiresNoGroup {
    #[arg(long, requires = "ouptut")]
    export: bool,
    #[arg(long, group = "output")]
    json: bool,
}

fn main() {}
