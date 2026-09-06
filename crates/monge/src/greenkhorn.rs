/// Greenkhorn: entropic optimal transport that, instead of alternating rows
/// and columns uniformly like Sinkhorn, repeatedly fixes whichever row or
/// column has the largest marginal error. Converges faster on sparse transport
/// plans — O(n² / ε²) iterations for an ε-approximation, O(n) work each.
///
/// Altschuler, Weed & Rigollet (2017), "Near-linear time approximation
/// algorithms for optimal transport via Sinkhorn iteration."
pub struct GreenhornOptimalTransport;
