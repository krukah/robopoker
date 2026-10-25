# spar

Benchmark client for **Slumbot** — the research bot that has been the field's
reference opponent since 2016. Open access, no key, no quota.

The harness (hand loop, statistics, hand-history rows, scoreboard) lives in
[`bout`](../bout); this crate is the two-endpoint HTTP surface, Slumbot's
action-string grammar, and the chip-scale translation.

```bash
export DB_URL="postgresql://postgres:password@localhost:5432/robopoker"
cargo run --release --bin slumbot -- \
  --regime pluribus --version v1 \
  --variants base*1,dirac*1,depth+world*4 --hands 4096
```

Slumbot deals 200 BB off 50/100, hence the `SLUMBOT_STACK = 400` mapping. It
reports no AIVAT, so runs here are scored on raw chips only and need roughly
ten times the hands to reach the same significance.

Part of the [robopoker](https://github.com/krukah/robopoker) workspace — see the
top-level README for the full architecture and crate map.
