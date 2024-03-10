pub struct InfoSet<'a> {
    pub game: &'a Hand,
    pub hole: &'a Hole,
}

use crate::{cards::hole::Hole, engine::game::Hand};
