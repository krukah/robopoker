# forge

Automated MCCFR training-pipeline orchestration that streams data directly to PostgreSQL.

## High-Level Overview

```mermaid
flowchart TD
  T["cargo run --bin trainer --<br/>[--status | --cluster | --fast | --slow | --deep]"]
  T --> S["status<br/>(query)"]
  T --> C["cluster<br/>(PreTraining)"]
  T --> TR["train<br/>(Trainer)"]
  C --> P1["Phase 1 · Clustering<br/>(offline)"]
  TR --> P2["Phase 2 · Blueprint<br/>(MCCFR)"]
  P1 -.->|required by| P2
```

1. **Clustering Phase**: Reduce 3.1 trillion poker situations into 782 abstract buckets (101 + 256 + 256 + 169)
2. **Blueprint Phase**: Train MCCFR strategies on the abstracted game tree

## Usage

```bash
# Check current state
cargo run --bin trainer -- --status

# Run clustering only
cargo run --bin trainer -- --cluster

# Drop every clustering table, so the next --cluster/--fast/--slow rebuilds the abstraction
cargo run --bin trainer -- --recluster

# Run fast in-memory training (includes clustering if needed)
cargo run --bin trainer -- --fast

# Run distributed training (includes clustering if needed)
cargo run --bin trainer -- --slow

# Drop the blueprint and truncate epoch, snapshot, fingerprint; --forget truncates hand histories
cargo run --bin trainer -- --reset

# Deep CFR on NLHE (FullInfo + NlheEmbedder); no database. Saves the strategy net to DIR
# (default ./nets) every TrainingHyperParams flush interval and on stop
cargo run --release --bin trainer -- --deep [DIR]
```

---

## Phase 1: Clustering (PreTraining)

**Entry Point:** `PreTraining::run()` in `src/pretraining.rs`; streets already in the `isomorphism` table are skipped.

The clustering phase processes streets in **reverse order** (River → Turn → Flop → Preflop) because each street's abstraction depends on the _next_ street's lookup and metric.

### Reverse Dependency Chain

```mermaid
flowchart LR
  River["RIVER<br/>123M obs · K=101<br/>Lookup::grow()"] -->|lookup| Turn["TURN<br/>14M obs · K=256<br/>Layer::cluster()"]
  Turn -->|lookup + metric| Flop["FLOP<br/>1.3M obs · K=256<br/>Layer::cluster()"]
  Flop -->|lookup + metric| Preflop["PREFLOP<br/>169 obs · K=169<br/>Layer::cluster()"]
```

Each street produces a `lookup` (iso→abs), plus `metric` and `future` (except River); the next-coarser street loads those artifacts.

### Street Parameters

| Street  | N           | K   | Metric     | Space                  | Loads                    | Produces                   |
| ------- | ----------- | --- | ---------- | ---------------------- | ------------------------ | -------------------------- |
| River   | 123,156,254 | 101 | `f32::abs` | `Probability`          | N/A                      | lookup                     |
| Turn    | 13,960,050  | 256 | `W1`       | `Density<Probability>` | Lookup River             | lookup, metric, transition |
| Flop    | 1,286,792   | 256 | `EMD`      | `Density<Abstraction>` | Lookup Turn, Metric Turn | lookup, metric, transition |
| Preflop | 169         | 169 | `EMD`      | `Density<Abstraction>` | Lookup Flop, Metric Flop | lookup, metric, transition |

- **N**: Number of isomorphic observations on this street
- **K**: Number of abstraction clusters (river uses equity buckets 0-100%)
- **Metric**: Distance function for clustering (`W1` = Wasserstein-1, `EMD` = Earth Mover's Distance)
- **Space**: Element type being compared in the metric

### Per-Street Processing Detail

#### River Clustering (`Lookup::grow`)

```mermaid
flowchart TD
  I["IsomorphismIterator(River)"] --> E["iso.equity()<br/>exact hand strength, ties half"]
  E --> A["Abstraction from<br/>equity percent (0-100 buckets)"]
  A --> L["Lookup → isomorphism table"]
```

No k-means (equity buckets *are* the abstractions) and no metric (equity distance is just `|e1 - e2|`).

#### Turn / Flop / Preflop Clustering (`Layer::cluster()`)

```mermaid
flowchart TD
  S1["1 · LOAD DEPENDENCIES<br/>Metric + Lookup from next street"] --> S2["2 · BUILD HISTOGRAMS<br/>iso.children() → abstractions → Histogram"]
  S2 --> S3["3 · K-MEANS++ INIT<br/>sample ∝ D(x)² until K centroids"]
  S3 --> S4["4 · K-MEANS ITERS<br/>Elkan bounds (flop) · Hamerly bounds (turn)"]
  S4 --> S5["5 · PRODUCE ARTIFACTS<br/>lookup · metric · future → Postgres"]
```

### Data Flow Through Tables

```mermaid
flowchart TD
  R["River 123M · equity()"] --> ISOa["isomorphism (obs, abs)"]
  T["Turn 14M · children()+lookup"] --> TH["Histogram → Hamerly<br/>K=256, W1"]
  TH --> OUTt["isomorphism · metric · transitions"]
  F["Flop 1.3M"] --> FH["Histogram → Elkan<br/>K=256, EMD"]
  FH --> OUTf["isomorphism · metric · transitions"]
  P["Preflop 169"] --> PH["Histogram → 1:1<br/>K=169"]
  PH --> OUTp["isomorphism · metric · transitions"]
  OUTt -.->|metric| FH
  OUTf -.->|metric| PH
```

---

## Phase 2: Blueprint Training

**Entry Point:** `Mode::run()` in `src/mode.rs`; the loop is the default `Trainer::train()` in `src/trainer.rs`.

```mermaid
flowchart TD
  M["Mode::run()"] --> FS["FastSession::new (--fast)<br/>Flagship · in-memory"]
  M --> SS["SlowSession::new (--slow)<br/>Pool of Workers · Postgres"]
  FS --> RC["PreTraining::run()<br/>cluster if needed · Fingerprint::check"]
  SS --> RC
  RC -->|"loop: step() → checkpoint() → flush()"| TT["Trainer::train()"]
  TT -->|"fast: every flush interval and on exit"| FSY["stage → COPY → merge → stamp(n) → snapshot"]
  TT -->|"slow"| SSY["direct writes (no-op sync)"]
  FSY --> DB[("PostgreSQL<br/>blueprint · epoch · snapshot")]
  SSY --> DB
```

### Fast vs Slow Mode Comparison

```mermaid
flowchart TB
  subgraph FAST["FAST MODE (fast.rs)"]
    direction TB
    F1["Flagship<br/>regret · policy in memory"] --> F2["step()"]
    F2 --> F3["single box · memory-bound"]
    F3 --> F4["flush every interval, sync on exit<br/>stage · COPY · merge · stamp(n)"]
  end
  subgraph SLOW["SLOW MODE (slow.rs)"]
    direction TB
    S1["Pool of Worker<br/>one per core"] --> S2["step() async<br/>(tokio)"]
    S2 --> S3["direct DB writes"]
    S3 --> S4["already persisted<br/>(no sync needed)"]
  end
```

- **Fast**: memory-bound, single machine.
- **Slow**: state lives in Postgres, I/O-bound.

Both implement the `Trainer` trait:

```rust
#[async_trait]
pub trait Trainer: Send + Sync + Sized {
    fn client(&self) -> &Arc<Client>;
    async fn sync(self);
    async fn step(&mut self);
    async fn epoch(&self) -> usize;
    async fn summary(&self) -> String;
    async fn checkpoint(&self) -> Option<Checkpoint>;
    async fn flush(&mut self) {}
    fn session_type(&self) -> &'static str { "generic" }
    async fn train(mut self) { ... }
}
```

`--deep [DIR]` takes neither path: `DeepSession` trains Deep CFR on `nlhe::FullInfo` with no database.

---

## Database Schema

### Core Tables

```mermaid
flowchart LR
  subgraph CL["clustering tables"]
    iso["isomorphism ~138M<br/>obs BIGINT · abs SMALLINT"]
    met["metric<br/>tri INT · dx REAL"]
    tr["transitions<br/>prev · next · dx (weight)"]
  end
  subgraph BP["training tables"]
    bp["blueprint (grows)<br/>past · present · choices · context · edge<br/>weight · regret · payoff · visits"]
    ep["epoch (1 row)<br/>key TEXT · value BIGINT"]
  end
```

- `isomorphism` maps every isomorphic hand to its abstraction bucket.
- `metric` holds pairwise abstraction distances, used by the previous street's EMD.
- `transitions` holds the distribution over next-street abstractions per abstraction.
- `blueprint` stores the MCCFR strategy per information set — upserted via a staging table on each flush (FastSession) or written directly by workers (SlowSession).

### Derived Tables

| Table         | Columns                           | Rows | Description                   |
| ------------- | --------------------------------- | ---- | ----------------------------- |
| `abstraction` | `abs, street, population, equity` | 782  | Summary stats per abstraction |
| `street`      | `street, nobs, nabs`              | 4    | Summary stats per street      |

---

## Streaming Protocol

All data uses **PostgreSQL binary COPY** via the `Streamable` trait:

```mermaid
flowchart LR
  R["T::rows() iterator"] --> W["BinaryCopyInWriter"] --> DB["PostgreSQL table"]
```

Implementors: `Lookup` (isomorphism), `Metric` (metric), `Future` (transitions). The blueprint streams the same way into `staging`.

---

## Resumability

| Feature           | Mechanism                                                 |
| ----------------- | --------------------------------------------------------- |
| Progress tracking | Skips streets already present in `isomorphism`            |
| Rebuild           | `--recluster` drops every clustering table                |
| Epoch persistence | `epoch` table tracks MCCFR iteration count                |
| Graceful shutdown | Press `Q + Enter` to finish current batch and sync        |

---

## Key Insight

> Clustering flows **backwards** (river → preflop) because each street's abstraction depends on the _next_ street's distribution, while training flows **forwards** through the game tree, building blueprint strategies via MCCFR iterations.
