use super::path::Path;
use crate::clustering::abstraction::Abstraction;
use crate::Arbitrary;
use std::hash::Hash;

/// can't tell whether default bucket makes sense.
/// it requires default absttraction, for one, which
/// i guess would just be P::00, but it can't be derived in [Abstraction]
/// because of the Middle bit hashing we do in [Abstraction]
#[derive(Debug, /* Default, */ Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(pub Path, pub Abstraction, pub Path);

impl From<(Path, Abstraction, Path)> for Bucket {
    fn from((past, present, future): (Path, Abstraction, Path)) -> Self {
        Self(past, present, future)
    }
}
impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}>>{}<<{}", self.0, self.1, self.2)
    }
}

impl Arbitrary for Bucket {
    fn random() -> Self {
        Self::from((Path::random(), Abstraction::random(), Path::random()))
    }
}
