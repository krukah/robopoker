use crate::*;

/// How the hand settled — the optional terminal layer of a scene.
///
/// A settlement is a *major* column, the same rank of thing as a street boundary: a band of
/// tiles that closes the tree rather than branching it. `badge` names the seat that took the
/// pot (the actor band), `verdict` names the manner it ended (the phase band), and `tokens` are
/// the result tiles themselves — pot, net, whatever the game cares to show.
#[derive(Debug, Clone)]
pub struct Tally {
    badge: String,
    verdict: String,
    tokens: Vec<Token>,
}

impl Tally {
    pub fn new(badge: impl Into<String>, verdict: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            badge: badge.into(),
            verdict: verdict.into(),
            tokens,
        }
    }

    pub fn badge(&self) -> &str {
        &self.badge
    }

    pub fn verdict(&self) -> &str {
        &self.verdict
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
}
