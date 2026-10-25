Poker Hand History (PHH) parsing, chip-exact replay, and the 6-max → 2-max
reduction used to evaluate a heads-up blueprint against multiway logs.

Written for the PHH dataset's `data/pluribus` — the 10,000 hands published
with Brown & Sandholm (2019) — but nothing here is Pluribus-specific beyond
the choice of defaults.

```text
  .phh files ──parse──▶ Record ──replay──▶ Outcome { finishing, nodes }
                           │                            │
                           └──project──▶ Shape          └─▶ calibration,
                               (how this hand relates       policy agreement
                                to our heads-up tree)
```

The three consumers, in the order they became useful:

- **conformance** — does [`Replay::replay`] reproduce every logged `finishing_stacks`?
  An independent reference implementation of multiway settlement.
- **calibration** — do the observed bet sizes land on our `Size` grid?
  Reads [`Node`] and needs no blueprint.
- **agreement** ([`Agreement`]) — does our blueprint choose what Pluribus
  chose, on the subset of hands that are heads-up in our tree's sense?
  Reads [`Shape`]. One [`Oracle`] query per logged decision.
- **value** ([`Ledger`]) — what was Pluribus's decision *worth* against our
  blueprint, next to what ours was worth? The log has no continuation behind
  an action nobody took, so this one generates it: play the hand out from
  [`Spot::game`] behind both actions and difference the results.

Agreement is similarity; value is the closest thing to strength this corpus
can support.
