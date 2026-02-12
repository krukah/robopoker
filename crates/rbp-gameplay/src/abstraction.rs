use rbp_cards::*;
use rbp_core::*;
use std::hash::Hash;

/// A discrete bucket representing strategically similar hands.
///
/// Abstraction collapses the vast observation space (~3 trillion states) into
/// a manageable number of buckets for strategy storage. The bucket assignment
/// varies by street:
///
/// - **River**: Equity buckets (0–100, representing win probability)
/// - **Preflop**: 169 strategically-unique starting hands
/// - **Flop/Turn**: K-means cluster assignments based on next-street distributions
///
/// # Encoding
///
/// Packed as `[8 bits street][8 bits index]` in a `u16`, enabling efficient
/// storage and comparison.
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "client", derive(serde::Serialize, serde::Deserialize))]
pub struct Abstraction(u16);

/// Implement Support for transport traits (Density, Coupling, Measure).
impl rbp_transport::Support for Abstraction {}

const INDEX_MASK: u16 = 0xFF;
const STREET_BITS: u16 = 8;
const STREET_MASK: u16 = 0xFF << STREET_BITS;

impl Abstraction {
    pub const DELIM: &'static str = "::";
    /// Maximum bucket index (for river equity buckets).
    pub const N: usize = rbp_core::KMEANS_EQTY_CLUSTER_COUNT - 1;
    /// Number of equity buckets.
    pub const fn size() -> usize {
        rbp_core::KMEANS_EQTY_CLUSTER_COUNT
    }
    /// Iterates over all river abstractions (equity buckets).
    pub fn range() -> impl Iterator<Item = Self> {
        (0..=Self::N).map(|i| Self::from((Street::Rive, i)))
    }
    /// Extracts the street from the packed representation.
    pub const fn street(&self) -> Street {
        match (self.0 & STREET_MASK) >> STREET_BITS {
            0 => Street::Pref,
            1 => Street::Flop,
            2 => Street::Turn,
            3 => Street::Rive,
            _ => panic!("invalid street"),
        }
    }
    /// Extracts the bucket index.
    pub const fn index(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }
    /// All abstractions for a given street.
    pub fn all(street: Street) -> Vec<Self> {
        if street == Street::Rive {
            Self::range().collect()
        } else {
            (0..street.k()).map(|i| Self::from((street, i))).collect()
        }
    }
    fn quantize(p: Probability) -> usize {
        (p * Self::N as Probability).round() as usize
    }
    fn floatize(q: usize) -> Probability {
        q as Probability / Self::N as Probability
    }
}

impl From<(Street, usize)> for Abstraction {
    fn from((street, index): (Street, usize)) -> Self {
        let hi_bits = (street as u16) << STREET_BITS;
        let lo_bits = index as u16 & INDEX_MASK;
        Self(hi_bits | lo_bits)
    }
}

impl From<Street> for Abstraction {
    fn from(street: Street) -> Self {
        Self::from((street, rand::random_range(0..street.n_abstractions())))
    }
}

/// probability isomorphism
///
/// for river, we use a u8 to represent the equity bucket,
/// i.e. Equity(0) is the 0% equity bucket,
/// and Equity(N) is the 100% equity bucket.
impl From<Probability> for Abstraction {
    fn from(p: Probability) -> Self {
        debug_assert!(p >= 0.);
        debug_assert!(p <= 1.);
        Self::from((Street::Rive, Self::quantize(p)))
    }
}

impl From<Abstraction> for Probability {
    fn from(abstraction: Abstraction) -> Self {
        match abstraction.street() {
            Street::Rive => Abstraction::floatize(abstraction.index()),
            _ => unreachable!("no non-river into probability"),
        }
    }
}

/// u16 isomorphism
///
/// conversion to u16 for storage.
impl From<Abstraction> for u16 {
    fn from(a: Abstraction) -> Self {
        a.0
    }
}

impl From<u16> for Abstraction {
    fn from(n: u16) -> Self {
        Self(n)
    }
}

/// i16 isomorphism
///
/// conversion to i16 for SQL storage (SMALLINT).
impl From<Abstraction> for i16 {
    fn from(abstraction: Abstraction) -> Self {
        abstraction.0 as i16
    }
}

impl From<i16> for Abstraction {
    fn from(n: i16) -> Self {
        Self(n as u16)
    }
}

/// string isomorphism
impl TryFrom<&str> for Abstraction {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.trim().split(Self::DELIM).collect::<Vec<_>>();
        let a = s
            .get(0)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("broken delimiter"))?;
        let b = s
            .get(1)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("broken delimiter"))?;
        let street = Street::try_from(a).map_err(|e| anyhow::anyhow!(e))?;
        let index = usize::from_str_radix(b, 16).map_err(|e| anyhow::anyhow!(e))?;
        Ok(Abstraction::from((street, index)))
    }
}

impl std::fmt::Display for Abstraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{:02x}",
            self.street().symbol(),
            Self::DELIM,
            self.index()
        )
    }
}

impl Arbitrary for Abstraction {
    fn random() -> Self {
        let street = Street::Flop;
        let k = street.k();
        let i = rand::random_range(0..k);
        Abstraction::from((street, i))
    }
}

#[cfg(feature = "database")]
impl rbp_pg::Schema for Abstraction {
    fn name() -> &'static str {
        rbp_pg::ABSTRACTION
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            rbp_pg::ABSTRACTION,
            " (
                abs         SMALLINT,
                street      SMALLINT,
                population  INTEGER,
                equity      REAL
            );
            TRUNCATE TABLE ",
            rbp_pg::ABSTRACTION,
            ";
            CREATE OR REPLACE FUNCTION get_population(xxx SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM ",
            rbp_pg::ISOMORPHISM,
            " e WHERE e.abs = xxx); END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_street_abs(abs SMALLINT) RETURNS SMALLINT AS
            $$ BEGIN RETURN ((abs >> 8) & 255)::SMALLINT; END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_equity(parent SMALLINT) RETURNS REAL AS
            $$ BEGIN RETURN CASE WHEN get_street_abs(parent) = 3
                THEN (parent & 255)::REAL / 100
                ELSE (
                    SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
                    FROM ",
            rbp_pg::TRANSITIONS,
            " t
                    JOIN ",
            rbp_pg::ABSTRACTION,
            " r ON t.next = r.abs
             WHERE t.prev = parent) END; END; $$
            LANGUAGE plpgsql;"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_",
            rbp_pg::ABSTRACTION,
            "_abs ON ",
            rbp_pg::ABSTRACTION,
            " (abs);
             CREATE INDEX IF NOT EXISTS idx_",
            rbp_pg::ABSTRACTION,
            "_st  ON ",
            rbp_pg::ABSTRACTION,
            " (street);
             CREATE INDEX IF NOT EXISTS idx_",
            rbp_pg::ABSTRACTION,
            "_eq  ON ",
            rbp_pg::ABSTRACTION,
            " (equity);
             CREATE INDEX IF NOT EXISTS idx_",
            rbp_pg::ABSTRACTION,
            "_pop ON ",
            rbp_pg::ABSTRACTION,
            " (population);"
        )
    }
    fn copy() -> &'static str {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", rbp_pg::ABSTRACTION, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            rbp_pg::ABSTRACTION,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            rbp_pg::ABSTRACTION,
            " SET (autovacuum_enabled = false);"
        )
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
}

#[cfg(feature = "database")]
impl rbp_pg::Derive for Abstraction {
    fn exhaust() -> Vec<Self> {
        Street::exhaust()
            .into_iter()
            .map(Self::all)
            .map(Vec::into_iter)
            .flatten()
            .collect()
    }
    fn inserts(&self) -> String {
        let abs = i16::from(*self);
        format!(
            "INSERT INTO {} (abs, street, equity, population) VALUES ({}, get_street_abs({}::SMALLINT), get_equity({}::SMALLINT), get_population({}::SMALLINT));",
            rbp_pg::ABSTRACTION,
            abs,
            abs,
            abs,
            abs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;
    #[test]
    fn is_quantize_inverse_floatize() {
        for p in (0..=100).map(|x| x as Probability / 100.) {
            let q = Abstraction::quantize(p);
            let f = Abstraction::floatize(q);
            assert!((p - f).abs() < 1. / Abstraction::N as Probability);
        }
    }
    #[test]
    fn is_floatize_inverse_quantize() {
        for q in 0..=Abstraction::N {
            let p = Abstraction::floatize(q);
            let i = Abstraction::quantize(p);
            assert!(q == i);
        }
    }
    #[test]
    fn bijective_u16_random() {
        let random = Abstraction::random();
        assert_eq!(random, Abstraction::from(u16::from(random)));
    }
    #[test]
    fn bijective_u16_equity() {
        let equity = Abstraction::from(Observation::from(Street::Rive).equity());
        assert_eq!(equity, Abstraction::from(u16::from(equity)));
    }
    #[test]
    fn bijective_str() {
        let abs = Abstraction::random();
        let str = format!("{}", abs);
        assert_eq!(abs, Abstraction::try_from(str.as_str()).unwrap());
    }
    #[test]
    fn street_index_roundtrip() {
        for street in Street::all() {
            for i in 0..street.n_abstractions() {
                let abs = Abstraction::from((street, i));
                assert_eq!(abs.street(), street);
                assert_eq!(abs.index(), i);
            }
        }
    }
}

