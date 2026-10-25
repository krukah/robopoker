/// The two seats of a reducible pot, in heads-up roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seats {
    oop: usize,
    ip: usize,
}
impl Seats {
    pub fn new(oop: usize, ip: usize) -> Self {
        Self { oop, ip }
    }

    /// Acts first postflop — the big blind's role in our tree.
    pub fn oop(&self) -> usize {
        self.oop
    }

    /// Acts last postflop — the button's role in our tree.
    pub fn ip(&self) -> usize {
        self.ip
    }

    /// True if `seat` is one of the two.
    pub fn has(&self, seat: usize) -> bool {
        self.oop() == seat || self.ip() == seat
    }
}

impl Seats {}
