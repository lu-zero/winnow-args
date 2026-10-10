# Development tasks. `just` with no arguments lists them. Each recipe is a
# bash script; arguments are passed to it as `$@`, and settings such as
# `PROFILE` and `PIN` are read from the environment.

set positional-arguments

# `just` alone lists the recipes. Private, so the list does not include it.
[private]
default:
    @just --list

# Build, lint, document and test every feature configuration, warning-free.
[doc("Every feature set and profile: build, clippy, docs, tests")]
check:
    #!/usr/bin/env bash
    set -uo pipefail

    status=0
    run() {
      local out warnings failed=""
      out=$("$@" 2>&1)
      local code=$?
      # A diagnostic's location is its first `-->` line.
      warnings=$(printf '%s\n' "$out" | grep -cE '^ *--> ')
      if [ "$code" -ne 0 ] || [ "$warnings" -ne 0 ]; then
        failed=" FAILED"
        status=1
      fi
      printf '%-98s %s diagnostics%s\n' "$*" "$warnings" "$failed"
      if [ -n "$failed" ]; then
        printf '%s\n' "$out" | grep -E '^(warning|error)|^ *--> |FAILED|panicked' | head -20
      fi
    }

    run cargo fmt --all -- --check
    for profile in dev release release-lto; do
      run cargo build --workspace --all-targets --profile "$profile"
    done
    for features in "--no-default-features" "--no-default-features --features derive" \
      "--no-default-features --features help-text" "--all-features"; do
      # shellcheck disable=SC2086 # several flags on purpose
      run cargo clippy -p winnow-args --all-targets $features -- -D warnings
      # shellcheck disable=SC2086
      run cargo test -p winnow-args $features
    done
    run cargo clippy --workspace --all-targets --all-features -- -D warnings
    run cargo clippy -p bench --all-targets --no-default-features -- -D warnings
    RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args -p winnow-args-derive -p winnow-args-spec -p winnow-args-markdown -p winnow-args-man --no-deps --all-features
    # A link to an item behind a feature only breaks with the feature off.
    RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args --no-deps --no-default-features
    RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args --no-deps --no-default-features --features help-text
    run cargo test --workspace --all-features
    # The fragments the derive writes, read back. Only this test: the others
    # reuse type names, and a fragment is filed under its type's name.
    WINNOW_ARGS_SPEC="$PWD/target/spec-test" run cargo test -p winnow-args --all-features --test spec_fragments
    exit "$status"

# Regenerate a generated source:
#   just gen mold MOLD_DIR
#   just gen examples BRUSH_DIR MOLD_DIR
#   just gen mise-shadow
[doc("Regenerate mold, the examples, or the mise shadow")]
gen *args:
    cargo run -q -p xtask -- gen "$@"

# Cost of `example -v/--verbose -p/--path=PATH` in each framework, three ways:
# instructions for one cold parse (PARSE_N=1 minus PARSE_N=0, as
# ../usage/tasks/perf-shadow.sh does), cold wall time (the first parse in a
# fresh process, median over $RUNS) and warm wall time (min and median per
# parse in a hot loop). With no argv, every line of benchmarks/argv.txt.
#
#   just perf -vp/tmp/x                # any argv; every binary gets the same one
#   SUITE=mise just perf               # mise's full CLI: benchmarks/mise-argv.txt
#   PROFILE=release-lto just perf      # one codegen unit and fat LTO: stable sizes
#   BPAF010=1 just perf                # with the unreleased bpaf 0.10, from git
[doc("Cold and warm parse cost, in each framework, of an argv or every bench line")]
perf *argv:
    #!/usr/bin/env bash
    set -euo pipefail

    SUITE=${SUITE:-example}
    if [ "$SUITE" = mise ]; then
      FRAMEWORKS=(usage wa bpaf clap)
      PREFIX=parse-n-mise-
      SWEEP=time-sweep-mise
      LINES=benchmarks/mise-argv.txt
    else
      FRAMEWORKS=(usage wa wa-disp wa-comb bpaf clap)
      PREFIX=parse-n-
      SWEEP=time-sweep
      LINES=benchmarks/argv.txt
    fi
    export SUITE
    PROFILE=${PROFILE:-release}
    export PROFILE
    BIN=./target/$PROFILE

    RUNS=${RUNS:-31}

    cargo build --profile "$PROFILE" -q -p bench 2>/dev/null || cargo build --profile "$PROFILE" -p bench
    sweeps=("$BIN/$SWEEP")
    if [ -n "${BPAF010:-}" ]; then
      cargo build --profile "$PROFILE" -q --manifest-path benchmarks/bpaf010/Cargo.toml --target-dir target
      FRAMEWORKS+=(bpaf010)
      sweeps+=("$BIN/$SWEEP-bpaf010")
    fi

    median() {
      sort -n | awk '{ a[NR] = $1 } END { print a[int((NR + 1) / 2)] }'
    }

    # cachegrind where it works; it does not on aarch64 hosts whose glibc uses
    # instructions valgrind cannot decode. The fallback hardware counter wobbles by
    # a few hundred per run, so it takes the median.
    if bash -c 'valgrind --tool=none true; exit $?' >/dev/null 2>&1; then
      counter=cachegrind
      instructions() {
        PARSE_N="$2" valgrind --tool=cachegrind --cache-sim=no --branch-sim=no \
          --cachegrind-out-file=/dev/null "$BIN/$1" "${ARGV[@]}" 2>&1 |
          sed -n 's/.*I *refs: *//p' | tr -d ','
      }
    elif perf stat -e instructions:u true >/dev/null 2>&1; then
      counter="perf instructions:u, median of $RUNS"
      instructions() {
        for _ in $(seq "$RUNS"); do
          PARSE_N="$2" perf stat -x, -e instructions:u "$BIN/$1" "${ARGV[@]}" 2>&1 >/dev/null |
            cut -d, -f1
        done | median
      }
    else
      counter="none available"
      instructions() { echo 0; }
    fi

    cold_ns() {
      for _ in $(seq "$RUNS"); do
        PARSE_N=1 PARSE_TIME=1 "$BIN/$1" "${ARGV[@]}" | sed -n 2p
      done | median
    }

    stripped_size() {
      local copy
      copy=$(mktemp)
      cp "$BIN/$1" "$copy" && strip "$copy"
      wc -c <"$copy" | tr -d ' '
      rm -f "$copy"
    }

    ratio() {
      awk -v a="$1" -v b="$2" 'BEGIN { if (b == 0) print "-"; else printf (a / b < 10 ? "%.1fx" : "%dx"), a / b }'
    }

    table() {
      ARGV=("$@")
      local name min med fw bin got instr cold base_instr= base_cold=
      declare -A warm_min warm_median
      while read -r name min med; do
        warm_min[$name]=$min
        warm_median[$name]=$med
      done < <(for sweep in "${sweeps[@]}"; do "$sweep" "${ARGV[@]}"; done)

      echo "argv: ${ARGV[*]}"
      echo "profile: $PROFILE; instructions: $counter; cold ns: median of $RUNS processes; warm ns: min / median of 2000 rounds"
      echo
      printf '| %-9s | %8s | %7s | %8s | %7s | %8s | %11s | %9s |\n' \
        framework instr "×usage" "cold ns" "×usage" "warm ns" "warm median" "stripped"
      printf '|%s|%s|%s|%s|%s|%s|%s|%s|\n' ----------- ---------: --------: ---------: --------: ---------: ------------: ----------:
      for fw in "${FRAMEWORKS[@]}"; do
        bin="$PREFIX$fw"
        got=$(PARSE_N=1 "$BIN/$bin" "${ARGV[@]}")
        [ "$got" = 1 ] || echo "warning: $bin did not accept the argv (printed $got)" >&2
        instr=$(($(instructions "$bin" 1) - $(instructions "$bin" 0)))
        cold=$(cold_ns "$bin")
        base_instr=${base_instr:-$instr}
        base_cold=${base_cold:-$cold}
        printf '| %-9s | %8s | %7s | %8s | %7s | %8s | %11s | %9s |\n' "$fw" \
          "$instr" "$(ratio "$instr" "$base_instr")" \
          "$cold" "$(ratio "$cold" "$base_cold")" \
          "${warm_min[$fw]}" "${warm_median[$fw]}" "$(stripped_size "$bin")"
      done
    }

    if [ $# -gt 0 ]; then
      table "$@"
    else
      while IFS= read -r line; do
        # shellcheck disable=SC2086 # a line is several words on purpose
        [ -n "$line" ] && table $line && echo
      done <"$LINES"
    fi

# Warm instructions per parse for the two examples (docs/PERF.md step 48):
# each line is parsed PARSE_N times and nothing printed; the count is
# (N=200000 minus N=0) / 200000. Needs `perf`.
[doc("Warm instructions per parse for the two examples")]
bench-examples:
    #!/usr/bin/env bash
    set -euo pipefail

    cargo build --release -q --examples
    BIN=./target/release/examples

    measure() {
      local n a b
      n=200000
      a=$(PARSE_N=$n perf stat -x, -e instructions:u "$@" 2>&1 >/dev/null | cut -d, -f1)
      b=$(PARSE_N=0 perf stat -x, -e instructions:u "$@" 2>&1 >/dev/null | cut -d, -f1)
      echo $(((a - b) / n))
    }

    while IFS= read -r line; do
      # shellcheck disable=SC2086 # a line is several words on purpose
      printf '%-34s %s\n' "$line" "$(measure "$BIN/brush_builtins" $line)"
    done <<'LINES'
    pwd -P
    cd -P /tmp
    unset -fv x
    declare -i +x n=1
    test -n x
    set -eu +x -o pipefail
    kill -s TERM 1234
    echo -n a b c
    printf -v x %s a
    read -rp prompt -t 2.5 a b
    LINES

    for line in \
      "-shared -o out.so a.o" \
      "-shared -o out --as-needed -lc -lm a.o b.o --whole-archive c.a -s -z now -zrelro --build-id=sha1 -L/usr/lib e.o -lpthread"; do
      # shellcheck disable=SC2086
      total=$(measure "$BIN/ld" $line)
      words=$(wc -w <<<"$line")
      printf 'ld, %2d words: %s (%s a word)\n' "$words" "$total" "$((total / words))"
    done

# Cost of a builtin call in bash-compatible shells: each command run 100 000
# times in a function, best of three, in microseconds a call (the loop
# included: `:` is the loop alone). Then, with SCRIPTS=dir, user instructions
# for each script in it, averaged over copies at four paths (a path changes a
# shell's allocation pattern by a few percent). Paths are the caller's.
#
#   just bench-shell bash=/usr/bin/bash clap=target-clap/brush winnow=target/release/brush
#   SCRIPTS=../brush/benchmarks/real-world just bench-shell ...
#   PIN="numactl -N 3 -m 3 taskset -c 96" just bench-shell ...
[doc("Cost of a builtin call, and of scripts, in bash-compatible shells: NAME=SHELL…")]
[no-cd]
bench-shell +shells:
    #!/usr/bin/env bash
    set -euo pipefail

    PIN=${PIN:-}
    names=() paths=()
    for shell in "$@"; do
      names+=("${shell%%=*}")
      paths+=("$(realpath "${shell#*=}")")
    done
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    printf '%-22s' command
    printf '%9s' "${names[@]}"
    echo
    while IFS= read -r cmd; do
      printf '%-22s' "$cmd"
      for path in "${paths[@]}"; do
        best=
        for _ in 1 2 3; do
          # shellcheck disable=SC2086 # PIN is several words on purpose
          $PIN /usr/bin/time -o "$tmp/t" -f %e "$path" --norc --noprofile -c \
            "f(){ for ((i=0;i<100000;i++)); do $cmd; OPTIND=1; done; }; f" >/dev/null 2>&1 || true
          t=$(tail -1 "$tmp/t")
          best=$(awk -v a="$t" -v b="${best:-999}" 'BEGIN { print (a < b) ? a : b }')
        done
        printf '%9.1f' "$(awk -v s="$best" 'BEGIN { print s * 10 }')"
      done
      echo
    done <<'COMMANDS'
    :
    set -f +f
    declare -i n=1
    local
    export E=1
    [ a = a ]
    test -n x
    echo -n
    printf %s x
    read -r v <<<x
    shift 0
    unset -v x
    type -t ls
    command true
    cd .
    shopt -q extglob
    ulimit -n
    kill -0 $$
    trap -p
    getopts ab o -a
    compgen -W "a b" a
    COMMANDS

    [ -n "${SCRIPTS:-}" ] || exit 0
    for copy in a bb ccc dddd; do
      mkdir -p "$tmp/$copy"
      cp "$SCRIPTS"/*.sh "$tmp/$copy/"
    done
    echo
    printf '%-22s' "script (M instr)"
    printf '%9s' "${names[@]}"
    echo
    for script in "$SCRIPTS"/*.sh; do
      script=$(basename "$script")
      # The scripts that take an argument, as brush's benchmarks run them.
      case $script in
        config-lint.sh) arg=500 ;;
        deploy-sim.sh) arg=staging ;;
        *) arg= ;;
      esac
      printf '%-22s' "$script"
      for path in "${paths[@]}"; do
        total=0
        for copy in a bb ccc dddd; do
          # shellcheck disable=SC2086
          v=$(cd "$tmp/$copy" && $PIN perf stat -x, -e instructions:u -r 3 \
            "$path" --norc --noprofile "./$script" $arg 2>&1 >/dev/null | tail -1 | cut -d, -f1)
          total=$((total + v))
        done
        printf '%9d' $((total / 4000000))
      done
      echo
    done

# Cost of parsing a link line in linkers that take GNU ld's options: user
# instructions a word over a bare `--version`, and the whole run in
# milliseconds, for three generated lines ending in `--version` (so nothing
# is linked): input files only, options only, and a mix shaped like a real
# link. Needs `perf`. Paths are the caller's.
#
#   just bench-ld builtin=../mold/target/release/mold winnow=target/release/mold
#   PIN="numactl -N 3 -m 3 taskset -c 96" just bench-ld ...
[doc("Cost of parsing a link line, per word: NAME=LINKER…")]
[no-cd]
bench-ld +linkers:
    #!/usr/bin/env bash
    set -euo pipefail

    PIN=${PIN:-}

    inputs=() options=() mixed=(-o out --hash-style=gnu --build-id --eh-frame-hdr -m aarch64linux
      -pie -z now -z relro --as-needed -dynamic-linker /lib/ld-linux-aarch64.so.1)
    for ((i = 0; i < 2500; i++)); do inputs+=("obj/file$i.o"); done
    for ((i = 0; i < 250; i++)); do
      options+=(--gc-sections -z now --as-needed -L/usr/lib --hash-style=gnu -pie
        --no-undefined -soname libx.so -lm)
    done
    for ((i = 0; i < 2000; i++)); do
      ((i % 10)) || mixed+=("-L/usr/lib/dir$i")
      mixed+=("obj/file$i.o")
      ((i % 50)) || mixed+=(--push-state --whole-archive "lib/libx$i.a" --pop-state -lm)
      ((i % 100)) || mixed+=(-z noexecstack --gc-sections -plugin-opt=O2)
    done

    count() { # event, repeats, command...
      local event=$1 repeats=$2
      shift 2
      # shellcheck disable=SC2086 # PIN is several words on purpose
      $PIN perf stat -x, -e "$event" -r "$repeats" "$@" 2>&1 >/dev/null | tail -1 | cut -d, -f1
    }

    for linker in "$@"; do
      name=${linker%%=*} path=${linker#*=}
      base=$(count instructions:u 10 "$path" --version)
      for line in inputs options mixed; do
        declare -n words=$line
        total=$(count instructions:u 10 "$path" "${words[@]}" --version)
        ms=$(count task-clock 20 "$path" "${words[@]}" --version)
        printf '%-10s %-8s %5d words: %6d instr/word, %s ms\n' \
          "$name" "$line" "$((${#words[@]} + 1))" "$(((total - base) / (${#words[@]} + 1)))" "$ms"
      done
    done
