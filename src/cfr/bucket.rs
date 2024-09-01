use crate::clustering::abstraction::Abstraction;

/// Product of Information Abstraction , Action Abstraction
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Bucket(Abstraction);

impl Bucket {
    pub const P1: Self = Self(todo!());
    pub const P2: Self = Self(todo!());
    pub const IGNORE: Self = Self(todo!());
}
