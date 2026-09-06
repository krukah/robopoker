/// Greedy approximation for optimal transport: sort all (source, target) pairs
/// by ground cost, then push as much mass as possible along each edge in turn.
/// O(n² log n), dominated by the sort. No optimality guarantee, but close
/// enough on well-behaved distributions.
pub struct GreedyOptimalTransport;
