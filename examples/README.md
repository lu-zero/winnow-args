# Examples

The programs are Cargo examples of the `winnow-args` package.

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
