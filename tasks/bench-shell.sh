#!/usr/bin/env bash
# Cost of a builtin call in bash-compatible shells: each command run 100 000
# times in a function, best of three, in microseconds a call (the loop
# included: `:` is the loop alone). Then, with SCRIPTS=dir, user instructions
# for each script in it, averaged over copies at four paths (a path changes a
# shell's allocation pattern by a few percent).
#
#   tasks/bench-shell.sh bash=/usr/bin/bash clap=target-clap/brush winnow=target/release/brush
#   SCRIPTS=../brush/benchmarks/real-world tasks/bench-shell.sh ...
#   PIN="numactl -N 3 -m 3 taskset -c 96" tasks/bench-shell.sh ...
set -euo pipefail

[ $# -gt 0 ] || { sed -n '2,11p' "$0"; exit 2; }
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
