//! One hand against Slumbot, end to end.

use crate::client::*;
use crate::translate::*;
use bout::Outcome;
use bout::Recorder;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Hand-scoped state.
///
/// Slumbot reports the action string *cumulatively*, so every response is re-read against the
/// prefix we already knew and only the suffix is applied — hence `history`. The board can grow
/// between our actions, which the engine can't express as a push, so the witness is rebuilt
/// from the deal whenever new cards land.
pub struct Hand<'hand> {
    client: &'hand mut Client,
    recorder: &'hand mut Recorder,
    witness: Witness,
    history: String,
    hero: Turn,
    hole: (Card, Card),
    board: Vec<Card>,
}

impl<'hand> Hand<'hand> {
    /// Play one hand using the given player.
    #[tracing::instrument(skip_all, name = "slumbot.hand")]
    pub async fn play(
        client: &'hand mut Client,
        player: &mut dyn parlor::Player,
        recorder: &'hand mut Recorder,
    ) -> anyhow::Result<Outcome> {
        let response = client.new_hand().await?;
        let hero = ParseState::pov(response.client_pos());
        let hole = ParseState::parse_hole(response.hole_cards())?;
        let board = ParseState::parse_board(response.board())?;
        let Turn::Choice(seat) = hero else {
            unreachable!("hero is always a choice node")
        };
        let witness = Witness::initial_with(hero, ParseState::arrangement(hole, &board), [SLUMBOT_STACK; N], 0);
        recorder.begin(&witness.head(), seat);
        let mut hand = Self {
            client,
            recorder,
            witness,
            history: String::new(),
            hero,
            hole,
            board,
        };
        tracing::info!(
            hero = if seat == 0 { "SB" } else { "BB" },
            hole = %format!("{} {}", hole.0, hole.1),
            board = %hand.show(),
            raw = %response.action(),
            "hand start",
        );
        if !response.action().is_empty() {
            hand.villain(&ParseState::parse_actions(response.action(), "", &hand.witness().head())?)?;
            response.action().clone_into(hand.history_mut());
        }
        match response.winnings() {
            Some(w) => hand.settle(seat, w).await,
            None => loop {
                if let Some(outcome) = hand.act(player).await? {
                    return Ok(outcome);
                }
            },
        }
    }

    /// Hero decides, we encode and send, then apply whatever came back.
    /// `Some` once the hand is over.
    async fn act(&mut self, player: &mut dyn parlor::Player) -> anyhow::Result<Option<Outcome>> {
        let Turn::Choice(seat) = self.witness().head().turn() else {
            anyhow::bail!("slumbot left us at {:?} with no winnings", self.witness().head().turn());
        };
        anyhow::ensure!(Turn::Choice(seat) == self.hero(), "slumbot's turn but no action received");
        let game = self.witness().head();
        let clock = std::time::Instant::now();
        let action = player.decide(self.witness()).await;
        let incr = ParseState::encode_action(action, &game);
        tracing::trace!(snapped = ?game.snap(action), raw = ?action, %incr, "hero");
        self.recorder_mut()
            .record(seat, game.snap(action), Some(clock.elapsed().as_millis() as i32));
        *self.witness_mut() = self.witness().try_push(game.snap(action))?;
        self.history_mut().push_str(&incr);
        let response = self.client_mut().act(&incr).await?;
        tracing::trace!(
            action = %response.action(),
            board = %response.board().join(" "),
            winnings = ?response.winnings(),
            "slumbot",
        );
        self.refresh(response.board())?;
        let prior = std::mem::replace(self.history_mut(), response.action().to_owned());
        self.villain(&ParseState::parse_actions(self.history(), &prior, &self.witness().head())?)?;
        self.refresh(response.board())?;
        match response.winnings() {
            None => Ok(None),
            Some(w) => self.settle(seat, w).await.map(Some),
        }
    }

    /// Apply the opponent's actions to the witness and the record.
    fn villain(&mut self, actions: &[Action]) -> anyhow::Result<()> {
        let rival = self.rival();
        for &action in actions {
            tracing::trace!(?action, "villain");
            self.recorder_mut().record(rival, action, None);
            *self.witness_mut() = self.witness().try_push(action)?;
        }
        Ok(())
    }

    /// Close the hand out: settle the pot, write the record, report the bb.
    ///
    /// Slumbot reports winnings the moment a hand resolves, which can be one
    /// response ahead of a fold we never saw — so an unfinished witness gets
    /// the villain's fold applied before it is written, or the recorded hand
    /// would not replay.
    async fn settle(&mut self, seat: Position, winnings: i64) -> anyhow::Result<Outcome> {
        tracing::info!(bb = ParseState::to_bb(winnings), board = %self.show(), "hand result");
        if !self.witness().head().must_stop() {
            self.villain(&[Action::Fold])?;
        }
        let rival = self.rival();
        self.recorder_mut().pnl(seat, ParseState::to_chips(winnings));
        self.recorder_mut().pnl(rival, -ParseState::to_chips(winnings));
        self.recorder()
            .flush(self.witness(), self.witness().head().board(), self.witness().head().pot())
            .await;
        Ok(Outcome::new(self.hero(), ParseState::to_bb(winnings)))
    }

    /// Rebuild the witness when the board has grown. The engine can't push a
    /// street's cards onto an existing line, so the hand is re-arranged from
    /// the deal with the choice actions replayed.
    fn refresh(&mut self, raw: &[String]) -> anyhow::Result<()> {
        let fresh = ParseState::parse_board(raw)?;
        if fresh.len() > self.board().len() {
            *self.witness_mut() = Witness::try_arrange_with(
                self.hero(),
                ParseState::arrangement(self.hole(), &fresh),
                [SLUMBOT_STACK; N],
                self.witness()
                    .actions()
                    .iter()
                    .copied()
                    .filter(Action::is_choice)
                    .collect(),
            )?;
            *self.board_mut() = fresh;
        }
        Ok(())
    }

    /// The seat opposite hero.
    fn rival(&self) -> Position {
        match self.hero() {
            Turn::Choice(0) => 1,
            _ => 0,
        }
    }

    /// The board as a space-separated string, for logs.
    fn show(&self) -> String {
        self.board()
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn client_mut(&mut self) -> &mut Client {
        self.client
    }

    fn recorder(&self) -> &Recorder {
        self.recorder
    }

    fn recorder_mut(&mut self) -> &mut Recorder {
        self.recorder
    }

    fn witness(&self) -> &Witness {
        &self.witness
    }

    fn witness_mut(&mut self) -> &mut Witness {
        &mut self.witness
    }

    fn history(&self) -> &str {
        &self.history
    }

    fn history_mut(&mut self) -> &mut String {
        &mut self.history
    }

    fn hero(&self) -> Turn {
        self.hero
    }

    fn hole(&self) -> (Card, Card) {
        self.hole
    }

    fn board(&self) -> &[Card] {
        &self.board
    }

    fn board_mut(&mut self) -> &mut Vec<Card> {
        &mut self.board
    }
}
