# forge

Automated MCCFR training-pipeline orchestration that streams data directly to PostgreSQL.

## High-Level Overview

```mermaid
flowchart TD
  T["cargo run --bin trainer --features server --<br/>[--status | --cluster | --fast | --slow]"]
  T --> S["status<br/>(query)"]
  T --> C["cluster<br/>(PreTraining)"]
  T --> TR["train<br/>(Session)"]
  C --> P1["Phase 1 · Clustering<br/>(offline)"]
  TR --> P2["Phase 2 · Blueprint<br/>(MCCFR)"]
  P1 -.->|required by| P2
```

1. **Clustering Phase**: Reduce 3.1 trillion poker situations into ~500 abstract buckets
2. **Blueprint Phase**: Train MCCFR strategies on the abstracted game tree

## Usage

```bash
# Check current state
cargo run --bin trainer --features server -- --status

# Run clustering only
cargo run --bin trainer --features server -- --cluster

# Run fast in-memory training (includes clustering if needed)
cargo run --bin trainer --features server -- --fast

# Run distributed training (includes clustering if needed)
cargo run --bin trainer --features server -- --slow
```

---

## Phase 1: Clustering (PreTraining)

**Entry Point:** `PreTraining::run()` in `src/pretraining.rs`

The clustering phase processes streets in **reverse order** (River → Turn → Flop → Preflop) because each street's abstraction depends on the _next_ street's lookup and metric.

### Reverse Dependency Chain

```mermaid
flowchart LR
  River["RIVER<br/>123M obs · K=101<br/>Lookup::grow()"] -->|lookup| Turn["TURN<br/>14M obs · K=144<br/>Layer::cluster()"]
  Turn -->|lookup + metric| Flop["FLOP<br/>1.3M obs · K=128<br/>Layer::cluster()"]
  Flop -->|lookup + metric| Preflop["PREFLOP<br/>169 obs · K=169<br/>Layer::cluster()"]
```

Each street produces a `lookup` (iso→abs), plus `metric` and `future` (except River); the next-coarser street loads those artifacts.

### Street Parameters

| Street  | N           | K   | Metric     | Space                  | Loads                    | Produces                   |
| ------- | ----------- | --- | ---------- | ---------------------- | ------------------------ | -------------------------- |
| River   | 123,156,254 | 101 | `f32::abs` | `Probability`          | N/A                      | lookup                     |
| Turn    | 13,960,050  | 144 | `W1`       | `Density<Probability>` | Lookup River             | lookup, metric, transition |
| Flop    | 1,286,792   | 128 | `EMD`      | `Density<Abstraction>` | Lookup Turn, Metric Turn | lookup, metric, transition |
| Preflop | 169         | 169 | `EMD`      | `Density<Abstraction>` | Lookup Flop, Metric Flop | lookup, metric, transition |

- **N**: Number of isomorphic observations on this street
- **K**: Number of abstraction clusters (river uses equity buckets 0-100%)
- **Metric**: Distance function for clustering (`W1` = Wasserstein-1, `EMD` = Earth Mover's Distance)
- **Space**: Element type being compared in the metric

### Per-Street Processing Detail

#### River Clustering (`pretraining.rs`)

```mermaid
flowchart TD
  I["IsomorphismIterator(River)"] --> E["iso.equity()<br/>Monte Carlo hand strength"]
  E --> A["Abstraction from<br/>equity percent (0-100 buckets)"]
  A --> L["Lookup → isomorphism table"]
```

No k-means (equity buckets *are* the abstractions) and no metric (equity distance is just `|e1 - e2|`).

#### Turn / Flop / Preflop Clustering (`Layer::cluster()`)

```mermaid
flowchart TD
  S1["1 · LOAD DEPENDENCIES<br/>Metric + Lookup from next street"] --> S2["2 · BUILD HISTOGRAMS<br/>iso.children() → abstractions → Histogram"]
  S2 --> S3["3 · K-MEANS++ INIT<br/>sample ∝ D(x)² until K centroids"]
  S3 --> S4["4 · ELKAN K-MEANS ITERS<br/>triangle-inequality bounds · Absorb · drift shift"]
  S4 --> S5["5 · PRODUCE ARTIFACTS<br/>lookup · metric · future → Postgres"]
```

### Data Flow Through Tables

```mermaid
flowchart TD
  R["River 123M · equity()"] --> ISOa["isomorphism (obs, abs)"]
  T["Turn 14M · children()+lookup"] --> TH["Histogram → Elkan<br/>K=144, EMD"]
  TH --> OUTt["isomorphism · metric · transitions"]
  F["Flop 1.3M"] --> FH["Histogram → Elkan<br/>K=128, EMD"]
  FH --> OUTf["isomorphism · metric · transitions"]
  P["Preflop 169"] --> PH["Histogram → 1:1<br/>K=169"]
  PH --> OUTp["isomorphism · metric · transitions"]
  OUTt -.->|metric| FH
  OUTf -.->|metric| PH
```

---

## Phase 2: Blueprint Training

**Entry Point:** `Trainer::train()` in `src/trainer.rs`

```mermaid
flowchart TD
  TT["Trainer::train()"] -->|cluster if needed| RC["require_clustering<br/>PreTraining::run()"]
  RC --> FS["FastSession (--fast)<br/>Nlhe · in-memory"]
  RC --> SS["SlowSession (--slow)<br/>Pool · distributed"]
  FS -->|"loop: step() → checkpoint()"| FSY["sync: stage → COPY → merge → stamp(n)"]
  SS -->|"loop: step().await → checkpoint()"| SSY["direct writes (no-op sync)"]
  FSY --> DB[("PostgreSQL<br/>blueprint · epoch")]
  SSY --> DB
```

### Fast vs Slow Mode Comparison

```mermaid
flowchart TB
  subgraph FAST["FAST MODE (fast.rs)"]
    direction TB
    F1["Nlhe → BTreeMap<br/>regret[k] · policy[k] (in-memory)"] --> F2["step() sync<br/>(spawn_blocking)"]
    F2 --> F3["100× faster · single-box"]
    F3 --> F4["on graceful exit → sync()<br/>stage · COPY · merge · stamp(n)"]
  end
  subgraph SLOW["SLOW MODE (slow.rs)"]
    direction TB
    S1["Pool&lt;Worker&lt;Postgres&gt;&gt;<br/>Worker 1..N"] --> S2["step() async<br/>(tokio)"]
    S2 --> S3["direct DB writes · scales out"]
    S3 --> S4["already persisted<br/>(no sync needed)"]
  end
```

- **Fast**: 100× more efficient, memory-bound, single machine.
- **Slow**: scales horizontally, I/O-bound, multi-machine ready.

Both modes implement the `Trainer` trait for polymorphic training:

```rust
#[async_trait]
pub trait Trainer: Send + Sync + Sized {
    fn client(&self) -> &Arc<Client>;
    async fn sync(self);
    async fn step(&mut self);
    async fn epoch(&self) -> usize;
    async fn summary(&self) -> String;
    async fn checkpoint(&self) -> Option<String>;
}
```

---

## Database Schema

### Core Tables

```mermaid
flowchart LR
  subgraph CL["clustering tables"]
    iso["isomorphism ~139M<br/>obs BIGINT · abs BIGINT"]
    met["metric ~40K<br/>xor BIGINT · dx REAL (EMD)"]
    tr["transitions ~29K<br/>prev · next · dx (weight)"]
    ep["epoch (1 row)<br/>key TEXT · value BIGINT"]
  end
  subgraph BP["blueprint table"]
    bp["blueprint ~200M+ (grows)<br/>past · present · future · edge<br/>policy REAL · regret REAL"]
  end
```

- `isomorphism` maps every isomorphic hand to its abstraction bucket.
- `metric` holds pairwise abstraction distances, used by the previous street's EMD.
- `transitions` holds the distribution over next-street abstractions per abstraction.
- `blueprint` stores the MCCFR strategy per information set — upserted via a staging table on graceful exit (FastSession) or written directly by workers (SlowSession).

### Derived Tables

| Table         | Columns                           | Rows | Description                   |
| ------------- | --------------------------------- | ---- | ----------------------------- |
| `abstraction` | `abs, street, population, equity` | 542  | Summary stats per abstraction |
| `street`      | `street, nobs, nabs`              | 4    | Summary stats per street      |

---

## Streaming Protocol

All data uses **PostgreSQL binary COPY** in 100k row chunks via the `Streamable` trait:

```mermaid
flowchart LR
  R["T::rows() iterator"] --> W["BinaryCopyInWriter"] --> DB["PostgreSQL table"]
```

Implementors: `Lookup` (isomorphism), `Metric` (metric), `Future` (transitions), `Profile` (blueprint via staging).

---

## Resumability

| Feature           | Mechanism                                           |
| ----------------- | --------------------------------------------------- |
| Progress tracking | Queries `isomorphism` table to check completion     |
| Partial cleanup   | `truncate_street()` clears data before re-uploading |
| Epoch persistence | `epoch` table tracks MCCFR iteration count          |
| Graceful shutdown | Press `Q + Enter` to finish current batch and sync  |

---

## Key Insight

> Clustering flows **backwards** (river → preflop) because each street's abstraction depends on the _next_ street's distribution, while training flows **forwards** through the game tree, building blueprint strategies via MCCFR iterations.
