use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;

/// Distance metric between abstractions for a specific street.
///
/// Provides two key operations:
/// 1. Point-to-point distance between abstractions (via `distance()`)
/// 2. Histogram-to-histogram EMD using Sinkhorn optimal transport (via `emd()`)
///
/// # Street-Specific Storage
///
/// Uses triangular [`Distances`] arrays to store pairwise distances:
/// - Preflop/Flop/Turn: Precomputed from clustering, loaded from database
/// - River: Uses raw equity difference (no precomputation needed)
///
/// # EMD Computation
///
/// For Flop/Turn histograms, EMD is computed via Sinkhorn algorithm using
/// this metric as the ground distance. River histograms use total variation
/// distance since equity abstractions have a natural ordering on [0,1].
#[derive(Clone, Copy)]
pub enum Metric {
    Pref(DistPref),
    Flop(DistFlop),
    Turn(DistTurn),
    Rive,
}

impl Default for Metric {
    fn default() -> Self {
        Metric::Pref(Distances::new(Street::Pref))
    }
}

impl Metric {
    /// Internal distance computation taking raw Abstraction references.
    /// Used by Sinkhorn and other internal code that works with Abstraction directly.
    pub fn raw_distance(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        if x == y {
            0.
        } else {
            match (x.street(), y.street()) {
                (Street::Pref, Street::Pref)
                | (Street::Flop, Street::Flop)
                | (Street::Turn, Street::Turn) => self.lookup(x, y),
                (Street::Rive, Street::Rive) => {
                    (Probability::from(*x) - Probability::from(*y)).abs()
                }
                _ => unreachable!("mismatched streets"),
            }
        }
    }
}

impl Measure for Metric {
    type X = ClusterAbs;
    type Y = ClusterAbs;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> Energy {
        self.raw_distance(x, y)
    }
}

impl Metric {
    /// Creates a new metric for the given street with zero distances.
    pub const fn new(street: Street) -> Self {
        match street {
            Street::Pref => Metric::Pref(Distances::new(street)),
            Street::Flop => Metric::Flop(Distances::new(street)),
            Street::Turn => Metric::Turn(Distances::new(street)),
            Street::Rive => Metric::Rive,
        }
    }
    /// The street this metric measures distances for.
    pub fn street(&self) -> Street {
        match self {
            Metric::Pref(_) => Street::Pref,
            Metric::Flop(_) => Street::Flop,
            Metric::Turn(_) => Street::Turn,
            Metric::Rive => Street::Rive,
        }
    }
    /// Looks up precomputed distance between two abstractions.
    fn lookup(&self, x: &Abstraction, y: &Abstraction) -> Energy {
        let pair = Pair::from((x, y));
        match self {
            Metric::Pref(d) => d.get(pair),
            Metric::Flop(d) => d.get(pair),
            Metric::Turn(d) => d.get(pair),
            Metric::Rive => unreachable!("no metric over Histogram<River>"),
        }
    }
    /// Sets distance value for a pair of abstractions.
    pub fn set(&mut self, pair: Pair, value: Energy) {
        match self {
            Metric::Pref(d) => d.set(pair, value),
            Metric::Flop(d) => d.set(pair, value),
            Metric::Turn(d) => d.set(pair, value),
            Metric::Rive => unreachable!("no metric over Histogram<River>"),
        }
    }
    /// Computes Earth Mover's Distance between two histograms.
    ///
    /// For Flop/Turn: Uses Sinkhorn entropic optimal transport.
    /// For River: Uses total variation (integrated CDF difference).
    pub fn emd(&self, source: &Histogram, target: &Histogram) -> Energy {
        match source.peek().street() {
            Street::Flop | Street::Turn => Sinkhorn::from((source, target, self)).minimize().cost(),
            Street::Rive => Equity::variation(source, target),
            Street::Pref => unreachable!("no preflop emd"),
        }
    }
    /// Normalize all distances by the maximum value.
    pub fn normalize(&mut self) {
        match self {
            Metric::Pref(d) => d.normalize(),
            Metric::Flop(d) => d.normalize(),
            Metric::Turn(d) => d.normalize(),
            Metric::Rive => {}
        }
    }
}

impl From<std::collections::BTreeMap<Pair, Energy>> for Metric {
    fn from(map: std::collections::BTreeMap<Pair, Energy>) -> Self {
        let max = map.values().copied().fold(f32::MIN_POSITIVE, f32::max);
        let mut metric = map
            .keys()
            .next()
            .map(|p| p.street())
            .map(Metric::new)
            .expect("map is empty");
        for (pair, distance) in map {
            metric.set(pair, distance / max);
        }
        metric
    }
}

impl IntoIterator for Metric {
    type Item = (i32, Energy);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + Send>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Metric::Pref(d) => d.into_iter(),
            Metric::Flop(d) => d.into_iter(),
            Metric::Turn(d) => d.into_iter(),
            Metric::Rive => unreachable!(),
        }
    }
}

#[cfg(feature = "database")]
impl rbp_database::Schema for Metric {
    fn name() -> &'static str {
        rbp_database::METRIC
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT4,   // tri (triangular index)
            tokio_postgres::types::Type::FLOAT4, // dx (distance)
        ]
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            rbp_database::METRIC,
            " (
                tri    INT  NOT NULL,
                dx     REAL NOT NULL,
                street SMALLINT
            );"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_metric_tri ON ",
            rbp_database::METRIC,
            " (tri);
             CREATE INDEX IF NOT EXISTS idx_metric_street ON ",
            rbp_database::METRIC,
            " (street);"
        )
    }
    fn copy() -> &'static str {
        const_format::concatcp!(
            "COPY ",
            rbp_database::METRIC,
            " (tri, dx) FROM STDIN BINARY"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", rbp_database::METRIC, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            rbp_database::METRIC,
            " SET (fillfactor = 100);
             ALTER TABLE ",
            rbp_database::METRIC,
            " SET (autovacuum_enabled = false);"
        )
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl rbp_database::Streamable for Metric {
    type Row = (i32, f32);
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.into_iter()
    }
}

#[cfg(feature = "database")]
impl Metric {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        let sql = const_format::concatcp!("SELECT tri, dx FROM ", rbp_database::METRIC);
        let mut keys = std::collections::HashSet::new();
        for ref x in Abstraction::all(street) {
            for ref y in Abstraction::all(street) {
                if x < y {
                    keys.insert(i32::from(Pair::from((x, y))));
                }
            }
        }
        let mut metric = Metric::new(street);
        client
            .query(sql, &[])
            .await
            .expect("query")
            .into_iter()
            .map(|row| (row.get::<_, i32>(0), row.get::<_, f32>(1)))
            .filter(|(tri, _)| keys.contains(tri))
            .map(|(tri, dx)| (Pair::from(tri), dx))
            .for_each(|(pair, dx)| metric.set(pair, dx));
        metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pair_triangular_roundtrip() {
        for k in [10, 50, 100] {
            for i in 0..k {
                for j in (i + 1)..k {
                    let pair = Pair::new(Street::Flop, i, j);
                    let (ri, rj) = pair.indices();
                    assert_eq!((i, j), (ri, rj), "roundtrip failed for ({}, {})", i, j);
                }
            }
        }
    }
    #[test]
    fn pair_abstractions_roundtrip() {
        let street = Street::Flop;
        let a = Abstraction::from((street, 5));
        let b = Abstraction::from((street, 10));
        let pair = Pair::from((&a, &b));
        let (ra, rb) = pair.abstractions();
        assert_eq!(a, ra);
        assert_eq!(b, rb);
    }
}
