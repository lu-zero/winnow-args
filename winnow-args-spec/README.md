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

A name in a fragment is the type's path as the source writes it. It is the
type of that name in the same crate, else the only one in the others. Where
several have the name, the path's modules say which file's is meant
(`add::Opts` is the one of `add.rs` or `add/mod.rs`), and a bare name is the
type of the same file. A root is named the same way, and `CRATE::Type` picks a
crate's.

When that does not tell two types apart (two of one name in one file, a
module brought in under another name), the stitching says which types the
name may be. `#[arg(spec = "AddOpts")]` on a type files it under a name of its
own, and the same attribute on each field or variant that holds the type
finds it there. A field that names a type other than as it is filed, through
an alias or without the `spec` its type states, fails the documentation
build, so a page never describes another type of that name.

A page's rows are those of `--help`. What help does not say is a sentence of
the page: under `long_only`, that a long option may also be spelled with one
dash.

The derive names a value's type, and finds its choices under that name. A type
named through an alias, or one that implements `FromArg` by hand, has choices
the name does not lead to: the documentation build stops there, and `spec`
(the name the choices are filed under) or `choices` on the field states them. A type that implements `Args` by hand writes no
fragment; one written by hand beside the others stands for it.
