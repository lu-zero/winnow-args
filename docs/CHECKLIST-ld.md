# Checklist: mold and wild (the `ld` command line)

What a linker needs to parse GNU `ld`'s command line with winnow-args, read
from [mold](../../mold) (the Rust port, `src/cmdline.rs`, branch
`fix/cmdline-gnu-dash-rules`) and [wild](../../wild) (`libwild/src/args.rs`,
`libwild/src/args/elf.rs`). `[x]` means winnow-args covers it today, `[ ]`
that it does not yet.

A linker parses once per link, but its command line can be thousands of words
long (response files listing every object), so per-word throughput and zero
copies matter; the grammar matters more.

## 1. Dashes

- [ ] Long options take one dash or two: `-shared` = `--shared`,
      `-soname x` = `--soname x`. (mold `match_option`, wild `strip_option`)
- [ ] Except names starting with `o`, which need two dashes: `-omagic` is
      `-o magic`. (mold)
- [ ] And a list of names that need two dashes (`--execute-only`,
      `--export-dynamic-symbol`, `--max-cache-size`, …): with one dash they are
      a short option with an attached value, `-execute-only` = `-e xecute-only`.
- [ ] A single-dash word is tried as a long option before any short option may
      take the rest of it: `-entry=main` is `--entry=main`, not `-e ntry=main`;
      `-eh-frame-hdr`, `-end-group`, `-filter`, `-fix-cortex-a53-843419` too
      (getopt_long_only; mold 890ec2da).
- [ ] …except `-l`, which always takes its attached value: `-lfoo` is
      `--library=foo` even where `lfoo` could spell a long option.
- [x] Long names are case-sensitive and may be capitalized: `-Map`, `-Bstatic`,
      `-Bdynamic`, `-Bsymbolic`.

## 2. Values

- [x] `--name=value` and `--name value`. (flags taking a value)
- [ ] `-name=value` and `-name value` for single-dash long options.
- [ ] Options whose value only binds with `=`, never the next word:
      `--thinlto-index-only[=file]` must not swallow an input file.
      (`require_equals`, once single-dash longs exist)
- [x] Optional values: `--build-id` alone means `fast` (wild), `--build-id=sha1`.
      (`value_optional` + `default_missing`, with `require_equals`)
- [x] Short options with attached or separate values: `-Ldir`/`-L dir`,
      `-ofile`/`-o file`, `-lname`/`-l name`, `-Tscript`, `-e sym`, `-u sym`,
      `-h soname`, `-R path`, `-m emul`/`-melf_x86_64`, `-O2`.
- [x] Values that start with a dash where the option demands one
      (`--defsym=-x=1`, `-o -`). (`allow_hyphen_values`)
- [ ] Options taking several following words: wild's three-parameter options.
- [ ] `-l:libfoo.a`: the colon form names a file exactly, not a library.
      (the value's meaning, but the lexer must leave `:` alone)
- [ ] Numbers in C syntax: `0x` hex, leading-zero octal
      (`--image-base=0x400000`, `-z max-page-size=0x1000`). (a `FromArg`)
- [ ] `--defsym=SYMBOL=EXPR`: the first `=` ends the name, the rest is a value
      that contains `=`.

## 3. `-z` keywords

- [ ] `-z keyword` and `-zkeyword`: `-z now`, `-z relro`/`-z norelro`,
      `-z noexecstack`.
- [ ] `-z key=value`: `-z max-page-size=4096`, `-z separate-code`.
- [ ] Keyword pairs as switches (`now`/`lazy`, `relro`/`norelro`,
      `text`/`notext`); unknown keywords warn rather than fail (both linkers).
      A second, nested vocabulary: `-z` is a sub-parser over its value.

## 4. Order is meaning

- [ ] Inputs and position-dependent state interleave: `--as-needed`,
      `--no-as-needed`, `--whole-archive`, `--no-whole-archive`, `-Bstatic`,
      `-Bdynamic`, `--push-state`/`--pop-state`, `--start-group`/`--end-group`,
      `-(`/`-)`. Each input takes the state in force where it appears (wild's
      `modifier_stack`). A derived struct of fields loses that; this wants the
      combinator layer: a sequence of occurrences folded in order.
- [x] Inputs are any word that is not an option, anywhere on the line.
- [ ] `--push-state` without `--pop-state`, or a stray `--end-group`, is an error
      with the position.

## 5. Response files and argv

- [ ] `@file` expands to the file's words, recursively (mold caps nesting at
      10), with quotes and backslashes; `@` is not a flag character. Words can
      borrow from the mapped file (mold) — `Argv` over `&[&BStr]` fits if the
      expansion owns the storage.
- [x] Non-UTF-8 paths. (`BStr` words)
- [ ] The flavour comes from `argv[0]` and a first pass over the arguments
      (wild decides the platform, ELF/Mach-O/Wasm, before parsing).

## 6. Unknown, ignored and compatibility options

- [ ] A list of options accepted and ignored with a warning (both linkers).
- [ ] Unknown options collected and reported together at the end (wild) or
      fatal at once (mold).
- [x] `--no-` negations: `--gc-sections`/`--no-gc-sections`,
      `--eh-frame-hdr`/`--no-eh-frame-hdr`. (`negate`)
- [ ] `-plugin`, `-plugin-opt=…` and `--lto-*`/`--thinlto-*` families forwarded
      as text, some with renamed prefixes (mold `read_lto_option`).
- [ ] `-v` prints the version and keeps linking, `-V` adds the emulations,
      `--version` prints and exits.
- [x] Environment: `MOLD_JOBS`, `WILD_*`. (`env`)

## 7. Scale and measurement

- [ ] Several hundred options (wild declares 158 for ELF, mold more): the
      option `match` at that size, compared with mold's and wild's own lookup.
- [ ] A benchmark: parse a real link line (a Chromium- or rustc-sized response
      file) in each linker's parser and in winnow-args.
- [ ] GNU `ld --help` layout, if either linker wants generated help.
