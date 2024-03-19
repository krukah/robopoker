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

    pub fn start(&mut self) {
        while self.has_hands() {
            self.start_hand();
            while self.has_streets() {
                self.start_street();
                while self.has_turns() {
                    self.take_turn();
                }
                self.end_street();
            }
            self.end_hand();
        }
    }

    pub fn gain_seat(&mut self, stack: u32, actor: Rc<dyn Player>) {
        println!("ADD  {}", self.hand.head.seats.len());
        let seat = Seat::new(stack, self.hand.head.seats.len(), actor);
        self.hand.head.seats.push(seat);
    }

    pub fn drop_seat(&mut self, position: usize) {
        println!("DROP {}", position);
        self.hand.head.seats.retain(|s| s.position != position);
    }

    fn start_street(&mut self) {
        self.hand.start_street();
        match self.hand.head.board.street {
            Street::Pre => (),
            _ => print!("   {}", self.hand.head.board),
        }
    }
    fn start_hand(&mut self) {
        println!("\n{}\nHAND   {}", "-".repeat(21), self.n_hands);
        self.hand.start();
    }
    fn take_turn(&mut self) {
        let seat = self.hand.head.next();
        let action = seat.actor.act(seat, &self.hand);
        self.hand.apply(action);
    }
    fn end_street(&mut self) {
        self.hand.end_street();
    }
    fn end_hand(&mut self) {
        print!("{}\n   {}", "-".repeat(21), self.hand.head.board);
        self.n_hands += 1;
        self.hand.end();
    }

    fn has_turns(&self) -> bool {
        self.hand.head.has_more_players()
    }
    fn has_streets(&self) -> bool {
        self.hand.head.has_more_streets()
    }
    fn has_hands(&self) -> bool {
        self.n_hands < 5000
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
    pub fn start(&mut self) {
        self.head.start_hand();
        self.tail = self.head.clone();
        self.deck = Deck::new();
        self.actions.clear();
        self.apply(Action::Blind(self.head.next().position, self.sblind));
        self.apply(Action::Blind(self.head.next().position, self.bblind));
        self.head.counter = 0;
    }
    pub fn start_street(&mut self) {
        self.head.start_street();
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
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
            }
            Street::River => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
            }
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end_street(&mut self) {
        self.head.end_street();
        self.head.board.street = match self.head.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end(&mut self) {
        for payout in self.settle() {
            let seat = self.head.seat_mut(payout.position);
            println!("{}{}", seat, payout);
            seat.stack += payout.reward;
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
            match self.next().status {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'left,
            }
        }
    }
}

use super::{
    action::Action,
    hand::Hand,
    node::Node,
    player::Player,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
use std::rc::Rc;
