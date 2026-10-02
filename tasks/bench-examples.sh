#!/usr/bin/env bash
# Warm instructions per parse for the two examples (docs/PERF.md step 48):
# each line is parsed PARSE_N times and nothing printed; the count is
# (N=200000 minus N=0) / 200000. Needs `perf`.
#
#   tasks/bench-examples.sh
set -euo pipefail
cd "$(dirname "$0")/.."

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
