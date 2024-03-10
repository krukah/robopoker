#[derive(Debug, Clone)]
pub struct Hand {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub outcomes: Vec<HandResult>,
    pub actions: Vec<Action>,
}
impl Hand {
    pub fn new(seats: Vec<Seat>) -> Self {
        let node = Node::new(seats);
        Hand {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            outcomes: Vec::new(),
            deck: Deck::new(),
            tail: node.clone(),
            head: node,
        }
    }

    pub fn start_hand(&mut self) {
        self.head.start_hand();
        // deal players
        for seat in self.head.seats.iter_mut() {
            let card1 = self.deck.draw().unwrap();
            let card2 = self.deck.draw().unwrap();
            seat.hole.cards.clear();
            seat.hole.cards.push(card1);
            seat.hole.cards.push(card2);
        }
        self.post_blinds();
    }
    pub fn begin_street(&mut self) {
        self.head.start_street();
    }
    pub fn apply(&mut self, action: Action) {
        self.head.apply(action.clone());
        self.actions.push(action.clone());
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
    }

    pub fn to_next_hand(&mut self) {
        self.allocate();
        for seat in &mut self.head.seats {
            seat.status = BetStatus::Playing;
            seat.stuck = 0;
        }
        self.tail = self.head.clone();
        self.deck = Deck::new();
        self.actions.clear();
        self.outcomes.clear();
    }
    pub fn to_next_street(&mut self) {
        for seat in self.head.seats.iter_mut() {
            seat.stuck = 0;
        }
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
            Street::River => {
                println!("SHOWDOWN")
            }
        }
    }
    pub fn to_next_player(&mut self) {
        let seat = self.head.next();
        let player = self.players.iter().find(|p| p.id() == seat.id).unwrap();
        let action = player.act(&self);
        self.apply(action);
    }

    fn post_blinds(&mut self) {
        // todo!() handle all in case. check if stack > blind ? Post : Shove
        self.apply(Action::Blind(self.head.next().id, self.sblind));
        self.apply(Action::Blind(self.head.next().id, self.bblind));
        self.head.counter = 0;
    }

    fn allocate(&mut self) {
        let results = self.showdown();
        for result in results {
            let seat = self
                .head
                .seats
                .iter_mut()
                .find(|s| s.id == result.id)
                .unwrap();
            seat.stack += result.reward;
        }
        println!("{}\n---\n", self.head);
    }

    fn showdown(&self) -> Vec<HandResult> {
        let mut showdown = Showdown {
            next_stake: u32::MIN,
            prev_stake: u32::MIN,
            next_score: u32::MAX,
            results: self.results(),
        };
        '_winners: loop {
            showdown.next_score();
            '_pots: loop {
                showdown.next_stake();
                showdown.distribute();
                if showdown.is_complete() {
                    return showdown.results;
                }
            }
        }
    }

    fn results(&self) -> Vec<HandResult> {
        let mut results = self
            .head
            .seats
            .iter()
            .map(|p| HandResult {
                id: p.id,
                score: self.score(p.id),
                status: self.status(p.id),
                staked: self.staked(p.id),
                reward: 0,
            })
            .collect::<Vec<HandResult>>();
        results.sort_by(|a, b| {
            let x = self.priority(a.id);
            let y = self.priority(b.id);
            x.cmp(&y)
        });
        results
    }
    fn status(&self, id: usize) -> BetStatus {
        self.head.seats.iter().find(|s| s.id == id).unwrap().status
    }
    fn staked(&self, id: usize) -> u32 {
        self.actions
            .iter()
            .filter(|a| match a {
                Action::Call(id_, _)
                | Action::Blind(id_, _)
                | Action::Raise(id_, _)
                | Action::Shove(id_, _) => *id_ == id,
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
        // O(n) in actions
    }
    fn score(&self, _id: usize) -> u32 {
        rand::thread_rng().gen::<u32>() % 32
    }
    fn priority(&self, id: usize) -> u32 {
        (id.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
}

use super::{
    action::Action,
    node::Node,
    payoff::HandResult,
    seat::{BetStatus, Seat},
    showdown::Showdown,
};
use crate::cards::{board::Street, deck::Deck};
use rand::Rng;
