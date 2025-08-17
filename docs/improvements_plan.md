## MCCFR Training Improvements Plan

### Objectives
- Reduce per-iteration CPU time without changing algorithmic behavior.
- Minimize allocations and redundant computations on hot paths.
- Maintain determinism and readability.

### High-impact optimizations
- **Keep the batch pipeline parallel end-to-end**
  - Today `train().batch()` materializes intermediate `Vec`s between parallel stages.
  - Action: replace collect/barriers in `batch()` (in `src/mccfr/traits/blueprint.rs`) with a single parallel pipeline that builds trees and computes counterfactuals without intermediate serial collections.

- **Cache denominators and shared sums per infoset**
  - `policy_vector`, `policy`, `advice`, `sample` recompute sums repeatedly.
  - Action: compute once per infoset call and reuse within the same vector construction (file: `src/mccfr/traits/profile.rs`).

- **Stream DFS instead of building leaf vectors**
  - `expected_value`/`cfactual_value` rely on `descendants()` which allocates a full `Vec` of leaves.
  - Action: implement streaming DFS accumulation that sums utilities without materializing all leaves (files: `src/mccfr/traits/profile.rs`, consider adding a helper on `Node`).

- **Factor and cache reach probabilities**
  - `relative_value` recomputes path products; `sampling_reach` recomputes common root factors.
  - Action: cache partial products during DFS; factor out common root sampling reach once per infoset (file: `src/mccfr/traits/profile.rs`).

- **Reduce allocations in branching and choices**
  - `Node::branches()`/`Encoder::branches()` and `Info::choices()` allocate `Vec`s repeatedly.
  - Action: switch to `smallvec::SmallVec` for small, fixed-capacity return values; or return iterators (files: `src/mccfr/structs/node.rs`, `src/mccfr/traits/encoder.rs`, `src/mccfr/nlhe/info.rs`).

- **Use faster hash maps for profile storage**
  - `Profile.encounters` uses nested `BTreeMap`s.
  - Action: migrate to `hashbrown::HashMap` or `ahash::AHashMap` for O(1) average lookups (file: `src/mccfr/nlhe/profile.rs`).

- **Cheaper weighted sampling**
  - `explore_one` builds `WeightedIndex` each time.
  - Action: sample via cumulative weights + single RNG draw; reuse per-thread `SmallRng` (file: `src/mccfr/traits/profile.rs`).

### Additional opportunities
- **Avoid repeated raises counting**
  - `nlhe::encoder::info()` derives raise depth by scanning history each call.
  - Action: compute raises once when forming `Info` or carry it alongside `Path` (files: `src/mccfr/nlhe/encoder.rs`, `src/gameplay/path.rs`).

- **I/O**
  - Ensure buffered writes for checkpoints; already mostly buffered. Consider batching save frequency.

### Build/profile settings
- Release flags: `-C target-cpu=native`, `lto = "fat"`, `panic = "abort"` for training binaries.
- Enable `rayon` thread pool sizing based on available cores; expose batch size as a CLI/env to tune at runtime.

### Benchmarking plan
- Add Criterion benches for:
  - `batch()` end-to-end throughput
  - `tree()` generation
  - `regret_vector()` and `policy_vector()`
  - `expected_value()`/`cfactual_value()`
- Use representative seeds and fixed RNG for determinism.

### Implementation checklist
- [ ] Replace barriers in `batch()` parallel pipeline.
- [ ] Cache denominators in policy/regret computations.
- [ ] Streaming DFS accumulation for utilities; remove `descendants()` allocations on hot path.
- [ ] Switch `Profile.encounters` maps to `hashbrown`/`ahash`.
- [ ] SmallVec/iterator returns for `branches()` and `choices()`.
- [ ] Replace `WeightedIndex` with cumulative sampling and per-thread RNG reuse.
- [ ] Optional: carry raises count with `Path`/`Info` to avoid rescans.
- [ ] Add Criterion benches and wire a `--bench` target.

### Notes
- No string parsing is on the hot path; core costs are tree expansion, policy/regret math, and equity/utility aggregation. The above focuses on those.

