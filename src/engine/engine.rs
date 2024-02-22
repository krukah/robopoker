pub struct Engine<'a> {
    deck: &'a mut Deck,
    game: &'a mut Game,
    players: Vec<Player>,
}

impl Actor for Seat {
    fn act(&self) -> Action {
        todo!()
    }
}

impl<'a> Engine<'a> {
    pub fn new(players: Vec<Player>) -> Engine<'static> {
        let deck = &mut Deck::new();
        let game = &mut Game::new(todo!());
        Engine {
            deck,
            game,
            players,
        }
    }

    pub fn run(&mut self) {
        self.init();
        self.play();
    }

    pub fn add(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn remove(&mut self, player: Player) {
        self.players.retain(|p| !eq(p, &player));
    }

    fn init(&mut self) {}

    fn play(&mut self) {
        'hand: loop {
            self.post_blinds();
            'street: loop {
                'player: while !self.game.head.is_end_of_street() {
                    let action = self.next_seat().act();
                    self.apply(action);
                    continue 'player;
                }
                if self.game.head.is_end_of_hand() {
                    self.stop_hand();
                    self.next_hand();
                    continue 'hand;
                }
                if self.game.head.is_end_of_street() {
                    self.deal_street();
                    self.next_street();
                    continue 'street;
                }
            }
        }
    }

    // dealer behavior
    fn post_blinds(&mut self) {
        self.next_seat();
        self.apply(Action::Call(self.game.bblind));
        self.next_seat();
        self.apply(Action::Call(self.game.sblind));
        self.game.head.counter = 0;
    }

    fn deal_street(&mut self) {
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

    fn stop_hand(&mut self) {
        todo!()
    }

    // node mutations
    fn next_seat(&mut self) -> &Seat {
        let node = &mut self.game.head;
        loop {
            node.counter += 1;
            node.pointer = node.after(node.pointer);
            let seat = node.seats.get(node.pointer).unwrap();
            match seat.status {
                BetStatus::Folded | BetStatus::Shoved => continue,
                BetStatus::Playing => return seat,
            }
        }
    }

    fn next_street(&mut self) {
        let node = &mut self.game.head;
        node.pointer = node.dealer;
        node.counter = 0;
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
        node.dealer = node.after(node.dealer);
        node.pointer = node.dealer;
        node.counter = 0;
        node.board.cards.clear();
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
            _ => (),
        }
        match action {
            Action::Draw(card) => node.board.push(card.clone()),
            Action::Call(bet) | Action::Open(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                self.bet(bet)
            }
            _ => (),
        }
        self.game.actions.push(action);
    }

    fn bet(&mut self, bet: u32) {
        let node = &mut self.game.head;
        let seat = node.seats.get_mut(node.pointer).unwrap();
        node.pot += bet;
        seat.sunk += bet;
        seat.stack -= bet;
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
