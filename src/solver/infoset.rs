pub struct InfoSet<'a> {
    pub game: &'a Game,
    pub hole: &'a Hole,
}

use crate::{cards::hole::Hole, engine::game::Game};
