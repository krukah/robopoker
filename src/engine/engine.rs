pub struct Engine {
    hand: Game,
    players: Vec<Box<dyn Player>>,
    n_hands: u32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            hand: Game::new(vec![
                Seat::new(1_000, 0),
                Seat::new(1_000, 1),
                Seat::new(1_000, 2),
                Seat::new(1_000, 3),
            ]),
            players: Vec::with_capacity(10),
            n_hands: 0,
        }
    }

    pub fn add(&mut self, seat: Seat) {
        println!("ADD  {}\n", seat);
        self.players.push(Box::new(RoboPlayer::new(&seat)));
        self.hand.head.seats.push(seat);
    }

    pub fn remove(&mut self, id: usize) {
        let seat = self.hand.head.seats.iter().find(|s| s.id == id).unwrap();
        println!("REMOVE  {}\n", seat);
        self.players.retain(|p| p.id() != id);
        self.hand.head.seats.retain(|s| s.id != id);
    }

    pub fn play(&mut self) {
        let game = &mut self.hand;
        'hands: loop {
            game.begin_hand();
            'streets: loop {
                game.begin_street();
                'players: loop {
                    if !game.head.has_more_players() {
                        break 'players;
                    }
                    game.to_next_player();
                    continue 'players;
                }
                if !game.head.has_more_streets() {
                    break 'streets;
                }
                game.to_next_street();
                continue 'streets;
            }
            if !game.head.has_more_hands() {
                break 'hands;
            }
            game.to_next_hand();
            continue 'hands;
        }
    }
}

use super::{action::Player, game::Game, player::RoboPlayer, seat::Seat};
