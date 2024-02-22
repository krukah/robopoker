struct HandCards {
    cards: [Card; 7],
}
impl HandCards {
    pub fn to_u64(&self) -> u64 {
        self.cards
            .iter()
            .map(|c| c.to_int())
            .fold(0, |bits, i| bits | 1 << i)
    }
}
use crate::cards::card::Card;
