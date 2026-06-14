/// Greenkhorn algorithm for entropic optimal transport.
///
/// A variant of Sinkhorn iteration that updates rows and columns greedily
/// based on marginal violation, rather than alternating uniformly.
/// This can accelerate convergence when the transport plan is sparse.
///
/// # Algorithm
///
/// 1. Initialize dual potentials
/// 2. At each step, identify the row or column with largest marginal error
/// 3. Update that row/column's potential to satisfy its marginal constraint
/// 4. Repeat until convergence
///
/// # References
///
/// Altschuler, J., Weed, J., & Rigollet, P. (2017).
/// "Near-linear time approximation algorithms for optimal transport via Sinkhorn iteration"
///
/// # Complexity
///
/// O(n² / ε²) iterations for ε-approximate solution, with O(n) work per iteration.
pub struct GreenhornOptimalTransport;
