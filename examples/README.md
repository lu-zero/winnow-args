# Examples

The programs are Cargo examples of the `winnow-args` package. The justfile
and the xtask are workspace crates that show how to generate their
documentation.

## Programs

The sources are in this directory. `winnow-args` names each one with
`path = "../examples/…"`, because Cargo would otherwise only look next to
that package.

| Source | Run |
|---|---|
| [`example.rs`](example.rs) | `cargo run --example example -- -vp /tmp` |
| [`help.rs`](help.rs) | `cargo run --example help -- --help` |
| [`brush_builtins.rs`](brush_builtins.rs) | `cargo run --example brush_builtins -- set -eu +x -o pipefail a b` |
| [`ld.rs`](ld.rs) | `cargo run --example ld -- -shared -o out.so --as-needed -lc a.o -z now` |

`brush_builtins.rs` and `ld.rs` are generated. Regenerate them with
`just gen examples BRUSH_DIR MOLD_DIR`. Do not edit them by hand.

## Documentation

A derived command becomes markdown and man pages in two steps. The derive
writes one TOML file per type when `WINNOW_ARGS_SPEC` is set. A second
program loads that directory, stitches the command nothing else names, and
renders pages. Cargo compiles a crate that derives again when that variable
changes, so the generator names a new directory each run, and keeps its own
target directory so that your ordinary builds are not the ones recompiled.

### Justfile

[`just-docs/`](just-docs/) is a small `hello` program and a justfile. This
is the shape for a repository that uses just. From the workspace root:

```
just -f examples/just-docs/justfile          # lists docs
just -f examples/just-docs/justfile docs     # pages in target/docs/just-docs/
```

The recipe calls the xtask crate below.

### xtask

[`xtask-docs/`](xtask-docs/) is the shape brush uses. Brush has no justfile;
its tasks are `cargo xtask`, and documentation would be `cargo xtask gen docs`.

```
cargo run -p xtask-docs -- gen docs
cargo run -p xtask-docs -- gen docs --example brush_builtins
cargo run -p xtask-docs -- gen docs --package just-docs
cargo run -p xtask-docs -- gen docs --target aarch64-unknown-linux-gnu
cargo run -p xtask-docs -- gen docs --root declare=DeclareCommand --root set=SetCommand
```

With no arguments it documents `brush_builtins`. Pages land in
`target/docs/<name>/`, markdown under `md/` and man pages under `man/`.
`--out` changes that directory; its `md` and `man` subdirectories are emptied
first, so a page never outlives its command. `--target` checks the crate for that triple
first, so the pages match its `cfg`. The crate is not linked, so the target
needs no cross toolchain, only `rustup target add`.

A crate with several commands, as brush's builtins are, has no single
command that nothing else names. `--root` names each type to document, and
may repeat. `NAME=TYPE` also gives the word the pages use for a command
that has no name of its own, and `CRATE::TYPE` picks the type of one crate
when a dependency has one of the same name.
