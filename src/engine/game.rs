#[derive(Debug, Clone)]
pub struct Game {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub payoffs: Vec<Payoff>,
    pub players: Vec<RoboPlayer>,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new(seats: Vec<Seat>) -> Self {
        let players: Vec<RoboPlayer> = seats.iter().map(|s| RoboPlayer::new(s)).collect();
        let node = Node::new(seats);
        Game {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            payoffs: Vec::new(),
            deck: Deck::new(),
            tail: node.clone(),
            head: node,
            players,
        }
    }

    pub fn begin_hand(&mut self) {
        self.head.begin_hand();
        self.deal_players();
        self.post_blinds();
    }
    pub fn begin_street(&mut self) {
        self.head.begin_street();
    }
    pub fn apply(&mut self, action: Action) {
        self.head.apply(action.clone());
        self.actions.push(action.clone());
    }

    pub fn to_next_hand(&mut self) {
        self.settle();
        self.prune();
        self.reset_hand();
        println!("{}\n---\n", self.head);
    }
    pub fn to_next_street(&mut self) {
        self.deal_board();
        self.reset_street();
    }
    pub fn to_next_player(&mut self) {
        let seat = self.head.next();
        let player = self.players.iter().find(|p| p.id == seat.id).unwrap();
        let action = player.act(&self);
        self.apply(action);
    }

    fn reset_hand(&mut self) {
        for seat in &mut self.head.seats {
            seat.status = BetStatus::Playing;
            seat.stuck = 0;
        }
        self.actions.clear();
        self.deck = Deck::new();
    }
    fn reset_street(&mut self) {
        for seat in &mut self.head.seats {
            seat.stuck = 0;
        }
    }

    fn post_blinds(&mut self) {
        // todo!() handle all in case. check if stack > blind ? Post : Shove
        self.apply(Action::Blind(self.head.next().id, self.sblind));
        self.apply(Action::Blind(self.head.next().id, self.bblind));
        self.head.counter = 0;
    }
    fn deal_players(&mut self) {
        // engine
        for player in self.players.iter_mut() {
            let card1 = self.deck.draw().unwrap();
            let card2 = self.deck.draw().unwrap();
            player.hole.cards.clear();
            player.hole.cards.push(card1);
            player.hole.cards.push(card2);
        }
    }
    fn deal_board(&mut self) {
        match self.head.board.street {
            Street::Pre => {
                let card1 = self.deck.draw().unwrap();
                let card2 = self.deck.draw().unwrap();
                let card3 = self.deck.draw().unwrap();
                self.head.board.street = Street::Flop;
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("FLOP   {} {} {}", card1, card2, card3);
            }
            Street::Flop => {
                let card = self.deck.draw().unwrap();
                self.head.board.street = Street::Turn;
                self.apply(Action::Draw(card));
                println!("TURN   {}", card)
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.head.board.street = Street::River;
                self.apply(Action::Draw(card));
                println!("RIVER  {}", card)
            }
            _ => (),
        }
    }

    fn settle(&mut self) {
        for player in self.players.iter() {
            let risked = self.risked(player);
            let seat = self
                .head
                .seats
                .iter_mut()
                .find(|s| s.id == player.id)
                .unwrap();
            seat.stack += risked;
        }
    }
    fn prune(&mut self) {
        self.head.seats.retain(|s| s.stack > 0);
    }

    fn risked(&self, player: &RoboPlayer) -> u32 {
        self.actions
            .iter()
            .filter(|a| match a {
                Action::Call(id, _)
                | Action::Blind(id, _)
                | Action::Raise(id, _)
                | Action::Shove(id, _) => *id == player.id,
                _ => false,
            })
            .map(|a| match a {
                Action::Call(_, bet)
                | Action::Blind(_, bet)
                | Action::Raise(_, bet)
                | Action::Shove(_, bet) => *bet,
                _ => 0,
            })
            .sum()
    }
    fn reward(&self, player: &RoboPlayer) -> u32 {
        let max = self.risked(player);
        self.players
            .iter()
            .map(|p| self.risked(p))
            .map(|r| std::cmp::min(r, max))
            .sum()
    }

    fn evaluate(&self, hole: &Hole) -> u32 {
        0
    }
}

use super::{
    action::{Action, Player},
    node::Node,
    payoff::Payoff,
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck, hole::Hole};

pub struct ShowdownHand<'a> {
    pub seat: &'a mut Seat,
    pub hole: &'a mut Hole,
}
pub struct ShowdownResult<'a> {
    pub seat: &'a mut Seat,
    pub eval: u32,
}
