# telemetry

OpenTelemetry pipeline for Rust-side metrics, tracing, and logging.

## Metric registry (single source of truth)

OTLP metric handles are centrally registered in `crates/telemetry/src/metrics.rs`. Add new handles there, never mint instruments at call sites — central ownership prevents typo-induced cardinality explosions and gives a single audit surface.

The TypeScript side mirrors this: `infra/pulumi/observability/lib/metrics.ts` exports an `M` registry with typed `Counter` / `Gauge` / `Histogram` builders that emit standard PromQL forms. **When adding a metric to `metrics.rs`, also add it to the TS registry.** The two are intended to mirror each other.

```typescript
M.mccfr.steps.rateSum(F); // sum(rate(rbp_mccfr_steps_total{…}[5m]))
M.kmeans.iteration_ms.quantile(0.95, ["street"], F);
M.kmeans.cluster_size.heatmap(F);
M.subgame.relative_regret.bucket(F);
```

Histogram bucket boundaries are configured via `histogram_view()` in `crates/telemetry/src/lib.rs` — if your histogram has values outside the OTLP default bucket range `[0..10000]`, register explicit boundaries.

## Structured tracing

Per the log→tracing migration (PR #75), all logging uses `tracing` not `log`. Tracing fields are key-value structured (NOT positional format strings):

```rust
// ✅ Good — queryable in CloudWatch Insights / Loki
tracing::info!(%street, phase = phase::HYDRATE, "kmeans phase begin");
tracing::debug!(iter = step.index, drift_max = step.drift.max(), "kmeans step");

// ❌ Bad — opaque to log queries
tracing::info!("{:<32}{:<32}", "kmeans hydrating", street);
```

**Field names that double as metric labels MUST be aliased to constants.** Example: `crates/clustering/src/telemetry.rs::phase::{HYDRATE, INIT, …}` — the same string is used in both the tracing event field and the metric label, so they can't drift. Apply this pattern whenever a tracing field shares a name with a metric label.

## Side effects via `Option::inspect`

For one-shot side-effect blocks gated on an Option, prefer chained `.inspect`:

```rust
frozen_at
    .inspect(|_| tel.rms(iter.rms() as f64))
    .inspect(|_| tel.early_terminated())
    .inspect(|i| tracing::info!(%street, iter = i + 1, total, "kmeans freeze"));
```

…instead of imperative `if let Some(x) = …` blocks. Each side effect reads as one line in a visible o11y pipe. Works even with read-then-write across closures (FnOnce closures are consumed sequentially).

## Cost intuition for adding metrics

Tracing/OTLP record cost is roughly **100-500 ns per histogram observation** (no labels, explicit-bucket fast path). For:

- **K-means**: every loop is bounded at compile time by const generics (K=128/144 clusters, t=20/24 iters). Per-iter cost: ~K records × ~100 ns ≈ tens-of-µs. Total iterate phase: ~ms against hours of EMD work. **Truly bounded; recording everything is free.**
- **MCCFR**: `mccfr_infoset_size` emits one record per filtered infoset per tree per batch (`batch_size=128 × ~K_infosets`, runtime-bounded). At ~200K infosets/batch this is ~20-100 ms per batch. Acceptable when batches take seconds; problematic if they're sub-second. Sample via a `MCCFR_INFOSET_SAMPLE_EVERY` constant if needed.

When in doubt, **walk the const-generic chain to confirm compile-time bounds**, then estimate per-batch cost from runtime-bounded loops. Anything growing with N points / K clusters / batch_size × infosets needs an audit.

## Dashboards

See `infra/pulumi/observability/CLAUDE.md` for dashboard-side conventions (file-to-dashboard mapping, panel layout rules, builder usage).
