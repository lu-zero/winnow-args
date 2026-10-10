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
type's name: two types of one target cannot share a name. A name in a fragment
is the type of that name in the same crate, else the only one in the others;
`CRATE::Type` names the root when two crates have one. Markdown and man pages are rendered by
`winnow-args-markdown` and `winnow-args-man`.

What the fragments do not hold: a type that implements `Args` or `FromArg` by
hand writes none, so its flags or its choices are missing, and so are the
choices of a field declared through a type alias, which the derive cannot see
through. A program whose top level is a `Subcommand` enum has no command to
start from.

Under `long_only`, a page lists the one-dash spelling of each long name beside
the two-dash one. `--help` shows only the latter.
