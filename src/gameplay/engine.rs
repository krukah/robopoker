pub struct Engine {
    hand: Hand,
    n_hands: u32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            hand: Hand::new(),
            n_hands: 0,
        }
    }

    pub fn gain_seat(&mut self, stack: u32, actor: Rc<dyn Player>) {
        println!("ADD  {}\n", self.hand.head.seats.len());
        let seat = Seat::new(stack, self.hand.head.seats.len(), actor);
        self.hand.head.seats.push(seat);
    }

    pub fn lose_seat(&mut self, seat_id: usize) {
        println!("DROP {}\n", seat_id);
        self.hand.head.seats.retain(|s| s.seat_id != seat_id);
    }

    pub fn start(&mut self) {
        loop {
            if self.has_exhausted_hands() {
                break;
            }
            self.start_hand();
            loop {
                if self.has_exhausted_streets() {
                    break;
                }
                self.start_street();
                loop {
                    if self.has_exhausted_turns() {
                        break;
                    }
                    self.end_turn();
                }
                self.end_street();
            }
            self.end_hand();
        }
    }

    fn start_street(&mut self) {
        self.hand.head.start_street();
        self.hand.start_street();
    }
    fn start_hand(&mut self) {
        println!("HAND  {}\n", self.n_hands);
        self.hand.head.start_hand();
        self.hand.start_hand();
    }

    fn end_turn(&mut self) {
        let seat = self.hand.head.to_act();
        let action = seat.actor.act(seat, &self.hand);
        self.hand.apply(action);
    }
    fn end_street(&mut self) {
        self.hand.head.end_street();
        self.hand.end_street();
    }
    fn end_hand(&mut self) {
        self.n_hands += 1;
        self.hand.end_hand();
    }

    fn has_exhausted_turns(&self) -> bool {
        !self.hand.head.has_more_players()
    }
    fn has_exhausted_streets(&self) -> bool {
        !self.hand.head.has_more_streets()
    }
    fn has_exhausted_hands(&self) -> bool {
        !self.hand.head.has_more_hands()
    }
}

impl Hand {
    pub fn apply(&mut self, action: Action) {
        self.head.apply(action.clone());
        self.actions.push(action.clone());
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
    }
    pub fn start_hand(&mut self) {
        self.tail = self.head.clone();
        self.deck = Deck::new();
        self.actions.clear();
        self.apply(Action::Blind(self.head.to_act().seat_id, self.sblind));
        self.apply(Action::Blind(self.head.to_act().seat_id, self.bblind));
        self.head.counter = 0;
    }
    pub fn start_street(&mut self) {
        match self.head.board.street {
            Street::Pre => {
                for hole in self.head.seats.iter_mut().map(|s| &mut s.hole) {
                    hole.cards.clear();
                    hole.cards.push(self.deck.draw().unwrap());
                    hole.cards.push(self.deck.draw().unwrap());
                }
            }
            Street::Flop => {
                let card1 = self.deck.draw().unwrap();
                let card2 = self.deck.draw().unwrap();
                let card3 = self.deck.draw().unwrap();
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("FLOP   {} {} {}", card1, card2, card3);
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("TURN   {}", card)
            }
            Street::River => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("RIVER  {}", card)
            }
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end_street(&mut self) {
        self.head.board.street = match self.head.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end_hand(&mut self) {
        println!("SHOW DOWN");
        println!("{}", self.head);
        for result in self.payouts().iter().filter(|p| p.reward > 0) {
            let seat = self
                .head
                .seats
                .iter_mut()
                .find(|s| s.seat_id == result.position)
                .unwrap();
            seat.stack += result.reward;
            println!(
                "{}",
                format!("{}  +{}", seat.seat_id, result.reward).green()
            );
        }
    }
}

impl Node {
    pub fn apply(&mut self, action: Action) {
        let seat = self.seats.get_mut(self.pointer).unwrap();
        // bets entail pot and stack change
        match action {
            Action::Call(_, bet)
            | Action::Blind(_, bet)
            | Action::Raise(_, bet)
            | Action::Shove(_, bet) => {
                self.pot += bet;
                seat.stake += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        // folds and all-ins entail status change
        match action {
            Action::Fold(..) => seat.status = BetStatus::Folded,
            Action::Shove(..) => seat.status = BetStatus::Shoved,
            _ => (),
        }
        // player actions entail rotation
        match action {
            Action::Draw(card) => self.board.push(card.clone()),
            _ => self.rotate(),
        }
    }
    pub fn start_hand(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.status = BetStatus::Playing;
            seat.stake = 0;
        }
        self.pot = 0;
        self.counter = 0;
        self.board.cards.clear();
        self.board.street = Street::Pre;
        self.dealer = self.after(self.dealer);
        self.pointer = self.dealer;
        self.rotate();
    }
    pub fn start_street(&mut self) {
        self.counter = 0;
        self.pointer = match self.board.street {
            Street::Pre => self.after(self.after(self.dealer)),
            _ => self.dealer,
        };
        self.rotate();
    }
    pub fn end_street(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.stake = 0;
        }
    }
    fn rotate(&mut self) {
        'left: loop {
            if !self.has_more_players() {
                return;
            }
            self.counter += 1;
            self.pointer = self.after(self.pointer);
            match self.to_act().status {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'left,
            }
        }
    }
}

use crate::cards::{board::Street, deck::Deck};

use super::{
    action::Action,
    hand::Hand,
    node::Node,
    player::Player,
    seat::{BetStatus, Seat},
};
use colored::Colorize;
use std::rc::Rc;
