// data structures
#[derive(Debug, Clone, Copy)]
pub enum Suit {
    Club = 0,
    Diamond = 1,
    Heart = 2,
    Spade = 3,
}
impl From<u8> for Suit {
    fn from(n: u8) -> Suit {
        match n {
            0 => Suit::Club,
            1 => Suit::Diamond,
            2 => Suit::Heart,
            3 => Suit::Spade,
            _ => panic!("Invalid suit"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Rank {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}
impl From<u8> for Rank {
    fn from(n: u8) -> Rank {
        match n {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => panic!("Invalid rank"),
        }
    }
}

pub enum Street {
    Pre,
    Flop,
    Turn,
    River,
}

pub struct Card {
    rank: Rank,
    suit: Suit,
}
impl Card {
    pub fn to_int(&self) -> u8 {
        (self.rank as u8) * 4 + (self.suit as u8)
    }
}
impl From<u8> for Card {
    fn from(n: u8) -> Self {
        Card {
            rank: Rank::from(n / 4),
            suit: Suit::from(n % 4),
        }
    }
}

pub struct Hole {
    cards: Vec<Card>, // presize
}

pub struct Board {
    pub cards: Vec<Card>, // presize
    pub street: Street,
}
impl Board {
    pub fn accept(&mut self, card: Card) {
        self.cards.push(card);
    }
}

pub struct Deck {
    cards: Vec<Card>, // presize
}
impl Deck {
    pub fn new() -> Deck {
        let mut cards = Vec::new();
        for i in 0..52 {
            cards.push(Card::from(i));
        }
        Deck { cards }
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn shuffle(&mut self) {}
}
