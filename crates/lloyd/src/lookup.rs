use crate::*;
use deuce::*;
use kicker::*;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Mapping from hand isomorphisms to abstraction buckets — the primary output
/// of clustering. River buckets come straight from showdown equity, preflop
/// gets one bucket per isomorphism (no abstraction), and flop/turn are learned
/// by k-means over next-street histograms.
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
    /// The abstraction for a hand isomorphism.
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

    /// Histogram over next-street abstractions for an isomorphism — the
    /// operation hierarchical clustering is built on.
    fn future(&self, iso: &Isomorphism) -> Histogram {
        debug_assert_ne!(iso.0.street(), Street::Rive);
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
impl daybook::Schema for Lookup {
    fn name() -> &'static str {
        daybook::isomorphism()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT8, // obs (observation/isomorphism)
            tokio_postgres::types::Type::INT2, // abs (abstraction bucket)
            tokio_postgres::types::Type::INT4, // position (dense per-bucket index)
        ]
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                obs      BIGINT   NOT NULL,
                abs      SMALLINT NOT NULL,
                equity   REAL,
                position INT DEFAULT 0
            );",
                daybook::isomorphism()
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = daybook::isomorphism();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                // Index-only. `position` (the dense per-bucket index the topology
                // sampler reads) is computed in memory and streamed in via COPY —
                // see the `Streamable` impl below — so finalize no longer runs the
                // full-table `UPDATE ... position` that went disk-bound for hours
                // on the ~123M-row river table. `idx_covering` serves the
                // `obs -> abs` lookup in `nlhe::lookup`; `idx_abs_pos` serves the
                // sampler's `(abs, position)` access; `idx_abs_obs` serves
                // per-bucket scans.
                "CREATE INDEX IF NOT EXISTS idx_{t}_abs_obs ON {t} (abs, obs);
             CREATE INDEX IF NOT EXISTS idx_{t}_abs_pos ON {t} (abs, position);
             CREATE INDEX IF NOT EXISTS idx_{t}_covering ON {t} (obs, abs) INCLUDE (abs);"
            ))
        })
    }

    fn copy() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::leaked(format!("COPY {} (obs, abs, position) FROM STDIN BINARY", daybook::isomorphism()))
        })
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| daybook::leaked(format!("TRUNCATE TABLE {};", daybook::isomorphism())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = daybook::isomorphism();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                "ALTER TABLE {t} SET (fillfactor = 100);
             ALTER TABLE {t} SET (autovacuum_enabled = false);"
            ))
        })
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl daybook::Streamable for Lookup {
    type Row = Mapping;

    /// Yields `(obs, abs, position)`, where `position` is the dense per-bucket
    /// index (`0..population` within each `abs`) the topology sampler reads,
    /// computed in one pass so finalize is index-only. The sampler treats it as
    /// an opaque dense index (`e.position = FLOOR(RANDOM() * population)`), so
    /// the per-bucket order — here, canonical `Isomorphism` order — is
    /// deterministic but otherwise unobservable: no sort, no full-table UPDATE.
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0
            .into_iter()
            .scan(HashMap::<Abstraction, i32>::new(), |counts, (iso, abs)| {
                Some(Mapping::from((iso, abs, *counts.entry(abs).and_modify(|n| *n += 1).or_insert(0))))
            })
    }
}

#[cfg(feature = "server")]
impl Lookup {
    pub async fn from_street(client: &tokio_postgres::Client, street: Street) -> Self {
        let sql = format!("SELECT obs, abs FROM {}", daybook::isomorphism());
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
    /// Lookup for the streets that need no clustering: river discretizes
    /// equity, preflop gives each isomorphism its own bucket.
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
