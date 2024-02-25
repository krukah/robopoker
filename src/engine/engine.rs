pub struct Engine {
    deck: Deck,
    game: Game,
    players: Vec<RoboPlayer>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            deck: Deck::new(),
            game: Game::new(),
            players: Vec::with_capacity(10),
        }
    }

    pub fn add(&mut self, seat: Seat) {
        println!("ADD  {:?}", seat.id);
        self.players.push(RoboPlayer::new(&seat));
        self.game.head.seats.push(seat);
    }

    pub fn remove(&mut self, id: usize) {
        println!("REMOVE  {:?}", id);
        self.players.retain(|p| p.id != id);
        self.game.head.seats.retain(|s| s.id != id);
    }

    pub fn play(&mut self) {
        'hand: loop {
            self.deal_players();
            self.post_blinds();
            'street: loop {
                'seat: while !self.game.head.is_end_of_street() {
                    self.take_action();
                    self.next_seat();
                    continue 'seat;
                }
                if self.game.head.is_end_of_hand() {
                    self.end_hand();
                    self.next_hand();
                    continue 'hand;
                }
                if self.game.head.is_end_of_street() {
                    self.deal_board();
                    self.next_street();
                    continue 'street;
                }
            }
        }
    }

    fn post_blinds(&mut self) {
        self.apply(Action::Post(self.game.sblind));
        self.next_seat();
        self.apply(Action::Post(self.game.bblind));
        self.next_seat();
        self.game.head.counter = 0;
    }

    fn deal_players(&mut self) {
        for player in self.players.iter_mut() {
            let card1 = self.deck.draw().unwrap();
            let card2 = self.deck.draw().unwrap();
            player.hole.cards.clear();
            player.hole.cards.push(card1);
            player.hole.cards.push(card2);
        }
    }

    fn take_action(&mut self) {
        let seat = self.game.head.get_seat();
        let action = self
            .players
            .iter()
            .find(|p| p.id == seat.id)
            .unwrap()
            .act(&self.game);
        self.apply(action);
    }

    fn deal_board(&mut self) {
        match self.game.head.board.street {
            Street::Pre => {
                let card1 = self.deck.draw().unwrap();
                let card2 = self.deck.draw().unwrap();
                let card3 = self.deck.draw().unwrap();
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("DEAL  {} {} {}", card1, card2, card3);
            }
            Street::Flop | Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("DEAL  {}", card)
            }
            Street::River => (),
        }
    }

    fn end_hand(&mut self) {
        println!("SHOWDOWN {}", &self.game.head);
        let positions: Vec<usize> = self
            .game
            .head
            .seats
            .iter()
            .filter(|s| s.stack == 0)
            .map(|s| s.id)
            .collect();
        positions.iter().for_each(|p| self.remove(*p));
        self.deck = Deck::new();
    }

    fn next_seat(&mut self) {
        loop {
            self.increment();
            let seat = &self.game.head.get_seat();
            let status = seat.status;
            match status {
                BetStatus::Folded | BetStatus::Shoved => continue,
                BetStatus::Playing => return,
            }
        }
    }

    fn next_street(&mut self) {
        let node = &mut self.game.head;
        node.counter = 0;
        node.pointer = node.after(node.dealer);
        node.board.street = match node.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Pre,
        };
        node.seats
            .iter_mut()
            .filter(|s| s.status != BetStatus::Shoved)
            .for_each(|s| s.stuck = 0);
        println!("  {:?}", node.board.street);
    }

    fn next_hand(&mut self) {
        println!("NEXT HAND\n");
        let node = &mut self.game.head;
        node.pot = 0;
        node.counter = 0;
        node.dealer = node.after(node.dealer);
        node.pointer = node.after(node.dealer);
        node.board.cards.clear();
        node.board.street = Street::Pre;
        node.seats.iter_mut().for_each(|s| {
            s.status = BetStatus::Playing;
            s.stuck = 0;
        });
        // sleep(Duration::from_secs(2));
    }

    fn apply(&mut self, action: Action) {
        let node = &mut self.game.head;
        let seat = node.seats.get_mut(node.pointer).unwrap();
        match action {
            Action::Fold => seat.status = BetStatus::Folded,
            Action::Shove(_) => seat.status = BetStatus::Shoved,
            Action::Draw(card) => node.board.push(card.clone()),
            _ => (),
        }
        match action {
            Action::Post(bet) | Action::Call(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                node.pot += bet;
                seat.stuck += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        match action {
            Action::Draw(_) => (),
            _ => println!("  {}  {:?}", seat.id, action),
        }
        self.game.actions.push(action);
    }

    fn increment(&mut self) {
        let node = &mut self.game.head;
        node.counter += 1;
        node.pointer = node.after(node.pointer);
    }
}
use std::{thread::sleep, time::Duration};

use super::{
    action::{Action, Player},
    game::Game,
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
