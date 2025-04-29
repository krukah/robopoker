use crate::clustering::abstraction::Abstraction;
use crate::gameplay::path::Path;
use crate::Arbitrary;
use std::hash::Hash;

/// can't tell whether default bucket makes sense.
/// it requires default absttraction, for one, which
/// i guess would just be P::00, but it can't be derived in [Abstraction]
/// because of the Middle bit hashing we do in [Abstraction]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Info {
    history: Path,
    present: Abstraction,
    futures: Path,
}

impl Info {
    pub fn history(&self) -> &Path {
        &self.history
    }
    pub fn present(&self) -> &Abstraction {
        &self.present
    }
    pub fn futures(&self) -> &Path {
        &self.futures
    }
}

impl crate::cfr::traits::info::Info for Info {
    type E = crate::cfr::nlhe::edge::Edge;
    type T = crate::cfr::nlhe::turn::Turn;
    fn choices(&self) -> Vec<Self::E> {
        self.futures.into_iter().collect()
    }
}

impl From<(Path, Abstraction, Path)> for Info {
    fn from((history, present, futures): (Path, Abstraction, Path)) -> Self {
        Self {
            history,
            present,
            futures,
        }
    }
}
impl From<Info> for (Path, Abstraction, Path) {
    fn from(info: Info) -> Self {
        (info.history, info.present, info.futures)
    }
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}>>{}<<{}", self.history, self.present, self.futures)
    }
}

impl Arbitrary for Info {
    fn random() -> Self {
        Self::from((Path::random(), Abstraction::random(), Path::random()))
    }
}
