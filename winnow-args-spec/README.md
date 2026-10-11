# winnow-args-spec

Reads the description a `winnow-args` derive writes of each type, and joins
them into the command tree of a program. `winnow-args-man` and
`winnow-args-markdown` render that tree; this crate is for a renderer of
your own.

```rust,no_run
use std::path::Path;
use winnow_args_spec::{Catalog, pages};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Written by `WINNOW_ARGS_SPEC=/abs/target/spec cargo check`.
    let catalog = Catalog::load(Path::new("target/spec"))?;
    let command = catalog.stitch(&catalog.root()?)?;
    // The root, then each visible subcommand at every depth.
    for page in pages(&command, "tool")? {
        println!("{}: {}", page.stem(), page.command.usage(&page.path));
    }
    Ok(())
}
```

## Writing the fragments

Check the program with `WINNOW_ARGS_SPEC` set to the absolute path of a
directory:

```sh
WINNOW_ARGS_SPEC=$PWD/target/spec cargo check
```

Each derive then writes one TOML file, a fragment, for its type:

```text
target/spec/
  tool-bin/src/main.rs/Cli.toml       the binary `tool`
  tool/src/cli/add.rs/Opts.toml       its library
  dep/src/lib.rs/Mode.toml            a dependency that derives too
```

Nothing is linked or run, so `--target` works for a target that is only
installed. Cargo compiles the deriving crates again when the variable
changes, so a new directory holds exactly what the program has now. Use a
directory Cargo does not watch, such as one under `target/`.

## From fragments to a tree

A fragment does not copy the fields of a type it flattens or takes as a
subcommand. It records the type's path as the source writes it, `add::Opts`,
and [`Catalog::stitch`] follows it. A command's own flags come first, then
those of each flatten, in the order of the source; subcommands likewise.

A path leads to a type this way:

1. A bare name is first read through the `use` lines of its module:
   with `use std::net::IpAddr;`, a field's `IpAddr` is `std::net::IpAddr`.
   `self` and `super` are replaced by the modules they stand for.
2. The modules written before the name must be among the type's own, in
   order. `add::Opts` and `cli::add::Opts` both lead to the `Opts` of
   `src/cli/add.rs`. So does `cli::Opts`, unless `cli` has an `Opts` itself.
3. A path that starts with a crate's name leads into that crate; one that
   starts with `std` leads to no type of the program.
4. The crate of the field is searched before the others.
5. A bare name that several types still answer to is the one of the same
   file.

When that leaves more than one type, nothing is guessed: the error lists
the types the name may be.

## Saying which type is meant

`#[arg(spec = "AddOpts")]` on a type files its fragment under that name. The
same attribute on a field or a variant that holds the type looks for it
there. It is needed for:

- two types of one name in one file;
- a type named through an alias, or through a dependency renamed in
  `Cargo.toml`;
- a `super::` that reaches the crate's root, or `crate::Opts` where the
  crate has several `Opts`.

The derive checks the same rules when the variable is set. A field whose
path does not lead to its type, or that leaves out the `spec` its type has,
fails to compile at the field, so a page never describes another type.

## A value's choices

The choices of a `ValueEnum` are filed under its name and found from the
field's type. They are not found for a type named through an alias, or for
one that implements `FromArg` by hand: the build with the variable set stops
at that field until it states `spec = "Name"` or `choices(...)`.

`winnow_args::complete::Shell` is known without a fragment.

## Limits

- A `use` inside a function is read as if it were its module's.
- A type that implements `Args` by hand writes no fragment. One written by
  hand in the same directory stands for it.
- A bare name that no `use` accounts for (a glob import) is refused when a
  `ValueEnum` of another file has that name. Import it by name, write its
  path, or state `spec`.

[`Catalog::stitch`]: https://docs.rs/winnow-args-spec/latest/winnow_args_spec/struct.Catalog.html#method.stitch
