# winnow-args, usage, bpaf and clap

What each derive (or, for bpaf, its combinators too) offers, read from the
projects' documentation: usage 6.11, bpaf 0.9, clap 4. `✓` built in · `~`
possible with some work, or in a companion crate · `–` not offered. How fast
each is: [`benchmarks/`](../benchmarks/README.md). The rename from clap's and
usage's derives is in the crate docs, under "From clap and usage".

## How the parser is made

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| Derive | ✓ | ✓ | ✓ | ✓ |
| Parser generated at compile time | ✓ | ✓ | – | – |
| Combinators, without a macro | ✓ | – | ✓ | ~ builder |
| A CLI built at runtime | – | ✓ `usage-lib` | ✓ | ✓ builder |
| Words borrowed, not copied | ✓ | ✓ | – | – |
| Non-UTF-8 values | ✓ Unix | ✓ | ✓ | ✓ |

## Flags and values

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| Short bundles, aliases, counts | ✓ | ✓ | ✓ | ✓ |
| Repeated and delimited values | ✓ | ✓ | ~ no delimiter | ✓ |
| Value enums, choices | ✓ | ✓ | ~ `FromStr` | ✓ |
| `env`, defaults | ✓ | ✓ | ✓ | ✓ |
| Optional value (`--color[=WHEN]`) | ✓ | ✓ | ~ | ✓ |
| `require_equals`, hyphen values, negative numbers | ✓ | ✓ | ~ | ✓ |
| `--no-x` negation | ✓ | ✓ | ~ | ~ |
| A flag taking several words, a fixed number or a range | ✓ | ✓ | ~ | ✓ |
| Custom value parser in the derive | ~ `FromArg` type | ~ `FromStr` type | ✓ | ✓ |
| Defaults computed, or depending on another flag | ✓ | ✓ | ~ | ✓ |

## Shell and linker conventions

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| `+x` options (shell builtins) | ✓ | – | ~ | – |
| Long options with one dash (GNU `ld`) | ✓ | – | ~ | – |
| Keyword vocabularies (`-z now`) | ✓ | – | ~ | – |
| Flags kept in command-line order | ✓ | ~ ordered group | ~ | ~ indices |
| Unknown flags kept as words, or collected | ✓ | ✓ | ~ | ~ |
| Sigil arguments, clauses | – | ✓ | ~ | – |
| Response files (`@file`) | ✓ | ✓ | – | ~ |

Unknown flags are errors here unless `unknown_flags = "value"`; usage keeps
the word. That default is a decision in [`CHECKLIST.md`](./CHECKLIST.md).
`OsString` and `PathBuf` keep non-UTF-8 bytes on Unix only.

## Positionals and subcommands

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| Positionals, `--`, a trailing list | ✓ | ✓ | ✓ | ✓ |
| Words after `--` only, or `--` kept | ✓ | ✓ | ~ | ✓ |
| Subcommands, nested, with aliases | ✓ | ✓ | ✓ | ✓ |
| Global flags | ✓ | ✓ | – (0.10: ✓) | ✓ |
| Shared flags (`flatten`) | ✓ | ✓ | ✓ | ✓ |
| Default subcommand | ✓ | ✓ | ~ | ~ |
| External subcommands, multicall | – | ✓ | ~ | ✓ |
| Subcommands discovered at runtime | – | ✓ | ~ | ~ |

## Rules

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| Conflicts, requirements, groups | ✓ | ✓ | ~ by composition | ✓ |
| Conditional requirements (`required_if`) | – | ✓ | ~ | ✓ |
| Flag constraints checked at compile time | ✓ | – | ~ types | – |
| Cross-field validation hook | – | ✓ | ✓ `guard` | ~ |

## Help, errors, completion

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| Help from doc comments, wrapped, coloured | ✓ | ✓ | ✓ | ✓ |
| Help prose left out of the binary by a feature | ✓ | – | – | – |
| "Did you mean" suggestions | – | – | ✓ | ✓ |
| Completion scripts | ✓ | ✓ | ✓ | ~ `clap_complete` |
| Completion of values by hint or function | – | ✓ | ✓ | ~ |
| Man pages, markdown | ~ `winnow-args-man`, `-markdown` | ✓ | ✓ | ~ `clap_mangen` |
| Deprecated flags with warnings | – | ✓ | – | – |

## Around the parser

| | winnow-args | usage | bpaf | clap |
|---|:-:|:-:|:-:|:-:|
| A portable spec of the CLI | – | ✓ | – | – |
| Config-file settings | – | ✓ | – | – |
| Generated dispatch | – | ✓ | – | – |
| Update a parsed value from another line | – | ✓ | – | ✓ |

[`CHECKLIST.md`](./CHECKLIST.md) lists what winnow-args has and lacks in
detail.
