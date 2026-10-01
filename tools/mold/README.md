# mold's command line on winnow-args

`apply.sh MOLD_DIR` regenerates mold's `--features winnow-args` parser from
the option chain in `MOLD_DIR/src/cmdline.rs` (as on mold's `main`):

- `moldgen.py` reads the chain: each `if … else if` arm's matchers
  (`read_flag`, `read_arg!`, `read_eq!`, `read_switch`, the `-z` forms, the LTO
  table) and its body;
- `moldgen2.py` turns every spelling into a variant of an `Occurrence` enum
  (`Item`), the `-z` keywords into `z_opt`, and the arms into one `match`
  over the parsed sequence, each arm running mold's own body;
- `mold_template.rs` is the rest of `src/cmdline_winnow.rs`.

The built-in parser stays, compiled out by the feature statement by statement.
