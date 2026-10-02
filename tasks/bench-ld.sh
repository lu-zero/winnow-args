#!/usr/bin/env bash
# Cost of parsing a link line in linkers that take GNU ld's options: user
# instructions a word over a bare `--version`, and the whole run in
# milliseconds, for three generated lines ending in `--version` (so nothing
# is linked): input files only, options only, and a mix shaped like a real
# link. Needs `perf`.
#
#   tasks/bench-ld.sh builtin=../mold/target/release/mold winnow=target/release/mold
#   PIN="numactl -N 3 -m 3 taskset -c 96" tasks/bench-ld.sh ...
set -euo pipefail

[ $# -gt 0 ] || { sed -n '2,10p' "$0"; exit 2; }
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
