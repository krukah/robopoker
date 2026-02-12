/// A probability distribution over actions (edges).
///
/// Maps each edge to its probability weight. Used throughout CFR for:
/// - Current iteration strategy (what the player actually does)
/// - Regret increments (how much better each action would have been)
/// - Accumulated average strategy (the converged Nash equilibrium)
///
/// # Implementation
///
/// Uses a `Vec` rather than `HashMap`/`BTreeMap` for better cache locality
/// and lower overhead with small action counts (typically 2-10 in poker).
/// Benchmarks on RPS (3 actions) confirm Vec outperforms map-based versions.
///
/// Lookup is O(n) but n is small, and the Vec representation enables
/// efficient iteration which dominates CFR computation.
pub type Policy<E> = Vec<(E, rbp_core::Probability)>;
