use croupier::*;
use fulcrum::*;
use kicker::*;

/// Slumbot uses 50/100 blinds with 20000 stacks (200 BB deep).
const BBLIND: i64 = 100;
const SBLIND: i64 = 50;
const STACKS: i64 = 20000;
/// Our chip stack when playing Slumbot. Hardcoded to 400 (200 BB at
/// `B_BLIND=2`) to match Slumbot's 200-BB game. The blueprint is trained at
/// `STACK=200` (100 BB); this override only affects the chip math during a
/// Slumbot session so our parser correctly interprets Slumbot's BB-relative
/// bets and `SCALE` lines our `B_BLIND` up with theirs (`B_BLIND * SCALE ==
/// BBLIND`). Strategy lookup degrades gracefully via the pot-relative bet
/// abstraction.
pub const SLUMBOT_STACK: Chips = 400;
/// Integer multiplier from our chip scale to Slumbot's.
const SCALE: i64 = STACKS / (SLUMBOT_STACK as i64);

pub fn parse_card(s: &str) -> anyhow::Result<Card> {
    Card::try_from(s).map_err(|e| anyhow::anyhow!("bad card '{s}': {e}"))
}

pub fn parse_hole(cards: &[String]) -> anyhow::Result<(Card, Card)> {
    anyhow::ensure!(cards.len() == 2, "expected 2 hole cards, got {}", cards.len());
    Ok((parse_card(&cards[0])?, parse_card(&cards[1])?))
}

pub fn parse_board(cards: &[String]) -> anyhow::Result<Vec<Card>> {
    cards.iter().map(|s| parse_card(s)).collect()
}

pub fn arrangement(hole: (Card, Card), board: &[Card]) -> Arrangement {
    [hole.0, hole.1]
        .into_iter()
        .chain(board.iter().copied())
        .collect::<Vec<Card>>()
        .into()
}

/// Minimal parser state for converting slumbot action strings.
/// Only used inbound (slumbot -> Action). Encoding uses Game directly.
struct ParseState {
    pot: i64,
    stakes: [i64; 2],
    actor: usize,
}

impl ParseState {
    fn new() -> Self {
        Self {
            pot: SBLIND + BBLIND,
            stakes: [SBLIND, BBLIND],
            actor: 0,
        }
    }

    fn to_call(&self) -> i64 {
        (self.stakes[1 - self.actor] - self.stakes[self.actor]).max(0)
    }

    fn check(&mut self) {
        self.actor = 1 - self.actor;
    }

    fn call(&mut self) {
        self.pot += self.to_call();
        self.stakes[self.actor] = self.stakes[1 - self.actor];
        self.actor = 1 - self.actor;
    }

    fn bet(&mut self, total: i64) {
        self.pot += total - self.stakes[self.actor];
        self.stakes[self.actor] = total;
        self.actor = 1 - self.actor;
    }

    fn street(&mut self) {
        self.stakes = [0, 0];
        self.actor = 1;
    }

    fn fraction(&self, total: i64) -> f64 {
        let above = total - self.stakes[self.actor] - self.to_call();
        let denom = self.pot + self.to_call();
        if denom <= 0 || above <= 0 { 0.0 } else { above as f64 / denom as f64 }
    }
    /// Advance through a slumbot action string prefix.
    fn advance(&mut self, actions: &str) {
        let mut rest = actions;
        while !rest.is_empty() {
            match rest.as_bytes()[0] {
                b'b' => {
                    let n = rest[1..].chars().take_while(char::is_ascii_digit).count();
                    if let Ok(total) = rest[1..=n].parse::<i64>() {
                        self.bet(total);
                    }
                    rest = &rest[1 + n..];
                }
                b'/' => {
                    self.street();
                    rest = &rest[1..];
                }
                b'k' => {
                    self.check();
                    rest = &rest[1..];
                }
                b'f' => {
                    rest = &rest[1..];
                }
                b'c' => {
                    self.call();
                    rest = &rest[1..];
                }
                _ => {
                    rest = &rest[1..];
                }
            }
        }
    }
    /// Parse a single Slumbot action token.
    fn parse_one(&mut self, token: &str, game: &Game) -> anyhow::Result<(Action, usize)> {
        anyhow::ensure!(!token.is_empty(), "empty action token");
        match token.as_bytes()[0] {
            b'k' => {
                self.check();
                Ok((Action::Check, 1))
            }
            b'f' => Ok((Action::Fold, 1)),
            b'c' => {
                self.call();
                Ok((Action::Call(game.to_call()), 1))
            }
            b'b' => {
                let n = token[1..].chars().take_while(char::is_ascii_digit).count();
                anyhow::ensure!(n > 0, "b without amount");
                let total = token[1..=n].parse::<i64>()?;
                let fraction = self.fraction(total);
                self.bet(total);
                Ok((fraction_to_action(fraction, game), 1 + n))
            }
            ch => Err(anyhow::anyhow!("unknown action char '{}'", ch as char)),
        }
    }
}

/// Parse new Slumbot actions (beyond `old` prefix) into our Action sequence.
pub fn parse_actions(raw: &str, old: &str, game: &Game) -> anyhow::Result<Vec<Action>> {
    let suffix = raw.strip_prefix(old).unwrap_or(raw);
    if suffix.is_empty() {
        return Ok(Vec::new());
    }
    let mut parse = ParseState::new();
    parse.advance(old);
    let mut actions = Vec::new();
    let mut state = *game;
    let mut rest = suffix;
    while !rest.is_empty() {
        if rest.starts_with('/') {
            parse.street();
            if state.turn() == Turn::Chance {
                state = state.apply(state.reveal());
            }
            rest = &rest[1..];
            continue;
        }
        let (action, consumed) = parse.parse_one(rest, &state)?;
        actions.push(state.snap(action));
        state = state.apply(*actions.last().unwrap());
        rest = &rest[consumed..];
    }
    Ok(actions.into_iter().filter(croupier::Action::is_choice).collect())
}

/// Convert a pot-fraction raise into an Action in our chip scale.
fn fraction_to_action(fraction: f64, game: &Game) -> Action {
    let total = game.to_call() as i64 + (fraction * (game.pot() + game.to_call()) as f64).round() as i64;
    if total >= game.actor().stack() as i64 {
        Action::Shove(game.to_shove())
    } else if total < game.to_raise() as i64 {
        Action::Call(game.to_call())
    } else {
        Action::Raise(total as Chips)
    }
}

/// Encode our Action as a Slumbot increment string.
/// Derives all slumbot-scale values from Game state via SCALE multiplier.
pub(crate) fn encode_action(action: Action, game: &Game) -> String {
    match action {
        Action::Check => "k".into(),
        Action::Fold => "f".into(),
        Action::Call(_) => "c".into(),
        Action::Shove(n) | Action::Raise(n) => {
            let call = game.to_call() as i64;
            let base = game.pot() as i64 + call;
            let fraction = if base > 0 { (n as i64 - call) as f64 / base as f64 } else { 1.0 };
            let p = game.turn().position();
            let scall = call * SCALE;
            let sbase = base * SCALE;
            let sremaining = game.seats()[p].stack() as i64 * SCALE;
            let sstake = game.seats()[p].stake() as i64 * SCALE;
            let smax = sstake + sremaining;
            let sopponent = game.stakes().iter().max().copied().unwrap_or(0) as i64 * SCALE;
            let sminraise = sopponent + scall.max(BBLIND);
            if sremaining <= scall {
                "c".into()
            } else if sminraise >= smax {
                format!("b{smax}")
            } else {
                let stotal = sstake + scall + (fraction * sbase as f64).round() as i64;
                format!("b{}", stotal.clamp(sminraise, smax))
            }
        }
        _ => unreachable!("unexpected action {:?}", action),
    }
}

/// Map Slumbot's client_pos to our Turn.
/// client_pos=0 means BB (P1); client_pos=1 means SB/BTN (P0).
pub fn pov(position: usize) -> Turn {
    match position {
        0 => Turn::Choice(1),
        1 => Turn::Choice(0),
        _ => unreachable!("invalid client_pos {}", position),
    }
}

/// Convert Slumbot winnings (their chips) to bb units.
pub fn to_bb(chips: i64) -> f64 {
    chips as f64 / BBLIND as f64
}
/// Convert Slumbot winnings (their chips) to our Chips scale.
pub fn to_chips(chips: i64) -> Chips {
    (chips / SCALE) as Chips
}
