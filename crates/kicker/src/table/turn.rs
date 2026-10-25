/// Whose turn it is to act: a player decision node, a chance node (card deal),
/// or terminal (hand complete, compute payoffs).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Turn {
    Terminal,
    Chance,
    Choice(usize),
}

impl Turn {
    /// Extracts the player index. Panics if not a Choice.
    pub fn position(&self) -> usize {
        match self {
            Self::Choice(c) => *c,
            _ => panic!("don't ask"),
        }
    }
    /// True if this is a player decision node.
    pub fn is_choice(&self) -> bool {
        matches!(self, Self::Choice(_))
    }
    /// True if this is a card deal node.
    pub fn is_chance(&self) -> bool {
        matches!(self, Self::Chance)
    }
    /// True if the hand is complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal)
    }
    /// 1-indexed player number for display.
    pub fn display(&self) -> usize {
        match self {
            Self::Choice(c) => *c + 1,
            _ => panic!("don't ask"),
        }
    }
    /// Display label (e.g., "P1", "P2").
    pub fn label(&self) -> String {
        format!("P{}", self.display())
    }
}

impl pokerkit::Arbitrary for Turn {
    fn random() -> Self {
        Self::Choice(rand::random_range(0..pokerkit::N))
    }
}

impl From<usize> for Turn {
    fn from(player: usize) -> Self {
        Self::Choice(player)
    }
}

impl TryFrom<&str> for Turn {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "XX" => Ok(Self::Terminal),
            "??" => Ok(Self::Chance),
            turn => turn[1..]
                .parse::<usize>()
                .map(Self::Choice)
                .map_err(|_| anyhow::anyhow!("invalid player turn")),
        }
    }
}

impl std::fmt::Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(c) => write!(f, "P{c}"),
            Self::Terminal => write!(f, "-"),
            Self::Chance => write!(f, "?"),
        }
    }
}

impl serde::Serialize for Turn {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for Turn {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(d)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}
