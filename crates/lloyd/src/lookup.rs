use crate::*;
use cowboys::*;
use kicker::*;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::sync::OnceLock;

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
        self.0.get(iso).copied().expect("precomputed abstraction in lookup")
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

#[cfg(feature = "server")]
impl ledger::Schema for Lookup {
    fn name() -> &'static str {
        ledger::isomorphism()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8, // obs (observation/isomorphism)
            tokio_postgres::types::Type::INT2, // abs (abstraction bucket)
        ]
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                obs      BIGINT   NOT NULL,
                abs      SMALLINT NOT NULL,
                equity   REAL,
                position INT DEFAULT 0
            );",
                ledger::isomorphism()
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = ledger::isomorphism();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "WITH numbered AS (
                SELECT obs, (ROW_NUMBER() OVER (PARTITION BY abs ORDER BY obs) - 1)::INTEGER AS pos
                FROM {t}
             )
             UPDATE {t} i SET position = n.pos
             FROM numbered n
             WHERE i.obs = n.obs AND i.position IS DISTINCT FROM n.pos;
             CREATE INDEX IF NOT EXISTS idx_{t}_obs ON {t} (obs);
             CREATE INDEX IF NOT EXISTS idx_{t}_abs ON {t} (abs);
             CREATE INDEX IF NOT EXISTS idx_{t}_abs_pos ON {t} (abs, position);
             CREATE INDEX IF NOT EXISTS idx_{t}_abs_obs ON {t} (abs, obs);
             CREATE INDEX IF NOT EXISTS idx_{t}_covering ON {t} (obs, abs) INCLUDE (abs);"
            ))
        })
    }

    fn copy() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| ledger::leaked(format!("COPY {} (obs, abs) FROM STDIN BINARY", ledger::isomorphism())))
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| ledger::leaked(format!("TRUNCATE TABLE {};", ledger::isomorphism())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = ledger::isomorphism();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "ALTER TABLE {t} SET (fillfactor = 100);
             ALTER TABLE {t} SET (autovacuum_enabled = false);"
            ))
        })
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl ledger::Streamable for Lookup {
    type Row = (i64, i16);

    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0.into_iter().map(|(iso, abs)| (i64::from(iso), i16::from(abs)))
    }
}

#[cfg(feature = "server")]
impl Lookup {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        let sql = format!("SELECT obs, abs FROM {}", ledger::isomorphism());
        client
            .query(&sql, &[])
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
