#[derive(Debug, PartialEq)]
pub enum HandStatus {
    Active,
    Folded,
    AllIn,
    StandingUp,
}

pub struct Player {
    pub stack: u32,
    pub wager: u32,
    pub cards: Hole,
    pub status: HandStatus,
}
impl Player {
    fn new() -> Player {
        todo!()
    }
    fn decide(&mut self) -> Action {
        todo!()
    }
}
