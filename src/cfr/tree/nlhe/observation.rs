use crate::cards::card::Card;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Observation([Card; 5]);
