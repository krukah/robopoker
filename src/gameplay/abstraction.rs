use crate::cards::*;
use crate::transport::*;
use crate::*;
use std::hash::Hash;

/// Abstraction represents a lookup value for a given set of Observations.
/// Packed as: [8 bits street][8 bits index]
///
/// - River: index represents the equity bucket (0-100)
/// - Pre-Flop: index represents one of 169 strategically-unique hands
/// - Flop/Turn: index represents k-means cluster assignment
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "client", derive(serde::Serialize, serde::Deserialize))]
pub struct Abstraction(u16);

const INDEX_MASK: u16 = 0xFF;
const STREET_BITS: u16 = 8;
const STREET_MASK: u16 = 0xFF << STREET_BITS;

impl Abstraction {
    pub const DELIM: &'static str = "::";
    pub const N: usize = crate::KMEANS_EQTY_CLUSTER_COUNT - 1;
    pub const fn size() -> usize {
        crate::KMEANS_EQTY_CLUSTER_COUNT
    }
    pub fn range() -> impl Iterator<Item = Self> {
        (0..=Self::N).map(|i| Self::from((Street::Rive, i)))
    }
    pub const fn street(&self) -> Street {
        match (self.0 & STREET_MASK) >> STREET_BITS {
            0 => Street::Pref,
            1 => Street::Flop,
            2 => Street::Turn,
            3 => Street::Rive,
            _ => panic!("invalid street"),
        }
    }
    pub const fn index(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }
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
        assert!(p >= 0.);
        assert!(p <= 1.);
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

/// i64 isomorphism (convenience, for legacy compatibility)
#[doc(hidden)]
#[warn(deprecated)]
impl From<Abstraction> for i64 {
    fn from(abstraction: Abstraction) -> Self {
        i16::from(abstraction) as i64
    }
}

#[doc(hidden)]
#[warn(deprecated)]
impl From<i64> for Abstraction {
    fn from(n: i64) -> Self {
        Self::from(n as i16)
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

impl Support for Abstraction {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arbitrary;
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

#[cfg(feature = "database")]
impl crate::save::Schema for Abstraction {
    fn name() -> &'static str {
        crate::save::ABSTRACTION
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            crate::save::ABSTRACTION,
            " (
                abs         SMALLINT,
                street      SMALLINT,
                population  INTEGER,
                equity      REAL
            );
            TRUNCATE TABLE ",
            crate::save::ABSTRACTION,
            ";
            CREATE OR REPLACE FUNCTION get_population(xxx SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM ",
            crate::save::ISOMORPHISM,
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
            crate::save::TRANSITIONS,
            " t
                    JOIN ",
            crate::save::ABSTRACTION,
            " r ON t.next = r.abs
             WHERE t.prev = parent) END; END; $$
            LANGUAGE plpgsql;"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ABSTRACTION,
            "_abs ON ",
            crate::save::ABSTRACTION,
            " (abs);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ABSTRACTION,
            "_st  ON ",
            crate::save::ABSTRACTION,
            " (street);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ABSTRACTION,
            "_eq  ON ",
            crate::save::ABSTRACTION,
            " (equity);
             CREATE INDEX IF NOT EXISTS idx_",
            crate::save::ABSTRACTION,
            "_pop ON ",
            crate::save::ABSTRACTION,
            " (population);"
        )
    }
    fn copy() -> &'static str {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", crate::save::ABSTRACTION, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            crate::save::ABSTRACTION,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            crate::save::ABSTRACTION,
            " SET (autovacuum_enabled = false);"
        )
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
}

#[cfg(feature = "database")]
impl crate::save::Derive for Abstraction {
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
            crate::save::ABSTRACTION,
            abs,
            abs,
            abs,
            abs
        )
    }
}
