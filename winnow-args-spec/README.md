# winnow-args-spec

Reads the TOML fragments a `winnow-args` derive writes and stitches them into
one command tree. Flattened commands and subcommands stay links in the files;
[`Catalog::stitch`](https://docs.rs/winnow-args-spec) follows those links and
keeps declaration order.

Set `WINNOW_ARGS_SPEC` to the absolute path of a directory Cargo does not
watch, such as one under `target/`, while checking the program. Cargo compiles
the crates that derive again when the value changes, so a new directory holds
exactly what the program has now. Each crate is written under its own
subdirectory, a binary's as `CRATE-bin`, and a fragment is filed under its
type's source file and name. Markdown and man pages are rendered by
`winnow-args-markdown` and `winnow-args-man`.

A fragment says which modules its type is in: those of its source file
(`cli/add.rs` and `cli/add/mod.rs` are `cli::add`), then the `mod name { }`
blocks written around it in that file. A name in a fragment is the type's path
as the source writes it, and it leads to a type whose modules have the path's
among them, one after the other: `add::Opts`, `cli::add::Opts`, and
`cli::Opts` where `cli` brings it out with a `pub use`. A path that starts
with a crate's name leads into that crate, and `std::net::IpAddr` leads to no
`IpAddr` of the program. The type is looked for in the same crate, then in
the others, and where several are left a bare name is the type of the same
file. A root is named the same way.

The derive holds each field to the same rule when the documentation is
built: a path that does not lead to its type, through an alias or a module
that goes by another name, fails to compile there. So a path that leads to a
type in the fragments is that type's path.

A `use` can bring a dependency's type under a bare name, which the derive
cannot see. So where the crate and a dependency both have a type of the name,
the crate's own is meant only in its own file, by its module, or as
`crate::Type`; any other spelling is refused with the two it may be.

When that does not tell two types apart (two of one name in one file, a
bare name several types answer to), the stitching says which types the name
may be. `#[arg(spec = "AddOpts")]` on a type files it under a name of its own,
and the same attribute on each field or variant that holds the type finds it
there. It is also what a field states for a type it names through an alias.

A page's rows are those of `--help`. What help does not say is a sentence of
the page: under `long_only`, that a long option may also be spelled with one
dash.

The derive names a value's type, and finds its choices under that name. A type
named through an alias, or one that implements `FromArg` by hand, has choices
the name does not lead to: the documentation build stops there, and `spec`
(the name the choices are filed under) or `choices` on the field states them. `winnow_args::complete::Shell` is known. One case is left to the
reader: a type with no fixed choices, written as a bare name that a
`ValueEnum` of the build also has (`use std::net::IpAddr` beside an enum
`IpAddr` in another module), is given that enum's choices. Its path written
out, or a `spec` that names nothing, says it is not that enum. A type that implements `Args` by hand writes no
fragment; one written by hand beside the others stands for it.
