# winnow-args-spec

Reads the TOML fragments a `winnow-args` derive writes and stitches them into
one command tree. Flattened commands and subcommands stay links in the files;
[`Catalog::stitch`](https://docs.rs/winnow-args-spec) follows those links and
keeps declaration order.

Set `WINNOW_ARGS_SPEC` to the absolute path of a directory Cargo does not
watch, such as one under `target/`, while checking the program. Cargo compiles
the crates that derive again when the value changes, so a new directory holds
exactly what the program has now. Each crate is written under its own
subdirectory, and a fragment is filed under its type's name: two types of one
crate cannot share a name. Markdown and man pages are rendered by
`winnow-args-markdown` and `winnow-args-man`.
