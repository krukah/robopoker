pub struct Engine {
    deck: Deck,
    game: Game,
    players: Vec<Player>,
}

impl Engine {
    pub fn new(players: Vec<Player>) -> Engine {
        // generate a default seat for each player
        let seats = players.iter().map(|p| Seat::new(100)).collect();
        let game = Game::new(seats);
        let deck = Deck::new();
        Engine {
            deck,
            game,
            players,
        }
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
        self.apply(Action::Call(self.game.bblind));
        self.next_seat();
        self.apply(Action::Call(self.game.sblind));
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
        let player = self.players.iter().find(|p| eq(p.seat, seat)).unwrap();
        let action = player.act();
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
            }
            Street::Flop | Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
            }
            Street::River => (),
        }
    }

    fn end_hand(&mut self) {
        todo!()
    }

    fn next_seat(&mut self) {
        loop {
            let node = &mut self.game.head;
            node.counter += 1;
            node.pointer = node.after(node.pointer);
            let seat = node.get_seat();
            match seat.status {
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
            .for_each(|s| s.sunk = 0);
    }

    fn next_hand(&mut self) {
        let node = &mut self.game.head;
        node.pot = 0;
        node.counter = 0;
        node.dealer = node.after(node.dealer);
        node.pointer = node.after(node.dealer);
        node.board.cards.clear();
        node.board.street = Street::Pre;
        node.seats
            .iter_mut()
            .for_each(|s| s.status = BetStatus::Playing);
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
            Action::Call(bet) | Action::Open(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                node.pot += bet;
                seat.sunk += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        self.game.actions.push(action);
    }
}
use super::{
    action::{Action, Actor},
    game::Game,
    player::Player,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
use std::ptr::eq;
