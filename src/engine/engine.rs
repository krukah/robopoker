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
        println!("ADD  {}\n", seat);
        self.players.push(RoboPlayer::new(&seat));
        self.game.head.seats.push(seat);
    }

    pub fn remove(&mut self, id: usize) {
        let seat = self.game.head.seats.iter().find(|s| s.id == id).unwrap();
        println!("REMOVE  {}\n", seat);
        self.players.retain(|p| p.id != id);
        self.game.head.seats.retain(|s| s.id != id);
    }

    pub fn play(&mut self) {
        'hand: loop {
            self.deal_hole_cards();
            self.post_blinds();
            'street: loop {
                'seat: loop {
                    match self.game.head.next() {
                        Some(_) => self.take_action(),
                        None => break 'seat,
                    }
                }
                if self.game.head.does_end_hand() {
                    self.show_down();
                    self.game.head.next_hand();
                    continue 'hand;
                }
                if self.game.head.does_end_street() {
                    self.deal_board_cards();
                    self.game.head.next_street();
                    continue 'street;
                }
            }
        }
    }

    fn post_blinds(&mut self) {
        // todo!() handle all in case. check if stack > blind ? Post : Shove
        self.game.head.next();
        self.apply(Action::Blind(self.game.sblind));
        self.game.head.next();
        self.apply(Action::Blind(self.game.bblind));
        self.game.head.counter = 0;
    }

    fn deal_hole_cards(&mut self) {
        for player in self.players.iter_mut() {
            let card1 = self.deck.draw().unwrap();
            let card2 = self.deck.draw().unwrap();
            player.hole.cards.clear();
            player.hole.cards.push(card1);
            player.hole.cards.push(card2);
        }
    }

    fn take_action(&mut self) {
        let seat = self.game.head.seat();
        let action = self
            .players
            .iter()
            .find(|p| p.id == seat.id)
            .unwrap()
            .act(&self.game);
        self.game.head.apply(action.clone());
        self.game.actions.push(action.clone());
    }

    fn deal_board_cards(&mut self) {
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

    fn show_down(&mut self) {
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

    fn apply(&mut self, action: Action) {
        self.game.head.apply(action.clone());
        self.game.actions.push(action.clone());
    }
}

use super::{
    action::{Action, Player},
    game::Game,
    player::RoboPlayer,
    seat::Seat,
};
use crate::cards::{board::Street, deck::Deck};
