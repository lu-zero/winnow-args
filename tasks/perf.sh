#!/usr/bin/env bash
# Cold-parse cost of `example -v/--verbose -p/--path=PATH` in each framework,
# measured as ../usage/tasks/perf-shadow.sh measures its mise shadows: run the
# same binary with PARSE_N=0 and PARSE_N=1 under cachegrind and difference the
# instruction counts, which leaves one parse in a fresh process and nothing else.
#
#   tasks/perf.sh                      # default argv
#   tasks/perf.sh -vp/tmp/x            # any argv; every binary gets the same one
set -euo pipefail
cd "$(dirname "$0")/.."

ARGV=("$@")
[ ${#ARGV[@]} -eq 0 ] && ARGV=(-v --path /tmp/x)

cargo build --release -q -p bench 2>/dev/null || cargo build --release -p bench

# cachegrind where it works; it does not on aarch64 hosts whose glibc uses
# instructions valgrind cannot decode. The fallback is the `instructions:u`
# hardware counter, which wobbles by a few hundred per run, so it takes the
# median of $RUNS runs.
RUNS=${RUNS:-31}
if (valgrind --tool=none true) >/dev/null 2>&1; then
  counter=cachegrind
  instructions() {
    PARSE_N="$2" valgrind --tool=cachegrind --cache-sim=no --branch-sim=no \
      --cachegrind-out-file=/dev/null "./target/release/$1" "${ARGV[@]}" 2>&1 |
      sed -n 's/.*I *refs: *//p' | tr -d ','
  }
else
  counter="perf instructions:u, median of $RUNS"
  instructions() {
    for _ in $(seq "$RUNS"); do
      PARSE_N="$2" perf stat -x, -e instructions:u "./target/release/$1" "${ARGV[@]}" 2>&1 >/dev/null |
        cut -d, -f1
    done | sort -n | awk '{ a[NR] = $1 } END { print a[int((NR + 1) / 2)] }'
  }
fi

size_of() {
  local copy
  copy=$(mktemp)
  cp "./target/release/$1" "$copy" && strip "$copy"
  wc -c <"$copy" | tr -d ' '
  rm -f "$copy"
}

echo "argv: ${ARGV[*]}   (counter: $counter)"
echo
printf '| %-20s | %14s | %8s | %16s |\n' framework "instr, cold" "vs usage" "stripped bytes"
printf '|%s|%s|%s|%s|\n' "----------------------" "---------------:" "---------:" "-----------------:"
base=
for bin in usage wa wa-comb bpaf clap; do
  got=$(PARSE_N=1 "./target/release/parse-n-$bin" "${ARGV[@]}")
  [ "$got" = 1 ] || echo "warning: parse-n-$bin did not accept the argv (printed $got)" >&2
  cold=$(($(instructions "parse-n-$bin" 1) - $(instructions "parse-n-$bin" 0)))
  base=${base:-$cold}
  ratio=$(awk -v a="$cold" -v b="$base" 'BEGIN { printf (a / b < 10 ? "%.1fx" : "%dx"), a / b }')
  printf '| %-20s | %14s | %8s | %16s |\n' "$bin" "$(printf "%'d" "$cold")" "$ratio" "$(printf "%'d" "$(size_of "parse-n-$bin")")"
done
echo
./target/release/time-sweep "${ARGV[@]}"
