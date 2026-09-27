#!/usr/bin/env bash
# Cost of `example -v/--verbose -p/--path=PATH` in each framework, three ways:
#
# - instructions for one cold parse: the same binary run with PARSE_N=0 and
#   PARSE_N=1, differenced, as ../usage/tasks/perf-shadow.sh does;
# - cold wall time: the first parse in a fresh process, median over $RUNS processes;
# - warm wall time: min and median per parse in a hot loop (time-sweep).
#
#   tasks/perf.sh                      # every line of bench/argv.txt, one table each
#   tasks/perf.sh -vp/tmp/x            # any argv; every binary gets the same one
#   SUITE=mise tasks/perf.sh           # mise's full CLI: bench/mise-argv.txt
set -euo pipefail
cd "$(dirname "$0")/.."

SUITE=${SUITE:-example}
if [ "$SUITE" = mise ]; then
  FRAMEWORKS=(usage wa bpaf clap)
  PREFIX=parse-n-mise-
  SWEEP=time-sweep-mise
  LINES=bench/mise-argv.txt
else
  FRAMEWORKS=(usage wa wa-disp wa-comb bpaf clap)
  PREFIX=parse-n-
  SWEEP=time-sweep
  LINES=bench/argv.txt
fi
export SUITE

if [ $# -eq 0 ]; then
  while IFS= read -r line; do
    # shellcheck disable=SC2086 # a line is several words on purpose
    [ -n "$line" ] && "$0" $line && echo
  done <"$LINES"
  exit
fi
ARGV=("$@")
RUNS=${RUNS:-31}

cargo build --release -q -p bench 2>/dev/null || cargo build --release -p bench

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
      --cachegrind-out-file=/dev/null "./target/release/$1" "${ARGV[@]}" 2>&1 |
      sed -n 's/.*I *refs: *//p' | tr -d ','
  }
elif perf stat -e instructions:u true >/dev/null 2>&1; then
  counter="perf instructions:u, median of $RUNS"
  instructions() {
    for _ in $(seq "$RUNS"); do
      PARSE_N="$2" perf stat -x, -e instructions:u "./target/release/$1" "${ARGV[@]}" 2>&1 >/dev/null |
        cut -d, -f1
    done | median
  }
else
  counter="none available"
  instructions() { echo 0; }
fi

cold_ns() {
  for _ in $(seq "$RUNS"); do
    PARSE_N=1 PARSE_TIME=1 "./target/release/$1" "${ARGV[@]}" | sed -n 2p
  done | median
}

stripped_size() {
  local copy
  copy=$(mktemp)
  cp "./target/release/$1" "$copy" && strip "$copy"
  wc -c <"$copy" | tr -d ' '
  rm -f "$copy"
}

ratio() {
  awk -v a="$1" -v b="$2" 'BEGIN { if (b == 0) print "-"; else printf (a / b < 10 ? "%.1fx" : "%dx"), a / b }'
}

declare -A warm_min warm_median
while read -r name min med; do
  warm_min[$name]=$min
  warm_median[$name]=$med
done < <("./target/release/$SWEEP" "${ARGV[@]}")

echo "argv: ${ARGV[*]}"
echo "instructions: $counter; cold ns: median of $RUNS processes; warm ns: min / median of 2000 rounds"
echo
printf '| %-9s | %8s | %7s | %8s | %7s | %8s | %11s | %9s |\n' \
  framework instr "×usage" "cold ns" "×usage" "warm ns" "warm median" "stripped"
printf '|%s|%s|%s|%s|%s|%s|%s|%s|\n' ----------- ---------: --------: ---------: --------: ---------: ------------: ----------:
base_instr=
base_cold=
for fw in "${FRAMEWORKS[@]}"; do
  bin="$PREFIX$fw"
  got=$(PARSE_N=1 "./target/release/$bin" "${ARGV[@]}")
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
