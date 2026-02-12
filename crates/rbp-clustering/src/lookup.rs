use crate::*;
use rayon::prelude::*;
use rbp_cards::*;
use rbp_gameplay::*;
use std::collections::BTreeMap;

/// Mapping from hand isomorphisms to abstraction buckets.
///
/// This is the primary output of clustering: given any poker hand
/// (represented as a suit-isomorphic [`Isomorphism`]), look up which
/// strategic [`Abstraction`] bucket it belongs to.
///
/// # Construction
///
/// - River: Computed directly from showdown equity
/// - Preflop: One bucket per isomorphism (no abstraction)
/// - Flop/Turn: Learned via k-means clustering over next-street histograms
///
/// # Database
///
/// With the `database` feature, supports streaming to/from PostgreSQL
/// for persistence between training runs.
#[derive(Default)]
pub struct Lookup(BTreeMap<Isomorphism, Abstraction>);

impl From<Lookup> for BTreeMap<Isomorphism, Abstraction> {
    fn from(lookup: Lookup) -> BTreeMap<Isomorphism, Abstraction> {
        lookup.0
    }
}
impl From<BTreeMap<Isomorphism, Abstraction>> for Lookup {
    fn from(map: BTreeMap<Isomorphism, Abstraction>) -> Self {
        Self(map)
    }
}

impl Lookup {
    /// Looks up the abstraction for a hand isomorphism.
    pub fn lookup(&self, iso: &Isomorphism) -> Abstraction {
        self.0
            .get(iso)
            .cloned()
            .expect("precomputed abstraction in lookup")
    }

    /// Generates histograms for all isomorphisms at the previous street.
    /// Used to build the data points for the next clustering layer.
    pub fn projections(&self) -> Vec<Histogram> {
        IsomorphismIterator::from(self.street().prev())
            .collect::<Vec<Isomorphism>>()
            .into_par_iter()
            .map(|i| self.future(&i))
            .collect::<Vec<Histogram>>()
    }

    /// Computes histogram over next-street abstractions for an isomorphism.
    /// This is the core operation that enables hierarchical clustering.
    fn future(&self, iso: &Isomorphism) -> Histogram {
        debug_assert!(iso.0.street() != Street::Rive);
        iso.0
            .children()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(Isomorphism::from)
            .map(|i| self.lookup(&i))
            .collect::<Vec<Abstraction>>()
            .into()
    }

    /// The street this lookup is for.
    fn street(&self) -> Street {
        self.0.keys().next().expect("non empty").0.street()
    }
}

#[cfg(feature = "database")]
impl rbp_pg::Schema for Lookup {
    fn name() -> &'static str {
        rbp_pg::ISOMORPHISM
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8, // obs (observation/isomorphism)
            tokio_postgres::types::Type::INT2, // abs (abstraction bucket)
        ]
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            rbp_pg::ISOMORPHISM,
            " (
                obs      BIGINT   NOT NULL,
                abs      SMALLINT NOT NULL,
                equity   REAL,
                position INT DEFAULT 0
            );"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_isomorphism_obs ON ",
            rbp_pg::ISOMORPHISM,
            " (obs);
             CREATE INDEX IF NOT EXISTS idx_isomorphism_abs ON ",
            rbp_pg::ISOMORPHISM,
            " (abs);
             CREATE INDEX IF NOT EXISTS idx_isomorphism_abs_pos ON ",
            rbp_pg::ISOMORPHISM,
            " (abs, position);"
        )
    }
    fn copy() -> &'static str {
        const_format::concatcp!(
            "COPY ",
            rbp_pg::ISOMORPHISM,
            " (obs, abs) FROM STDIN BINARY"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", rbp_pg::ISOMORPHISM, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            rbp_pg::ISOMORPHISM,
            " SET (fillfactor = 100);
             ALTER TABLE ",
            rbp_pg::ISOMORPHISM,
            " SET (autovacuum_enabled = false);"
        )
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl rbp_pg::Streamable for Lookup {
    type Row = (i64, i16);
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0
            .into_iter()
            .map(|(iso, abs)| (i64::from(iso), i16::from(abs)))
    }
}

#[cfg(feature = "database")]
impl Lookup {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        let sql = const_format::concatcp!("SELECT obs, abs FROM ", rbp_pg::ISOMORPHISM);
        client
            .query(sql, &[])
            .await
            .expect("query")
            .into_iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, i16>(1)))
            .filter(|(obs, _)| Street::from(*obs) == street)
            .map(|(obs, abs)| (Isomorphism::from(obs), Abstraction::from(abs)))
            .collect::<BTreeMap<_, _>>()
            .into()
    }
}

impl Lookup {
    /// Creates lookup tables for streets that don't require clustering.
    ///
    /// - River: Uses equity as abstraction (discretized win probability)
    /// - Preflop: Each isomorphism gets its own bucket (no compression)
    pub fn grow(street: Street) -> Self {
        match street {
            Street::Rive => IsomorphismIterator::from(Street::Rive)
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|iso| (iso, Abstraction::from(iso.0.equity())))
                .collect::<BTreeMap<_, _>>()
                .into(),
            Street::Pref => IsomorphismIterator::from(Street::Pref)
                .enumerate()
                .map(|(k, iso)| (iso, Abstraction::from((Street::Pref, k))))
                .collect::<BTreeMap<_, _>>()
                .into(),
            Street::Flop | Street::Turn => panic!("lookup must be learned via layer for {street}"),
        }
    }
}
