#[cfg(feature = "server")]
use super::*;
#[cfg(feature = "server")]
use mccfr::*;
#[cfg(feature = "server")]
use std::sync::OnceLock;

/// The blueprint table: one row per info-edge pair of an [`Book<NlheInfo>`].
#[cfg(feature = "server")]
pub struct NlheBlueprint;

#[cfg(feature = "server")]
impl daybook::Schema for NlheBlueprint {
    fn name() -> &'static str {
        daybook::blueprint()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        static COLS: OnceLock<Vec<tokio_postgres::types::Type>> = OnceLock::<Vec<tokio_postgres::types::Type>>::new();
        COLS.get_or_init(|| {
            vec![
                kicker::Subgame::kind(),             // past (subgame path; INT8 heads-up, BYTEA multiway)
                tokio_postgres::types::Type::INT2,   // present (abstraction bucket)
                tokio_postgres::types::Type::INT8,   // choices (available edges)
                tokio_postgres::types::Type::INT4,   // context (button-anchored field)
                tokio_postgres::types::Type::INT8,   // edge (action taken)
                tokio_postgres::types::Type::FLOAT4, // weight
                tokio_postgres::types::Type::FLOAT4, // regret
                tokio_postgres::types::Type::FLOAT4, // payoff
                tokio_postgres::types::Type::INT4,   // visits
            ]
        })
    }

    fn copy() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!(
                "COPY {} (past, present, choices, context, edge, weight, regret, payoff, visits) FROM STDIN BINARY",
                daybook::blueprint()
            ))
        })
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                edge       BIGINT,
                past       {},
                present    SMALLINT,
                choices    BIGINT,
                context    INT,
                weight     REAL,
                regret     REAL,
                payoff     REAL,
                visits     INT DEFAULT 0,
                UNIQUE     (past, present, choices, context, edge)
            );",
                daybook::blueprint(),
                kicker::Subgame::sql(),
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = daybook::blueprint();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_{t}_upsert  ON {t} (present, past, choices, context, edge);
             CREATE        INDEX IF NOT EXISTS idx_{t}_bucket  ON {t} (present, past, choices, context);
             CREATE        INDEX IF NOT EXISTS idx_{t}_present ON {t} (present);
             CREATE        INDEX IF NOT EXISTS idx_{t}_edge    ON {t} (edge);
             CREATE        INDEX IF NOT EXISTS idx_{t}_past    ON {t} (past);"
            ))
        })
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| daybook::Pool::leaked(format!("TRUNCATE TABLE {};", daybook::blueprint())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = daybook::blueprint();
        SQL.get_or_init(|| {
            daybook::Pool::leaked(format!(
                "ALTER TABLE {t} SET (fillfactor = 100);
             ALTER TABLE {t} SET (autovacuum_enabled = false);"
            ))
        })
    }
}

#[cfg(feature = "server")]
impl NlheBlueprint {
    /// The blueprint profile, read back from the table at its current epoch.
    pub async fn profile(client: std::sync::Arc<tokio_postgres::Client>) -> Book<NlheInfo> {
        tracing::info!("{:<32}{:<32}", "loading blueprint", "from database");
        let epoch_sql = format!("SELECT value FROM {} WHERE key = 'current'", daybook::epoch());
        let epochs = client
            .query_opt(&epoch_sql, &[])
            .await
            .ok()
            .flatten()
            .map(|r| r.get::<_, i64>(0) as usize)
            .expect("to have already created epoch metadata");
        let blueprint_sql = format!(
            "SELECT past, present, choices, context, edge, weight, regret, payoff, visits FROM {}",
            daybook::blueprint()
        );
        let mut encounters = FxHashMap::default();
        for row in client
            .query(&blueprint_sql, &[])
            .await
            .expect("to have already created blueprint")
        {
            let subgame = row.get::<_, kicker::Subgame>(0);
            let present = row.get::<_, kicker::Abstraction>(1);
            let choices = row.get::<_, kicker::Path>(2);
            let field = row.get::<_, kicker::Field>(3);
            let edge = NlheEdge::from(row.get::<_, kicker::Edge>(4));
            let weight = row.get::<_, f32>(5);
            let regret = row.get::<_, f32>(6);
            let payoff = row.get::<_, f32>(7);
            let visits = row.get::<_, i32>(8) as u32;
            let bucket = NlheInfo::from((subgame, present, choices, field));
            encounters
                .entry(bucket)
                .or_insert_with(FxHashMap::default)
                .entry(edge)
                .or_insert(Encounter::new(weight, regret, payoff, visits));
        }
        tracing::info!("{:<32}{:<32}", format!("{} infos", encounters.len()), "from database");
        tracing::info!("{:<32}{:<32}", format!("{epochs} iters"), "from database");
        Book::new(epochs, encounters)
    }

    /// A solver over the stored blueprint and the stored abstraction.
    pub async fn solver<R, W, S>(client: std::sync::Arc<tokio_postgres::Client>) -> Tabular<NlheEncoder, R, W, S, 128>
    where
        R: RegretSchedule,
        W: WeightSchedule,
        S: SamplingScheme,
    {
        Tabular::new(Self::profile(client.clone()).await, <NlheEncoder as daybook::Hydrate>::hydrate(client).await)
    }
}
