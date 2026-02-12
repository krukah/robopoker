//! NLHE private state: hand abstraction bucket.
use rbp_cards::Street;
use rbp_gameplay::Abstraction;
use rbp_mccfr::*;
use rbp_transport::Support;

/// NLHE private information: the player's hand abstraction bucket.
///
/// Newtype wrapper around gameplay `Abstraction` for NLHE-specific CFR.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NlheSecret(Abstraction);

impl NlheSecret {
    /// The street this abstraction belongs to.
    pub fn street(&self) -> Street {
        self.0.street()
    }
}

impl Default for NlheSecret {
    fn default() -> Self {
        Self(Abstraction::default())
    }
}

impl Support for NlheSecret {}
impl CfrSecret for NlheSecret {}

impl From<Abstraction> for NlheSecret {
    fn from(abs: Abstraction) -> Self {
        Self(abs)
    }
}
impl From<NlheSecret> for Abstraction {
    fn from(secret: NlheSecret) -> Self {
        secret.0
    }
}
impl AsRef<Abstraction> for NlheSecret {
    fn as_ref(&self) -> &Abstraction {
        &self.0
    }
}
impl From<i16> for NlheSecret {
    fn from(val: i16) -> Self {
        Self(Abstraction::from(val))
    }
}
impl From<NlheSecret> for i16 {
    fn from(secret: NlheSecret) -> Self {
        i16::from(Abstraction::from(secret))
    }
}

impl std::fmt::Display for NlheSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Abstraction::from(*self))
    }
}
