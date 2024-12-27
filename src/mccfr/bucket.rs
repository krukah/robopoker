use super::path::Path;
use crate::clustering::abstraction::Abstraction;
use crate::Arbitrary;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(pub Path, pub Abstraction, pub Path);

impl From<(Path, Abstraction, Path)> for Bucket {
    fn from((past, present, future): (Path, Abstraction, Path)) -> Self {
        Self(past, present, future)
    }
}
impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;
        write!(
            f,
            "{}{}{}",
            format!("{}>>", self.0).bright_cyan(),
            self.1,
            format!("<<{}", self.2).bright_cyan()
        )
    }
}

impl Arbitrary for Bucket {
    fn random() -> Self {
        Self::from((Path::random(), Abstraction::random(), Path::random()))
    }
}
