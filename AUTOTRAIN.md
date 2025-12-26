# Autotrain Pipeline

Unified training pipeline that streams data directly to PostgreSQL.

## High-Level Overview

```
                          AUTOTRAIN PIPELINE

    cargo run --bin trainer --features server -- [--status|--cluster|--fast|--slow]
                                    |
         +--------------------------+---------------------------+
         v                          v                           v
    +---------+              +--------------+             +------------+
    | status  |              |   cluster    |             |   train    |
    | (query) |              | (PreTraining)|             | (Session)  |
    +---------+              +--------------+             +------------+
                                    |                           |
                                    |                           |
                             +------+------+              +-----+-----+
                             |  PHASE 1    |              |  PHASE 2  |
                             | Clustering  |---requires---|  Blueprint|
                             | (offline)   |              |  (MCCFR)  |
                             +-------------+              +-----------+
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

**Entry Point:** `PreTraining::run()` in `src/autotrain/pretraining.rs`

The clustering phase processes streets in **reverse order** (River -> Turn -> Flop -> Preflop) because each street's abstraction depends on the _next_ street's lookup and metric.

### Reverse Dependency Chain

```
                     CLUSTERING: REVERSE DEPENDENCY CHAIN

     +---------------+      +---------------+      +---------------+      +---------------+
     |    RIVER      |      |     TURN      |      |     FLOP      |      |   PREFLOP     |
     |  123M obs     |      |   14M obs     |      |   1.3M obs    |      |   169 obs     |
     |  K=101        |      |   K=144       |      |   K=128       |      |   K=169       |
     +-------+-------+      +-------+-------+      +-------+-------+      +-------+-------+
             |                      |                      |                      |
             | Lookup::grow()       | Layer::cluster()     | Layer::cluster()     | Layer::cluster()
             |                      |                      |                      |
             v                      v                      v                      v
     +---------------+      +---------------+      +---------------+      +---------------+
     |   produces:   |      |   produces:   |      |   produces:   |      |   produces:   |
     |   * lookup    |------|   * lookup    |------|   * lookup    |------|   * lookup    |
     |               |  ^   |   * metric    |  ^   |   * metric    |  ^   |   * metric    |
     |               |  |   |   * future    |  |   |   * future    |  |   |   * future    |
     +---------------+  |   +---------------+  |   +---------------+  |   +---------------+
                        |           |          |           |          |
                        |           |          |           |          |
                        +---loads---+          +---loads---+          +---loads---
                         lookup               lookup + metric        lookup + metric
```

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

#### River Clustering

```
                           RIVER CLUSTERING
                           (pretraining.rs:31)

   IsomorphismIterator::from(River)
              |
              v
   +-------------------+     +-----------------+
   | foreach 123M iso  |---->| iso.equity()    |  // Monte Carlo hand strength
   +-------------------+     +--------+--------+
                                      |
                                      v
                            +-----------------+
                            | Abstraction from|  // 0-100 equity buckets
                            | equity percent  |
                            +--------+--------+
                                     |
                                     v
                              +-------------+
                              |   Lookup    |  // stream to isomorphism table
                              +-------------+

   NO k-means (equity buckets are the abstractions directly)
   NO metric  (equity distance is just |e1 - e2|)
```

#### Turn / Flop / Preflop Clustering

```
                     TURN / FLOP / PREFLOP CLUSTERING
                     (layer.rs Layer::cluster())

   +-----------------------------------------------------------------------+
   | STEP 1: LOAD DEPENDENCIES                                             |
   |         Layer::build() loads from postgres                            |
   |                                                                       |
   |    Metric::from_street(next_street)  --> pairwise abstraction distances
   |    Lookup::from_street(next_street)  --> iso->abs mappings            |
   +-----------------------------------------------------------------------+
                        |
                        v
   +-----------------------------------------------------------------------+
   | STEP 2: BUILD HISTOGRAMS                                              |
   |         lookup.projections() for each isomorphism                     |
   |                                                                       |
   |   foreach Isomorphism on this street:                                 |
   |       +------------------------------------------------------------+  |
   |       | iso.children()  // all possible next-street observations   |  |
   |       |      |                                                     |  |
   |       |      v                                                     |  |
   |       | map to abstractions via loaded Lookup                      |  |
   |       |      |                                                     |  |
   |       |      v                                                     |  |
   |       | collect into Histogram (distribution over abstractions)    |  |
   |       +------------------------------------------------------------+  |
   +-----------------------------------------------------------------------+
                        |
                        v
   +-----------------------------------------------------------------------+
   | STEP 3: K-MEANS++ INITIALIZATION (elkan.rs init_kmeans)               |
   |                                                                       |
   |   1. Sample first centroid uniformly from dataset                     |
   |   2. For each remaining centroid:                                     |
   |      - Compute D(x)^2 = min distance to existing centroids            |
   |      - Sample next centroid with probability proportional to D(x)^2   |
   |   3. Repeat until K centroids                                         |
   +-----------------------------------------------------------------------+
                        |
                        v
   +-----------------------------------------------------------------------+
   | STEP 4: ELKAN K-MEANS ITERATIONS (elkan.rs next_eklan)                |
   |                                                                       |
   |   for t iterations:                                                   |
   |       +------------------------------------------------------------+  |
   |       | a. Compute pairwise centroid distances                     |  |
   |       | b. Compute midpoints s(c) = 1/2 min_{c'!=c} d(c,c')        |  |
   |       | c. Exclude points where upper_bound <= s(c) (triangle ineq)|  |
   |       | d. For remaining points, update bounds and assignments     |  |
   |       | e. Recompute centroids from assignments (Absorb trait)     |  |
   |       | f. Shift bounds by centroid drift                          |  |
   |       +------------------------------------------------------------+  |
   +-----------------------------------------------------------------------+
                        |
                        v
   +-----------------------------------------------------------------------+
   | STEP 5: PRODUCE ARTIFACTS & STREAM TO POSTGRES                        |
   |                                                                       |
   |   layer.lookup()  --> isomorphism table (obs, abs)                    |
   |   layer.metric()  --> metric table (xor, dx)                          |
   |   layer.future()  --> transitions table (prev, next, dx)              |
   +-----------------------------------------------------------------------+
```

### Data Flow Through Tables

```
                        CLUSTERING DATA FLOW

    Isomorphism                    K-Means                       PostgreSQL
    Space                          Clustering                    Tables
    ----------                     ----------                    ------

    River (123M)
         |
         | equity()
         v
    +----------+
    |0-100 eqty|--------------------------------------------------------> isomorphism
    +----------+                                                            (obs, abs)
         |
    =====|=====================================================================
         |
    Turn (14M)
         |
         | children() + lookup
         v
    +--------------+      +---------------+
    | Histogram    |------| Elkan K-Means |
    | per iso      |      | K=144, EMD    |
    +--------------+      +-------+-------+
                                  |
                    +-------------+-------------+
                    v             v             v
               isomorphism    metric      transitions
               (obs, abs)    (xor, dx)   (prev,next,dx)
                    |
    ================|==========================================================
                    |
    Flop (1.3M)     |
         |          | load
         | children() + lookup
         v          v
    +--------------+      +---------------+
    | Histogram    |------| Elkan K-Means |<--- metric (turn)
    | per iso      |      | K=128, EMD    |
    +--------------+      +-------+-------+
                                  |
                    +-------------+-------------+
                    v             v             v
               isomorphism    metric      transitions
                    |
    ================|==========================================================
                    |
    Preflop (169)   |
         |          | load
         | children() + lookup
         v          v
    +--------------+      +---------------+
    | Histogram    |------| 1:1 mapping   |<--- metric (flop)
    | per iso      |      | K=169         |
    +--------------+      +-------+-------+
                                  |
                    +-------------+-------------+
                    v             v             v
               isomorphism    metric      transitions
```

---

## Phase 2: Blueprint Training

**Entry Point:** `Trainer::train()` in `src/autotrain/trainer.rs`

```
                           BLUEPRINT TRAINING

                        Trainer::train()
                              |
                              | first: cluster() if needed
                              v
                 +------------------------+
                 |   require_clustering   |
                 |   PreTraining::run()   |
                 +-----------+------------+
                              |
                              | then: training loop
                              v
           +------------------+------------------+
           |                                     |
           v                                     v
    +-----------------+                  +-----------------+
    |   FastSession   |                  |   SlowSession   |
    |   (--fast)      |                  |   (--slow)      |
    +--------+--------+                  +--------+--------+
             |                                    |
             v                                    v
    +-----------------+                  +-----------------+
    |   NlheSolver    |                  |      Pool       |
    |   (in-memory)   |                  |  (distributed)  |
    +--------+--------+                  +--------+--------+
             |                                    |
             |                                    |
    =========|====================================|===========================
             |        TRAINING LOOP               |
    =========|====================================|===========================
             |                                    |
             | loop {                             | loop {
             |   solver.step()                    |   pool.step().await
             |   checkpoint()                     |   checkpoint()
             |   if Q+Enter: break                |   if Q+Enter: break
             | }                                  | }
             |                                    |
    =========|====================================|===========================
             |           SYNC                     |
    =========|====================================|===========================
             |                                    |
             v                                    v
    +-----------------+                  +-----------------+
    | client.stage()  |                  |    (no-op)      |
    | COPY rows       |                  |   direct writes |
    | client.merge()  |                  |   to blueprint  |
    | client.stamp(n) |                  |                 |
    +--------+--------+                  +--------+--------+
             |                                    |
             +--------------+---------------------+
                            v
                   +----------------+
                   |   PostgreSQL   |
                   |   ----------   |
                   |   blueprint    |
                   |   epoch        |
                   +----------------+
```

### Fast vs Slow Mode Comparison

```
+--------------------------------+--------------------------------+
|         FAST MODE              |         SLOW MODE              |
|         (fast.rs)              |         (slow.rs)              |
+--------------------------------+--------------------------------+
|                                |                                |
|  NlheSolver                    |  Pool<Worker<Postgres>>        |
|      |                         |      |                         |
|      v                         |      v                         |
|  +--------------+              |  +--------------+              |
|  |  BTreeMap    |              |  |  Worker 1    |--+           |
|  |  ----------  |              |  +--------------+  |           |
|  |  regret[k]   |              |  |  Worker 2    |--+-- async   |
|  |  policy[k]   |              |  +--------------+  |   queries |
|  |  (in-memory) |              |  |  Worker N    |--+           |
|  +--------------+              |  +--------------+              |
|         |                      |         |                      |
|         | step() is sync       |         | step() is async      |
|         | (spawn_blocking)     |         | (tokio)              |
|         v                      |         v                      |
|  +--------------+              |  +--------------+              |
|  |   100x       |              |  |   direct     |              |
|  |   faster     |              |  |   DB writes  |              |
|  |   single-box |              |  |   scales out |              |
|  +--------------+              |  +--------------+              |
|         |                      |         |                      |
|         | on graceful exit     |         | (no sync needed)     |
|         v                      |         v                      |
|  +--------------+              |  +--------------+              |
|  | sync():      |              |  |   already    |              |
|  |  stage()     |              |  |   persisted  |              |
|  |  COPY bulk   |              |  |              |              |
|  |  merge()     |              |  |              |              |
|  |  stamp(n)    |              |  |              |              |
|  +--------------+              |  +--------------+              |
|                                |                                |
|  * 100x more efficient         |  * Scales horizontally         |
|  * Memory-bound                |  * I/O-bound                   |
|  * Single machine              |  * Multi-machine ready         |
|                                |                                |
+--------------------------------+--------------------------------+
```

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

```
                          POSTGRES TABLES

+-----------------------------------------------------------------------------+
|                          CLUSTERING TABLES                                  |
+-----------------------------------------------------------------------------+
|                                                                             |
|  isomorphism (~139M rows)         |  metric (~40K rows)                     |
|  ---------------------            |  -----------------                      |
|  obs   BIGINT  --> Isomorphism    |  xor   BIGINT --> Pair(abs1 ^ abs2)     |
|  abs   BIGINT  --> Abstraction    |  dx    REAL   --> EMD distance          |
|                                   |                                         |
|  * Maps every isomorphic hand     |  * Pairwise abstraction distances       |
|    to its abstraction bucket      |  * Used by previous street's EMD        |
|                                   |                                         |
+-----------------------------------+-----------------------------------------+
|                                   |                                         |
|  transitions (~29K rows)          |  epoch (1 row)                          |
|  -----------------------          |  ------------                           |
|  prev  BIGINT  --> Abstraction    |  key   TEXT   = 'current'               |
|  next  BIGINT  --> Abstraction    |  value BIGINT --> iteration count       |
|  dx    REAL    --> weight         |                                         |
|                                   |  * Training progress counter            |
|  * Distribution over next-street  |                                         |
|    abstractions per abstraction   |                                         |
|                                   |                                         |
+-----------------------------------+-----------------------------------------+

+-----------------------------------------------------------------------------+
|                          BLUEPRINT TABLE                                    |
+-----------------------------------------------------------------------------+
|                                                                             |
|  blueprint (~200M+ rows, grows with training)                               |
|  -------------------------------------------                                |
|  past    BIGINT  --> past abstraction path                                  |
|  present BIGINT  --> current abstraction                                    |
|  future  BIGINT  --> future abstraction path                                |
|  edge    BIGINT  --> action encoding                                        |
|  policy  REAL    --> strategy probability                                   |
|  regret  REAL    --> cumulative regret                                      |
|                                                                             |
|  * MCCFR strategy stored per information set                                |
|  * Upserted via staging table on graceful exit (FastSession)                |
|  * Written directly by workers (SlowSession)                                |
|                                                                             |
+-----------------------------------------------------------------------------+
```

### Derived Tables

| Table         | Columns                           | Rows | Description                   |
| ------------- | --------------------------------- | ---- | ----------------------------- |
| `abstraction` | `abs, street, population, equity` | 542  | Summary stats per abstraction |
| `street`      | `street, nobs, nabs`              | 4    | Summary stats per street      |

---

## Streaming Protocol

All data uses **PostgreSQL binary COPY** in 100k row chunks via `Streamable` trait:

```
                         BINARY COPY STREAMING

   impl Streamable for T
        |
        v
   +----------------+      +----------------+      +----------------+
   |  T::rows()     |----->| BinaryCopyIn   |----->|   PostgreSQL   |
   |  iterator      |      | Writer         |      |   table        |
   +----------------+      +----------------+      +----------------+

   Implementors:
   * Lookup  (isomorphism table)
   * Metric  (metric table)
   * Future  (transitions table)
   * Profile (blueprint table via staging)
```

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

> Clustering flows **backwards** (river->preflop) because each street's abstraction depends on the _next_ street's distribution, while training flows **forwards** through the game tree building blueprint strategies via MCCFR iterations.
