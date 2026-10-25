//! Slumbot as a [`Rival`].

use crate::client::*;
use crate::hand::*;
use crate::translate::*;
use bout::Outcome;
use bout::Recorder;
use bout::Rival;
use bout::Throttle;
use pokerkit::Adversary;
use pokerkit::Chips;

/// One Slumbot session — a token and the connection behind it.
///
/// Slumbot's token is per-session, not per-hand: it carries the running
/// series, so a session plays many hands through the same [`Client`] and the
/// harness only replaces the whole thing when a hand fails.
pub struct Session {
    client: Client,
}

impl Session {
    fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

#[async_trait::async_trait]
impl Rival for Session {
    /// Slumbot is open — no key, no registration, no rate-limit contract
    /// beyond not being rude about it.
    type Auth = ();

    const ADVERSARY: Adversary = Adversary::Slumbot;

    const STACK: Chips = SLUMBOT_STACK;

    fn fresh((): &Self::Auth, throttle: Throttle) -> Self {
        Self {
            client: Client::new(throttle),
        }
    }

    async fn play(&mut self, player: &mut dyn parlor::Player, recorder: &mut Recorder) -> anyhow::Result<Outcome> {
        Hand::play(self.client_mut(), player, recorder).await
    }
}
