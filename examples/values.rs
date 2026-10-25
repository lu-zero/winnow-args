//! Values and defaults, as an `mk` build tool has them: a `Vec` with two
//! defaults, a default made from another field, a variable named after the
//! field with another read after it, a type parsed by its `FromStr`, a
//! `bool_value` flag and a flag of two values.
//!
//!     cargo run --example values -- -h
//!     cargo run --example values                         # every default
//!     NPROC=8 cargo run --example values -- lib          # out is lib.out
//!     JOBS=4 NPROC=8 cargo run --example values -- lib   # JOBS first
//!     cargo run --example values -- --cache ::1 --wait=false --size 640 480

use std::net::Ipv6Addr;
use std::path::PathBuf;

use winnow_args::Args;

/// Build a project.
#[derive(Args, Debug)]
#[arg(name = "mk")]
struct Mk {
    /// Jobs at once.
    #[arg(short, long, env, env_fallback("NPROC"), default = "1")]
    jobs: u32,
    /// Features to enable.
    #[arg(long, default = "std", default = "derive")]
    features: Vec<String>,
    /// The source directory.
    #[arg(positional, default = "src")]
    source: PathBuf,
    /// Where the output goes.
    #[arg(short, long, default_from(source, |source| source.with_extension("out")))]
    out: Option<PathBuf>,
    /// The cache server, parsed by `FromStr`.
    #[arg(long, from_str)]
    cache: Option<Ipv6Addr>,
    /// Wait for the cache.
    #[arg(long, bool_value)]
    wait: bool,
    /// The image size.
    #[arg(long, value_names("W", "H"))]
    size: Vec<u32>,
}

fn main() {
    let mk = Mk::parse();
    println!(
        "jobs {}, features {:?}, source {:?}, out {:?}",
        mk.jobs, mk.features, mk.source, mk.out
    );
    println!("cache {:?}, wait {}, size {:?}", mk.cache, mk.wait, mk.size);
}
