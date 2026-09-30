# Checklist: mold and wild (the `ld` command line)

What a linker needs to parse GNU `ld`'s command line with winnow-args, read
from [mold](../../mold) (the Rust port: `src/cmdline.rs`, branch
`fix/cmdline-gnu-dash-rules`) and [wild](../../wild) (`libwild/src/args.rs`,
`libwild/src/args/elf.rs`).

Legend: `[x]` winnow-args covers it today (the feature that does is named),
`[ ]` not yet.

## How it is checked

- **Conformance, unit level**: mold's `ArgCursor` tests in `src/cmdline.rs`
  (`cursor_preserves_gnu_option_boundaries_and_values`,
  `a_single_dash_long_wins_over_a_short_with_an_attached_value`,
  `thinlto_index_only_takes_its_value_after_equals`,
  `cursor_borrows_non_utf8_separate_and_attached_values`, …) and wild's
  argument tests (`test_parse_inline_only_options`,
  `test_parse_mixed_file_and_inline_options`,
  `test_parse_recursive_file_option`, `test_arguments_from_string`,
  `test_ignored_flags`, `test_flavor`, the `-Ttext`/`-Tdata`/`-Tbss` round
  trips). Ported as winnow-args tests, they pin the grammar.
- **Conformance, end to end**: mold's 573 link tests (`tests/*.sh`); about 170
  are named after the options they exercise, cited below as `mold: name`.
  wild's integration tests (`wild/tests/`). Acceptance: a linker built on the
  winnow-args parser passes what its own parser passes.
- **Scale**: mold reads ~214 option spellings through its cursor (124 flags,
  34 `--x`/`--no-x` pairs, 42 `-z` keywords, 8 `-z` pairs, plus the value
  options it lists in tables); wild declares 158 for ELF. A linker parses once,
  but a command line can hold thousands of words through response files.
- **Speed**: parse a real link line (a rustc- or Chromium-sized response file)
  with mold's cursor, wild's parser and winnow-args; per-word throughput and
  allocations.

## Phase 1: GNU's dash rules

A lexer mode, since the default (usage's grammar) differs on every point here.

- [ ] Long options take one dash or two: `-shared` = `--shared`,
      `-soname x` = `--soname x`. (mold `match_option`, wild `strip_option`;
      `mold: shared`, `soname`)
- [ ] Names starting with `o` need two dashes: `-omagic` is `-o magic`.
      (`mold: omagic`, `nmagic`)
- [ ] A list of names needs two dashes (`--execute-only`,
      `--export-dynamic-symbol`, `--max-cache-size`, `--undefined-glob`, …):
      with one dash they are a short option with an attached value.
      (`mold: undefined-glob`)
- [ ] A single-dash word is tried as a long option before a short option
      takes the rest of it: `-entry=main` is `--entry=main`, not
      `-e ntry=main`; `-eh-frame-hdr`, `-end-group`, `-filter`,
      `-fix-cortex-a53-843419` too (getopt_long_only; mold 890ec2da).
      (`a_single_dash_long_wins_over_a_short_with_an_attached_value`;
      `mold: entry`, `filter`, `auxiliary`)
- [ ] …except `-l`, which always takes its attached value: `-lfoo` is
      `--library=foo`. (`mold: library`)
- [x] Long names are case-sensitive and may be capitalized: `-Map`,
      `-Bstatic`, `-Bsymbolic`.

## Phase 2: values

- [x] `--name=value` and `--name value`.
- [ ] `-name=value` and `-name value` for single-dash long options (after
      phase 1).
- [x] Short options with attached or separate values: `-Ldir`/`-L dir`,
      `-ofile`/`-o file`, `-lname`, `-Tscript`, `-e sym`, `-u sym`,
      `-h soname`, `-R path`, `-m emul`/`-melf_x86_64`, `-O2`.
- [x] Non-UTF-8 values, borrowed rather than copied. (`BStr` words;
      `cursor_borrows_non_utf8_separate_and_attached_values`)
- [x] Values that only bind with `=`: `--thinlto-index-only[=file]` must not
      swallow an input file. (`require_equals`;
      `thinlto_index_only_takes_its_value_after_equals`)
- [x] Optional values: `--build-id` alone is `fast` (wild), `--build-id=sha1`.
      (`value_optional` + `default_missing` + `require_equals`; `mold: build-id`)
- [ ] Options taking several following words (wild's three-parameter
      handlers).
- [ ] `-l:libfoo.a` names a file exactly, not a library: the value's meaning,
      but the lexer must leave the `:` alone. (`mold: library`)
- [ ] Numbers in C syntax: `0x` hex, leading-zero octal
      (`--image-base=0x400000`, `--section-start=.text=0x1000`,
      `-Ttext=0x1000`): a `FromArg`. (`mold: image-base`, `section-start`;
      wild's `-Ttext`/`-Tdata`/`-Tbss` round trips,
      `test_section_start_takes_precedence_over_ttext`)
- [ ] `--defsym=SYMBOL=EXPR`: the first `=` ends the name.
      (`mold: defsym`, `defsym2`, `defsym-error`, `defsym-overflow`)

## Phase 3: `-z` keywords

- [ ] `-z keyword` and `-zkeyword`: `-z now`, `-z relro`, `-z noexecstack`.
      (`mold: z-now`, `z-defs`, `z-origin`, `z-nodump`, `z-rodynamic`)
- [ ] `-z key=value`: `-z max-page-size=4096`, `-z stack-size=…`,
      `-z cet-report=error`. (`mold: z-max-page-size`, `z-stack-size`,
      `z-cet-report`)
- [ ] Keyword pairs as switches (`now`/`lazy`, `relro`/`norelro`,
      `sectionheader`/`nosectionheader`). (`mold: z-sectionheader`,
      `z-separate-code`)
- [ ] Unknown keywords warn rather than fail, in both linkers. A second
      vocabulary: `-z` is a sub-parser over its value.

## Phase 4: order is meaning

- [ ] Inputs and position-dependent state interleave: `--as-needed`,
      `--whole-archive`, `-Bstatic`/`-Bdynamic`, `--push-state`/`--pop-state`,
      `--start-group`/`--end-group`, `-(`/`-)`, `--start-lib`/`--end-lib`.
      Each input takes the state where it appears (wild's `modifier_stack`).
      A derived struct of fields loses the order: this wants the combinator
      layer, occurrences folded in order into a list of events.
      (`mold: as-needed`, `as-needed-dso`, `whole-archive`, `push-pop-state`,
      `start-lib`, `static-archive`)
- [x] Inputs are any word that is not an option, anywhere on the line.
- [ ] A stray `--pop-state` or `--end-group` is an error at its position.

## Phase 5: response files

- [ ] `@file` expands to the file's words, recursively (mold caps nesting at
      10), with quotes and backslashes; `@` is not a flag character.
      (`mold: response-file`, `response-file2`, `response-file-quoting`;
      wild `test_parse_recursive_file_option`, `test_arguments_from_string`,
      `test_parse_overlapping_file_and_inline_options`)
- [ ] Words borrow from the mapped file where they need no unquoting (mold);
      `Argv` over `&[&BStr]` fits if the expansion owns the storage.

## Phase 6: unknown, ignored, compatibility

- [ ] A list of options accepted and ignored with a warning.
      (wild `test_ignored_flags`)
- [ ] Unknown options collected and reported together (wild) or fatal at once
      (mold). (`mold: no-object-file`)
- [x] `--x`/`--no-x` pairs. (`negate`; `mold: gc-sections`,
      `no-eh-frame-header`, `no-quick-exit`, `warn-once`)
- [ ] `-plugin`, `-plugin-opt=…`, `--lto-*`/`--thinlto-*` forwarded as text,
      some renamed (mold `read_lto_option`). (`mold: lto-*`)
- [ ] `-v` prints the version and keeps linking, `-V` adds the emulations,
      `--version` prints and exits; `--help` in GNU ld's layout.
      (`mold: version`, `help`)
- [ ] The flavour from `argv[0]` and a first pass over the arguments (wild
      decides ELF/Mach-O/Wasm before parsing). (wild `test_flavor`)
- [x] Environment variables: `MOLD_JOBS`, `WILD_*`. (`env`)

## Phase 7: the linkers

- [ ] mold's command line parsed by winnow-args behind a feature, its 573
      tests passing, and the parse benchmark against its cursor.
- [ ] The same for wild against its parser and integration tests.
