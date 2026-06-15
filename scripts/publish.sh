#!/usr/bin/env bash
#
# Publish all publishable workspace crates to crates.io in dependency order.
#
# The order is derived dynamically from `cargo metadata` (deps before
# dependents), so it stays correct as the graph changes. Crates marked
# `publish = false` (the bin/* runners) are skipped automatically.
#
# Usage:
#   scripts/publish.sh            # print the topological order and exit
#   scripts/publish.sh --execute  # actually `cargo publish` each crate in order
#
# Requires: jq, tsort (coreutils/util on macOS & Linux).
#
# Note: crates.io needs a few seconds to index each new version before a
# dependent can be published; --execute waits between crates.

set -euo pipefail
cd "$(dirname "$0")/.."

command -v jq >/dev/null   || { echo "error: jq not found"   >&2; exit 1; }
command -v tsort >/dev/null || { echo "error: tsort not found" >&2; exit 1; }

order() {
  local meta; meta=$(cargo metadata --format-version 1 --no-deps)
  # dependency edges among publishable crates -> topological order (deps first)
  local edges; edges=$(echo "$meta" | jq -r '
        [.packages[] | select(.publish == null) | .name] as $pub
        | .packages[] | select(.publish == null) as $p
        | ($p.dependencies[].name | select(. as $d | $pub | index($d))) as $dep
        | "\($dep) \($p.name)"
      ')
  # every publishable crate (so isolated ones with no edges aren't dropped by tsort)
  local all; all=$(echo "$meta" | jq -r '.packages[] | select(.publish == null) | .name')
  { echo "$edges" | tsort; echo "$all"; } | awk 'NF && !seen[$0]++'
}

CRATES=$(order)

if [[ "${1:-}" != "--execute" ]]; then
  echo "Publish order (deps first); pass --execute to publish:"
  echo "$CRATES" | nl -ba
  exit 0
fi

echo "Publishing $(echo "$CRATES" | wc -l | tr -d ' ') crates to crates.io..."
for crate in $CRATES; do
  echo "==> cargo publish -p $crate"
  cargo publish -p "$crate"
  echo "    waiting for crates.io to index $crate..."
  sleep 20
done
echo "Done."
