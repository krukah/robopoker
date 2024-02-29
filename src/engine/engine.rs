pub struct Engine {
    deck: Deck,
    game: Game,
    players: Vec<RoboPlayer>,
}

struct Payoff {
    position: usize,
    reward: u32,
    risked: u32,
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
                    if self.game.head.does_end_street() {
                        break 'seat;
                    }
                    self.game.head.advance();
                    if self.game.head.does_end_street() {
                        break 'seat;
                    }
                    self.take_action();
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
        self.game.head.advance();
        self.apply(Action::Blind(self.game.sblind));
        self.game.head.advance();
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
        let actor = self.actor();
        let action = actor.act(&self.game);
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
    fn payoff<'seat>(&self, node: &'seat Node) -> Vec<Payoff> {
        let mut payoffs: Vec<Payoff> = Vec::new();
        let mut winners: Vec<&Seat> = Vec::new();
        for seat in &node.seats {
            if seat.status == BetStatus::Playing || seat.status == BetStatus::Shoved {
                winners.clear();
                winners.push(seat);
            }
        }
        let share = node.pot / winners.len() as u32;
        for winner in winners {
            payoffs.push(Payoff {
                position: winner.id,
                reward: share,
                risked: winner.stuck,
            });
        }
        payoffs
    }
    fn show_down(&mut self) {
        println!("SHOWDOWN {}", &self.game.head);
        // select a random seat to give the pot to
        let payoffs = self.payoff(&self.game.head);
        for payoff in payoffs {
            println!("  PAYOFF  {}  {}", payoff.position, payoff.risked);
            let seat = self
                .game
                .head
                .seats
                .iter_mut()
                .find(|po| po.id == payoff.position)
                .unwrap();
            seat.stack += payoff.reward;
        }
        let removing: Vec<usize> = self
            .game
            .head
            .seats
            .iter()
            .filter(|s| s.stack == 0)
            .map(|s| s.id)
            .collect();
        removing.iter().for_each(|p| self.remove(*p));
        self.deck = Deck::new();
    }

    fn apply(&mut self, action: Action) {
        self.game.head.apply(action.clone());
        self.game.actions.push(action.clone());
    }

    fn actor(&self) -> &RoboPlayer {
        let seat = self.game.head.seat();
        self.players.iter().find(|p| p.id == seat.id).unwrap()
    }
}

use super::{
    action::{Action, Player},
    game::Game,
    node::Node,
    player::RoboPlayer,
    seat::{BetStatus, Seat},
};
use crate::cards::{board::Street, deck::Deck};
