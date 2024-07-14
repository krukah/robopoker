use crate::cards::card::Card;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum Observation {
    Pre([Card; 2]),
    Flo([Card; 5]),
    Tur([Card; 6]),
    Riv([Card; 7]),
}
