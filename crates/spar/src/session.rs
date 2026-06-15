use crate::client::*;
use crate::recorder::*;
use crate::result::*;
use crate::translate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Hand-scoped state for a single Slumbot session.
pub struct Session<'sesh> {
    client: &'sesh mut Client,
    recorder: &'sesh mut Recorder,
    witness: Witness,
    history: String,
    hero: Turn,
    hole: (Card, Card),
    board: Vec<Card>,
}

impl<'sesh> Session<'sesh> {
    /// Play a single hand against Slumbot using the given Player.
    #[tracing::instrument(skip_all, name = "slumbot.hand")]
    pub async fn play(
        client: &'sesh mut Client,
        player: &mut dyn parlor::Player,
        recorder: &'sesh mut Recorder,
    ) -> anyhow::Result<HandResult> {
        let resp = client.new_hand().await?;
        let hero = pov(resp.client_pos);
        let hole = parse_hole(&resp.hole_cards)?;
        let board = parse_board(&resp.board)?;
        let witness = Witness::initial_with(hero, arrangement(hole, &board), [SLUMBOT_STACK; N], 0);
        let history = String::new();
        let mut session = Self {
            client,
            recorder,
            witness,
            history,
            hero,
            hole,
            board,
        };
        let Turn::Choice(seat) = hero else { unreachable!() };
        let pos = if seat == 0 { "SB" } else { "BB" };
        tracing::info!(
            hero = pos,
            hole = %format!("{} {}", hole.0, hole.1),
            board = %session
                .board
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(" "),
            raw = %resp.action,
            "hand start",
        );
        session.recorder.begin(&session.witness.head(), seat);
        if !resp.action.is_empty() {
            for action in parse_actions(&resp.action, "", &session.witness.head())? {
                tracing::trace!(?action, "villain");
                session.recorder.record(villain(hero), action, None);
                session.witness = session.witness.try_push(action)?;
            }
            session.history = resp.action.clone();
        }
        if let Some(w) = resp.winnings {
            tracing::info!(bb = to_bb(w), "immediate result");
            session.recorder.set_pnl(seat, to_chips(w));
            session.recorder.set_pnl(villain(hero), -to_chips(w));
            session
                .recorder
                .flush(&session.witness, session.witness.head().board(), session.witness.head().pot())
                .await;
            return Ok(HandResult {
                winnings_bb: to_bb(w),
                hero,
            });
        }
        loop {
            if let Some(result) = session.act(player).await? {
                return Ok(result);
            }
        }
    }
    /// Hero decides, encodes, sends to API, parses response.
    /// Returns `Some(HandResult)` if the hand is over.
    async fn act(&mut self, player: &mut dyn parlor::Player) -> anyhow::Result<Option<HandResult>> {
        match self.witness.head().turn() {
            Turn::Terminal => anyhow::bail!("reached terminal without winnings from Slumbot"),
            Turn::Chance => anyhow::bail!("unexpected chance node in session loop"),
            Turn::Choice(p) if Turn::Choice(p) != self.hero => {
                anyhow::bail!("Slumbot's turn but no action received")
            }
            Turn::Choice(p) => {
                let game = self.witness.head();
                let start = std::time::Instant::now();
                let action = player.decide(&self.witness).await;
                let elapsed = start.elapsed().as_millis() as i32;
                let snapped = game.snap(action);
                tracing::trace!(
                    ?snapped,
                    raw = ?action,
                    incr = %encode_action(action, &game),
                    "hero",
                );
                self.recorder.record(p, snapped, Some(elapsed));
                let incr = encode_action(action, &game);
                self.witness = self.witness.try_push(snapped)?;
                self.history.push_str(&incr);
                let resp = self.client.act(&incr).await?;
                tracing::trace!(
                    action = %resp.action,
                    board = %resp.board.join(" "),
                    winnings = ?resp.winnings,
                    "slumbot",
                );
                self.refresh(&resp.board)?;
                let prior = std::mem::replace(&mut self.history, resp.action.clone());
                for action in parse_actions(&self.history, &prior, &self.witness.head())? {
                    tracing::trace!(?action, "villain");
                    self.recorder.record(villain(self.hero), action, None);
                    self.witness = self.witness.try_push(action)?;
                }
                self.refresh(&resp.board)?;
                if let Some(w) = resp.winnings {
                    tracing::info!(
                        bb = to_bb(w),
                        board = %self.board
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" "),
                        "hand result",
                    );
                    if !self.witness.head().must_stop() {
                        self.recorder.record(villain(self.hero), Action::Fold, None);
                        self.witness = self.witness.try_push(Action::Fold)?;
                    }
                    self.recorder.set_pnl(p, to_chips(w));
                    self.recorder.set_pnl(villain(self.hero), -to_chips(w));
                    self.recorder
                        .flush(&self.witness, self.witness.head().board(), self.witness.head().pot())
                        .await;
                    return Ok(Some(HandResult {
                        winnings_bb: to_bb(w),
                        hero: self.hero,
                    }));
                }
                Ok(None)
            }
        }
    }
    /// Update witness with new board cards if the board has grown.
    fn refresh(&mut self, raw: &[String]) -> anyhow::Result<()> {
        let fresh = parse_board(raw)?;
        if fresh.len() > self.board.len() {
            self.witness = Witness::try_arrange_with(
                self.hero,
                arrangement(self.hole, &fresh),
                [SLUMBOT_STACK; N],
                self.witness
                    .actions()
                    .iter()
                    .filter(|a| a.is_choice())
                    .copied()
                    .collect(),
            )?;
            self.board = fresh;
        }
        Ok(())
    }
}

/// Returns the villain's position (opposite of hero).
fn villain(hero: Turn) -> Position {
    match hero {
        Turn::Choice(0) => 1,
        _ => 0,
    }
}
