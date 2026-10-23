/// Multiplier of the classic glibc LCG.
pub const MUL: u64 = 1_103_515_245;

/// Increment of the classic glibc LCG.
pub const INC: u64 = 12_345;

/// 31-bit state mask.
pub const MASK: u64 = 0x7fff_ffff;

/// Deterministic linear congruential generator: every "stochastic-looking"
/// choice in the reel is seeded and reproducible — chosen, not random.
#[derive(Debug, Clone)]
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(seed & MASK)
    }

    /// The next sample in [0, 1).
    pub fn roll(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(MUL).wrapping_add(INC) & MASK;
        self.0 as f64 / MASK as f64
    }
}
