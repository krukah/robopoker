/// Product of Information Abstraction , Action Abstraction
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(usize);

impl Bucket {
    pub const IGNORE: Self = Self(0);
    pub const P1: Self = Self(1);
    pub const P2: Self = Self(2);
}
