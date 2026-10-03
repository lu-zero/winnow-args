# Contributing

The tasks are recipes of [`just`](https://github.com/casey/just); `just --list`
shows them all.

```
cargo test             # the library: unit, integration and doctests
just check             # every feature set and profile, clippy, docs, tests
just perf [argv...]    # parse cost against usage, bpaf and clap
just bench-examples    # warm instructions per parse, the two examples
just gen --help        # regenerate the mold port, the examples, the mise shadow
```

`just check` passes before a commit. A change to the parse path is measured
before and after, and the numbers go to [`docs/PERF.md`](./docs/PERF.md); a
feature costs nothing to a struct that does not use it.

The conventions (style, comments, commit messages, performance posture) are in
[`AGENTS.md`](./AGENTS.md), written for people and coding agents alike.
