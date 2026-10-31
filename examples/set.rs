//! bash's `set`, whose options apply in the order they are given: `-e` and
//! `+e`, `-u` and `+u`, `-x` and `+x`, and `-o NAME` and `+o NAME`, in one
//! `sequence`. Each option is one variant, holding the sign of its `-` and
//! `+` spellings, so a later spelling undoes an earlier one, a letter or a
//! name alike.
//!
//!     cargo run --example set -- -o noglob +o noglob          # noglob off
//!     cargo run --example set -- -e -o nounset +o errexit     # errexit off, nounset on
//!     cargo run --example set -- -o errexit +e -x a b         # errexit off, xtrace on
//!     cargo run --example set -- -eo pipefail                 # a bundle, then the name
//!     cargo run --example set -- -o noglob -o                 # set it, then list
//!     cargo run --example set -- -oe                          # list, then errexit on
//!     cargo run --example set -- -h

use std::collections::BTreeMap;

use winnow_args::{Args, Occurrence};

/// Set or unset shell options.
#[derive(Args, Debug)]
#[arg(name = "set", plus_options)]
struct Set {
    #[arg(sequence)]
    options: Vec<Item>,
    /// The positional parameters.
    #[arg(positional, double_dash = "preserve", stop_flags)]
    args: Vec<String>,
}

/// One option, `true` where it is set (`-`) and `false` where it is unset
/// (`+`), in command-line order.
#[derive(Occurrence, Debug)]
enum Item {
    /// Exit when a command fails.
    #[arg(short = 'e', plus = 'e')]
    Errexit(bool),
    /// Treat unset variables as an error.
    #[arg(short = 'u', plus = 'u')]
    Nounset(bool),
    /// Print commands as they run.
    #[arg(short = 'x', plus = 'x')]
    Xtrace(bool),
    /// Set or unset the option NAME; alone, list the options.
    #[arg(
        short = 'o',
        plus = 'o',
        value_name = "NAME",
        default_missing = "",
        detached_only
    )]
    Named(bool, String),
}

fn main() {
    let set = Set::parse();
    let mut options: BTreeMap<&str, bool> = ["errexit", "noglob", "nounset", "pipefail", "xtrace"]
        .map(|name| (name, false))
        .into();
    for item in &set.options {
        let (name, on) = match item {
            Item::Errexit(on) => ("errexit", *on),
            Item::Nounset(on) => ("nounset", *on),
            Item::Xtrace(on) => ("xtrace", *on),
            // Alone: `-o` lists the options, `+o` lists them as commands.
            Item::Named(set, name) if name.is_empty() => {
                for (name, on) in &options {
                    if *set {
                        println!("{name:<15}{}", if *on { "on" } else { "off" });
                    } else {
                        println!("set {}o {name}", if *on { '-' } else { '+' });
                    }
                }
                continue;
            }
            Item::Named(on, name) => (name.as_str(), *on),
        };
        match options.get_mut(name) {
            Some(state) => *state = on,
            None => {
                // As bash, stop at an unknown name: what follows is not applied.
                eprintln!("set: {name}: invalid option name");
                break;
            }
        }
    }
    let on: Vec<_> = options
        .iter()
        .filter(|(_, on)| **on)
        .map(|(name, _)| *name)
        .collect();
    println!("on: {}", on.join(" "));
    println!("parameters: {:?}", set.args);
}
