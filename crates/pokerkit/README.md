# pokerkit

Core type aliases, traits, and constants for robopoker.

`pokerkit` is the foundational crate of the robopoker workspace: the shared
vocabulary of domain types, the `(Version × Regime)` training configuration,
the `hyperparams!` singleton macro, and the generic action-translation engine
that maps arbitrary bet sizes onto a finite abstract action grid.

## Architecture

```mermaid
flowchart LR
  subgraph Aliases["Type aliases"]
    A1["Chips"]
    A2["Utility"]
    A3["Probability"]
    A4["Energy / Entropy"]
    A5["ID&lt;T&gt;"]
  end
  subgraph Config["Runtime config (OnceLock)"]
    C1["Version - V0..V3"]
    C2["Regime - Pluribus / Slumbot"]
    C3["Translation - Snap / Harmonic / Phargmax"]
  end
  subgraph Tools["Utilities"]
    T1["hyperparams! macro"]
    T2["Arbitrary / Unique traits"]
    T3["Summary metrics"]
  end
```

The crate defines the numeric vocabulary used across the workspace as thin
`f32`/`i16` aliases so intent is legible at call sites: `Chips` (stacks and
bets in big blinds), `Utility` (payoffs and regrets), `Probability` (strategy
weights), and `Energy`/`Entropy` (distances and temperatures). Training runs
are keyed by two orthogonal axes — `Version` (clustering abstraction) and
`Regime` (bet-sizing grid) — each a process-global `OnceLock` set once at
startup, together naming the database tables a run reads and writes.

### Action translation

```mermaid
flowchart LR
  raw["Raw bet size"] --> scalar["Scalar&lt;A&gt; (axis-tagged f64)"]
  lattice["Lattice&lt;A, P&gt; (sorted anchors + payloads)"] --> bracket
  scalar --> bracket["bracket -> Bracket(lo, hi)"]
  bracket -->|clamped| clamp["extreme anchor"]
  bracket -->|inside| ph["pharmonic p = (B-x)(1+A) / (B-A)(1+x)"]
  ph -->|Harmonic - sample| anchor["Anchor"]
  ph -->|Phargmax - p &gt;= 0.5| anchor
  scalar -->|Snap - L1 nearest| anchor
  clamp --> anchor
  anchor --> payload["payload"] --> out["Translated::Snap(P)"]
```

Translation answers "which abstract action does an off-tree opponent bet
correspond to?" An observed size becomes a `Scalar<A>` (phantom-typed to an
axis like big blinds or pot fraction) and is located within a `Lattice<A, P>`,
a validated strictly-ascending list of anchor scalars each carrying a payload
`P`. `bracket` finds the surrounding `(lo, hi)` anchors (or clamps at an
extreme); the `Translation` policy then picks one: `Snap` takes the L1-nearest
anchor, while `Harmonic` and `Phargmax` apply the Ganzfried–Sandholm 2013
pseudo-harmonic weighting (randomized vs. deterministic argmax). The chosen
`Anchor` yields its payload as `Translated::Snap(P)`.
