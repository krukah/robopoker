use crate::*;
use deuce::*;
use mccfr::*;
use neural::*;

/// A perfect-recall NLHE infoset for Deep CFR: nothing abstracted but suit symmetry.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FullInfo(Composite<FullPublic, FullSecret>);

impl FullInfo {
    pub fn new(public: FullPublic, secret: FullSecret) -> Self {
        Self(Composite::new(public, secret))
    }

    pub fn street(&self) -> Street {
        self.public().history().street()
    }
}

impl CfrInfo for FullInfo {
    type X = FullPublic;
    type Y = FullSecret;

    fn public(&self) -> FullPublic {
        self.0.public()
    }

    fn secret(&self) -> FullSecret {
        self.0.secret()
    }

    fn join(public: FullPublic, secret: FullSecret) -> Self {
        Self::new(public, secret)
    }
}

/// The secret's block, then the public's: the whole row an [`NlheEmbedder`] feeds a net.
impl Block for FullInfo {
    const WIDTH: usize = FullSecret::WIDTH + FullPublic::WIDTH;

    fn values(self) -> impl Iterator<Item = f32> {
        self.secret().values().chain(self.public().values())
    }
}
