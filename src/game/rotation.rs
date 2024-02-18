pub struct Rotation {
    index: usize,
    dealer: usize,
    players: Vec<RefCell<Player>>,
}
impl Rotation {
    fn new() -> Rotation {
        todo!()
    }
    fn advance(&mut self) {
        self.dealer = self.dealer + 1;
        self.index = self.dealer + 1;
        self.dealer %= self.players.len();
        self.index %= self.players.len();
    }
}
impl Iterator for Rotation {
    type Item = RefCell<Player>;
    fn next(&mut self) -> Option<RefCell<Player>> {
        loop {
            let player = self.players[self.index];
            self.index += 1;
            self.index %= self.players.len();
            if player.borrow().status == PlayerStatus::Active {
                return Some(player);
            }
        }
    }
}
