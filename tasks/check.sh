#!/usr/bin/env bash
# Build, lint, document and test every feature configuration, and report the
# warnings in this workspace. Warnings from sibling checkouts (a local
# `[patch]` in .cargo/config.toml) are counted apart: they are not ours to fix.
#
#   tasks/check.sh
set -uo pipefail
cd "$(dirname "$0")/.."

status=0
sibling='\.\./|/(usage|bpaf|clap|winnow)/'
run() {
  local out ours theirs failed=""
  out=$("$@" 2>&1)
  local code=$?
  # A diagnostic's location is its first `-->` line.
  ours=$(printf '%s\n' "$out" | grep -E '^ *--> ' | grep -vcE "$sibling")
  theirs=$(printf '%s\n' "$out" | grep -E '^ *--> ' | grep -cE "$sibling")
  if [ "$code" -ne 0 ] || [ "$ours" -ne 0 ]; then
    failed=" FAILED"
    status=1
  fi
  printf '%-88s ours %s, siblings %s%s\n' "$*" "$ours" "$theirs" "$failed"
  if [ -n "$failed" ]; then
    printf '%s\n' "$out" | grep -E '^(warning|error)|^ *--> |FAILED|panicked' | grep -vE "$sibling" | head -20
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
RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args -p winnow-args-derive --no-deps --all-features
# A link to an item behind a feature only breaks with the feature off.
RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args --no-deps --no-default-features
RUSTDOCFLAGS='-D warnings' run cargo doc -p winnow-args --no-deps --no-default-features --features help-text
run cargo test --workspace --all-features
exit "$status"
