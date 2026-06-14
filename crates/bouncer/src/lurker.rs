use pokerkit::ID;
use pokerkit::Unique;

/// Anonymous session tracking for unauthenticated visitors.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lurker {
    id: ID<Lurker>,
}

impl Unique for Lurker {
    fn id(&self) -> ID<Lurker> {
        self.id
    }
}
