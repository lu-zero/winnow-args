# winnow-args-spec

Reads the TOML fragments a `winnow-args` derive writes and stitches them into
one command tree. Flattened commands and subcommands stay links in the files;
[`Catalog::stitch`](https://docs.rs/winnow-args-spec) follows those links and
keeps declaration order.

Set `WINNOW_ARGS_SPEC` to a directory Cargo does not watch, such as
`target/spec`, while building the program. Each crate is written under its
own subdirectory; load that subdirectory here. Markdown and man pages are
rendered by `winnow-args-markdown` and `winnow-args-man`.
