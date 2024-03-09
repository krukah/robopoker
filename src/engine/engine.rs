pub struct Engine {
    game: Game,
    players: Vec<RoboPlayer>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            game: Game::new(vec![
                Seat::new(1_000, 0),
                Seat::new(1_000, 1),
                Seat::new(1_000, 2),
                Seat::new(1_000, 3),
            ]),
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
            self.game.begin_hand();
            'street: loop {
                self.game.begin_street();
                'player: loop {
                    if self.game.head.has_more_players() {
                        self.game.to_next_player();
                        continue 'player;
                    }
                    break 'player;
                }
                if self.game.head.has_more_streets() {
                    self.game.to_next_street();
                    continue 'street;
                }
                break 'street;
            }
            self.game.to_next_hand();
            continue 'hand;
        }
    }
}

use super::{game::Game, player::RoboPlayer, seat::Seat};
