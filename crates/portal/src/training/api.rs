use kicker::*;
use ledger::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

pub struct TrainingAPI(Arc<Client>);

impl TrainingAPI {
    pub fn new(client: Arc<Client>) -> Self {
        Self(client)
    }
}
// status + snapshots
impl TrainingAPI {
    pub async fn status(&self) -> anyhow::Result<ApiStatus> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT \
             (SELECT COALESCE(value, 0) FROM {} WHERE key = 'current') AS epoch, \
             (SELECT COUNT(DISTINCT (past, present, choices)) FROM {}) AS infosets, \
             (SELECT exploit FROM {} ORDER BY id DESC LIMIT 1) AS exploit, \
             (SELECT stamped FROM {} ORDER BY id DESC LIMIT 1) AS stamped",
                epoch(),
                blueprint(),
                snapshot(),
                snapshot()
            ))
        });
        self.0
            .query_one(sql, &[])
            .await
            .map(|r| ApiStatus {
                epoch: r.get::<_, Option<i64>>(0).unwrap_or(0),
                infosets: r.get::<_, Option<i64>>(1).unwrap_or(0),
                exploit: r.get(2),
                stamped: r.get(3),
            })
            .map_err(|e| anyhow::anyhow!("fetch status: {e}"))
    }

    pub async fn snapshots(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<ApiSnapshot>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT epoch, infos, nodes, exploit, elapsed, stamped \
             FROM {} \
             ORDER BY id DESC \
             LIMIT $1 OFFSET $2",
                snapshot()
            ))
        });
        self.0
            .query(sql, &[&limit, &offset])
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|r| ApiSnapshot {
                        epoch: r.get(0),
                        infos: r.get(1),
                        nodes: r.get(2),
                        exploit: r.get(3),
                        elapsed: r.get(4),
                        stamped: r.get(5),
                    })
                    .collect()
            })
            .map_err(|e| anyhow::anyhow!("fetch snapshots: {e}"))
    }
}
// aggregate stats
impl TrainingAPI {
    pub async fn stats(&self) -> anyhow::Result<ApiBlueprintStats> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT \
             COUNT(DISTINCT (past, present, choices))::BIGINT, \
             COUNT(*)::BIGINT, \
             (AVG(regret))::float4, \
             MAX(regret)::float4, \
             MIN(regret)::float4, \
             (AVG(weight))::float4, \
             MAX(weight)::float4, \
             (AVG(payoff))::float4, \
             MAX(payoff)::float4, \
             MIN(payoff)::float4, \
             (AVG(visits::float4))::float4, \
             MAX(visits), \
             MIN(visits) \
             FROM {}",
                blueprint()
            ))
        });
        self.0
            .query_one(sql, &[])
            .await
            .map(|r| ApiBlueprintStats {
                infosets: r.get(0),
                edges: r.get(1),
                avg_regret: r.get(2),
                max_regret: r.get(3),
                min_regret: r.get(4),
                avg_weight: r.get(5),
                max_weight: r.get(6),
                avg_payoff: r.get(7),
                max_payoff: r.get(8),
                min_payoff: r.get(9),
                avg_visits: r.get(10),
                max_visits: r.get(11),
                min_visits: r.get(12),
            })
            .map_err(|e| anyhow::anyhow!("fetch blueprint stats: {e}"))
    }

    pub async fn street_stats(&self) -> anyhow::Result<Vec<ApiStreetStats>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT \
             CASE a.street \
               WHEN 0 THEN 'P' \
               WHEN 1 THEN 'F' \
               WHEN 2 THEN 'T' \
               WHEN 3 THEN 'R' \
               ELSE '?' \
             END AS street, \
             COUNT(DISTINCT (b.past, b.present, b.choices))::BIGINT, \
             COUNT(*)::BIGINT, \
             (AVG(b.regret))::float4, \
             (AVG(b.weight))::float4, \
             (AVG(b.payoff))::float4, \
             (AVG(b.visits::float4))::float4 \
             FROM {} b \
             JOIN {} a ON a.abs = b.present \
             GROUP BY a.street \
             ORDER BY a.street",
                blueprint(),
                abstraction()
            ))
        });
        self.0
            .query(sql, &[])
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|r| ApiStreetStats {
                        street: r.get(0),
                        infosets: r.get(1),
                        edges: r.get(2),
                        avg_regret: r.get(3),
                        avg_weight: r.get(4),
                        avg_payoff: r.get(5),
                        avg_visits: r.get(6),
                    })
                    .collect()
            })
            .map_err(|e| anyhow::anyhow!("fetch street stats: {e}"))
    }
}
// cold + hot infosets
impl TrainingAPI {
    pub async fn cold(&self, limit: i64) -> anyhow::Result<Vec<ApiColdInfoset>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT past, present, choices, MIN(visits), COUNT(*)::BIGINT \
             FROM {} \
             GROUP BY past, present, choices \
             ORDER BY MIN(visits) ASC \
             LIMIT $1",
                blueprint()
            ))
        });
        self.0
            .query(sql, &[&limit])
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|r| ApiColdInfoset {
                        past: r.get(0),
                        present: r.get(1),
                        choices: r.get(2),
                        visits: r.get(3),
                        edges: r.get(4),
                    })
                    .collect()
            })
            .map_err(|e| anyhow::anyhow!("fetch cold infosets: {e}"))
    }

    pub async fn hot(&self, limit: i64) -> anyhow::Result<Vec<ApiHotInfoset>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT past, present, choices, MAX(regret)::float4, COUNT(*)::BIGINT \
             FROM {} \
             GROUP BY past, present, choices \
             ORDER BY MAX(ABS(regret)) DESC \
             LIMIT $1",
                blueprint()
            ))
        });
        self.0
            .query(sql, &[&limit])
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|r| ApiHotInfoset {
                        past: r.get(0),
                        present: r.get(1),
                        choices: r.get(2),
                        max_regret: r.get(3),
                        edges: r.get(4),
                    })
                    .collect()
            })
            .map_err(|e| anyhow::anyhow!("fetch hot infosets: {e}"))
    }
}
// convergence + saturation
impl TrainingAPI {
    pub async fn convergence(&self, limit: i64) -> anyhow::Result<Vec<ApiConvergence>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT epoch, exploit, \
             (exploit - LAG(exploit) OVER (ORDER BY id))::float4 AS delta, \
             stamped \
             FROM {} \
             WHERE exploit IS NOT NULL \
             ORDER BY id DESC \
             LIMIT $1",
                snapshot()
            ))
        });
        self.0
            .query(sql, &[&limit])
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|r| ApiConvergence {
                        epoch: r.get(0),
                        exploit: r.get(1),
                        delta: r.get::<_, Option<f32>>(2).unwrap_or(0.),
                        stamped: r.get(3),
                    })
                    .collect()
            })
            .map_err(|e| anyhow::anyhow!("fetch convergence: {e}"))
    }

    pub async fn saturation(&self) -> anyhow::Result<ApiSaturation> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "SELECT \
             MAX(weight)::float4, \
             MAX(ABS(regret))::float4, \
             MAX(ABS(payoff))::float4, \
             MAX(visits) \
             FROM {}",
                blueprint()
            ))
        });
        let precision = f32::MAX;
        self.0
            .query_one(sql, &[])
            .await
            .map(|r| {
                let weight: f32 = r.get(0);
                let regret: f32 = r.get(1);
                ApiSaturation {
                    max_weight: weight,
                    max_regret: regret,
                    max_payoff: r.get(2),
                    max_visits: r.get(3),
                    precision_f32: precision,
                    weight_pct: weight / precision * 100.,
                    regret_pct: regret / precision * 100.,
                }
            })
            .map_err(|e| anyhow::anyhow!("fetch saturation: {e}"))
    }
}
