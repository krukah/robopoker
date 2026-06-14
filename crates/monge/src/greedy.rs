/// Greedy approximation algorithm for optimal transport.
///
/// Computes an approximate transport plan by greedily matching source
/// and target masses in order of increasing ground cost. While not
/// guaranteed to find the global optimum, this heuristic is fast and
/// provides reasonable approximations for well-behaved distributions.
///
/// # Algorithm
///
/// 1. Sort all (source, target) pairs by ground cost
/// 2. Greedily transport as much mass as possible along each edge
/// 3. Continue until all mass is transported
///
/// # Complexity
///
/// O(n² log n) where n is the support size, dominated by sorting pairs.
pub struct GreedyOptimalTransport;
