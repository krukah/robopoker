pub enum HandRank {
    HighCard(Rank),
    OnePair(Rank),
    TwoPair(Rank),
    ThreeOfAKind(Rank),
    Straight(Rank),
    Flush(Rank),
    FullHouse(Rank),
    FourOfAKind(Rank),
    StraightFlush(Rank),
}

use crate::cards::rank::Rank;
