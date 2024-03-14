pub struct InfoSet<'a> {
    pub hand: &'a Hand,
    pub hole: &'a Hole,
}

use crate::{cards::hole::Hole, gameplay::hand::Hand};
