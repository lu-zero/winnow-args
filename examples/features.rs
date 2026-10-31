//! A build tool's features, turned on by `-F NAME` and off by `+F NAME`, as a
//! `Vec<(bool, String)>`: one field keeps both spellings in the order given,
//! so the last one for each name wins. A comma separates several names, each
//! turned on or off as its flag says.
//!
//!     cargo run --example features -- -F simd +F simd -F lto  # lto
//!     cargo run --example features -- +F std -F std           # std
//!     cargo run --example features -- -Fsimd,lto +Fsimd       # lto: an attached value
//!     cargo run --example features -- -h

use std::collections::BTreeSet;

use winnow_args::Args;

/// Build with features.
#[derive(Args, Debug)]
#[arg(name = "features", plus_options)]
struct Features {
    /// Turn a feature on (`-F`) or off (`+F`); a comma separates several.
    #[arg(short = 'F', plus = 'F', value_name = "NAME", delimiter = ',')]
    features: Vec<(bool, String)>,
    /// Print what would be built.
    #[arg(short = 'n', long)]
    dry_run: bool,
}

fn main() {
    let cli = Features::parse();
    let mut on = BTreeSet::from(["std"]);
    for (enable, name) in &cli.features {
        if *enable {
            on.insert(name.as_str());
        } else {
            on.remove(name.as_str());
        }
    }
    let on: Vec<_> = on.into_iter().collect();
    let verb = if cli.dry_run {
        "would build"
    } else {
        "building"
    };
    println!("{verb} with: {}", on.join(" "));
}
