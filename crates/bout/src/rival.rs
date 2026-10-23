//! The one thing a benchmark client has to supply.

use crate::outcome::*;
use crate::recorder::*;
use crate::throttle::*;
use parlor::Player;
use pokerkit::Adversary;
use pokerkit::Chips;

/// A remote opponent's protocol, reduced to "play me one hand".
///
/// Everything around a hand — pacing, retries, statistics, hand-history rows, scoreboard rows,
/// telemetry — belongs to the harness and is written once. An implementor owns only the parts
/// that are genuinely service-specific: its transport, its wire vocabulary, and the translation
/// between that wire and our [`Witness`](kicker::Witness).
#[async_trait::async_trait]
pub trait Rival: Send + Sized {
    /// Whatever opening a session needs beyond a throttle — an API key, a
    /// base URL, nothing at all. Cloned once per session, so keep it cheap.
    type Auth: Clone + Send + Sync + 'static;

    /// Who is on the other side. Fixes the villain's hand-history identity,
    /// the `rbp.<service>.*` metric namespace, and the scoreboard's
    /// `opponent` column.
    const ADVERSARY: Adversary;

    /// Their starting stack expressed in our chip scale. Slumbot deals
    /// 200 BB, which is `B_BLIND * 200` for us.
    const STACK: Chips;

    /// Open a transport. Also the reconnect path — the harness discards a
    /// [`Rival`] and builds another after any failed hand.
    fn fresh(auth: &Self::Auth, throttle: Throttle) -> Self;

    /// Play exactly one hand to completion, recording it as it goes.
    async fn play(&mut self, player: &mut dyn Player, recorder: &mut Recorder) -> anyhow::Result<Outcome>;

    /// The service's own account of how we did, as a line to log at the end of a run.
    ///
    /// Worth having wherever the opponent scores us itself: it is an independent audit of our
    /// accounting, and a disagreement with our own tally means the translation dropped
    /// something rather than that we ran bad. Services that keep no such record return `None`,
    /// which is the default.
    async fn audit(&mut self) -> Option<String> {
        None
    }
}
