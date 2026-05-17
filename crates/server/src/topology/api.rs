use rbp_cards::*;
use rbp_clustering::*;
use rbp_core::*;
use rbp_database::*;
use rbp_gameplay::*;
use rbp_transport::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

const N_NEIGHBORS: i64 = 6;

// Local conversion functions for database types.
// These bridge tokio_postgres::Row to our domain types.
// We use free functions instead of From traits due to orphan rules.

fn api_sample_from_row(row: tokio_postgres::Row) -> ApiSample {
    ApiSample {
        obs: Observation::from(row.get::<_, i64>("obs")),
        abs: Abstraction::from(row.get::<_, i16>("abs")),
        equity: row.get::<_, f32>("equity"),
        density: row.get::<_, f32>("density"),
        distance: row.try_get::<_, f32>("distance").unwrap_or_default(),
    }
}

pub struct TopologyAPI(Arc<Client>);

impl From<Arc<Client>> for TopologyAPI {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl TopologyAPI {
    pub fn new(client: Arc<Client>) -> Self {
        Self(client)
    }

    pub fn client(&self) -> &Arc<Client> {
        &self.0
    }
}

// global lookups
impl TopologyAPI {
    pub async fn obs_to_abs(&self, obs: Observation) -> anyhow::Result<Abstraction> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT abs FROM {} WHERE obs = $1", isomorphism()));
        let iso = Isomorphism::from(obs);
        let idx = i64::from(iso);
        self.0
            .query_one(sql.as_str(), &[&idx])
            .await
            .map(|row| Abstraction::from(row.get::<_, i16>(0)))
            .map_err(|e| anyhow::anyhow!("fetch abstraction: {}", e))
    }

    pub async fn metric(&self, street: Street) -> anyhow::Result<Metric> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT tri, dx FROM {} WHERE street = $1", metric()));
        let s = street as i16;
        let rows = self
            .0
            .query(sql.as_str(), &[&s])
            .await
            .map_err(|e| anyhow::anyhow!("fetch metric: {}", e))?;
        let mut metric = Metric::new(street);
        for row in rows {
            let tri = row.get::<_, i32>(0);
            let dx = row.get::<_, Energy>(1);
            metric.set(Pair::from(tri), dx);
        }
        Ok(metric)
    }
}

// equity calculations
impl TopologyAPI {
    pub async fn abs_equity(&self, abs: Abstraction) -> anyhow::Result<Probability> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("SELECT equity FROM {} WHERE abs = $1", abstraction()));
        let abs = i16::from(abs);
        self.0
            .query_one(sql.as_str(), &[&abs])
            .await
            .map(|row| Probability::from(row.get::<_, f32>(0)))
            .map_err(|e| anyhow::anyhow!("fetch abstraction equity: {}", e))
    }

    pub async fn obs_equity(&self, obs: Observation) -> anyhow::Result<Probability> {
        static RIVER: OnceLock<String> = OnceLock::<String>::new();
        static OTHER: OnceLock<String> = OnceLock::<String>::new();
        let river =
            RIVER.get_or_init(|| format!("SELECT equity FROM {} WHERE obs = $1", isomorphism()));
        let other = OTHER.get_or_init(|| {
            format!(
                "SELECT SUM(t.dx * a.equity) \
             FROM   {} t \
             JOIN   {} e ON e.abs = t.prev \
             JOIN   {} a ON a.abs = t.next \
             WHERE  e.obs = $1",
                transitions(),
                isomorphism(),
                abstraction()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        let sql = if obs.street() == Street::Rive {
            river
        } else {
            other
        };
        Ok(self
            .0
            .query_one(sql.as_str(), &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation equity: {}", e))?
            .get::<_, f32>(0))
    }
}

// distance calculations
impl TopologyAPI {
    pub async fn abs_distance(
        &self,
        abs1: Abstraction,
        abs2: Abstraction,
    ) -> anyhow::Result<Energy> {
        if abs1.street() != abs2.street() {
            return Err(anyhow::anyhow!("abstractions must be from the same street"));
        }
        if abs1 == abs2 {
            return Ok(0 as Energy);
        }
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT dx FROM {} WHERE tri = $1", metric()));
        let pair = Pair::from((&abs1, &abs2));
        let tri = i32::from(pair);
        self.0
            .query_one(sql.as_str(), &[&tri])
            .await
            .map(|row| row.get::<_, Energy>(0))
            .map_err(|e| anyhow::anyhow!("fetch distance: {}", e))
    }

    pub async fn obs_distance(
        &self,
        obs1: Observation,
        obs2: Observation,
    ) -> anyhow::Result<Energy> {
        if obs1.street() != obs2.street() {
            return Err(anyhow::anyhow!("observations must be from the same street"));
        }
        let (ref hx, ref hy, ref metric) = tokio::try_join!(
            self.obs_histogram(obs1),
            self.obs_histogram(obs2),
            self.metric(obs1.street().next())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }

    pub async fn obs_abs_distance(
        &self,
        obs: Observation,
        abs: Abstraction,
    ) -> anyhow::Result<Energy> {
        if obs.street() != abs.street() {
            return Err(anyhow::anyhow!(
                "observation and abstraction must be from the same street"
            ));
        }
        let (ref hx, ref hy, ref metric) = tokio::try_join!(
            self.obs_histogram(obs),
            self.abs_histogram(abs),
            self.metric(obs.street().next())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }
}

// population lookups
impl TopologyAPI {
    pub async fn abs_population(&self, abs: Abstraction) -> anyhow::Result<usize> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("SELECT population FROM {} WHERE abs = $1", abstraction()));
        let abs = i16::from(abs);
        self.0
            .query_one(sql.as_str(), &[&abs])
            .await
            .map(|row| row.get::<_, i64>(0) as usize)
            .map_err(|e| anyhow::anyhow!("fetch abstraction population: {}", e))
    }

    pub async fn obs_population(&self, obs: Observation) -> anyhow::Result<usize> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT population \
             FROM   {} a \
             JOIN   {} e ON e.abs = a.abs \
             WHERE  e.obs = $1",
                abstraction(),
                isomorphism()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        Ok(self
            .0
            .query_one(sql.as_str(), &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation population: {}", e))?
            .get::<_, i64>(0) as usize)
    }
}

// histogram aggregation
impl TopologyAPI {
    pub async fn abs_histogram(&self, abs: Abstraction) -> anyhow::Result<Histogram> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("SELECT next, dx FROM {} WHERE prev = $1", transitions()));
        let abs_i = i16::from(abs);
        let street = abs.street().next();
        let rows = self
            .0
            .query(sql.as_str(), &[&abs_i])
            .await
            .map_err(|e| anyhow::anyhow!("fetch abstraction histogram: {}", e))?;
        Ok(rows
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (Abstraction::from(next), (dx * 1000.0) as usize))
            .fold(Histogram::empty(street), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            }))
    }

    pub async fn obs_histogram(&self, obs: Observation) -> anyhow::Result<Histogram> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT next, dx \
             FROM   {} t \
             JOIN   {} e ON e.abs = t.prev \
             WHERE  e.obs = $1",
                transitions(),
                isomorphism()
            )
        });
        let idx = i64::from(Isomorphism::from(obs));
        let mass = obs.street().n_children() as f32;
        let street = obs.street().next();
        Ok(self
            .0
            .query(sql.as_str(), &[&idx])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation histogram: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (next, (dx * mass).round() as usize))
            .map(|(next, dx)| (Abstraction::from(next), dx))
            .fold(Histogram::empty(street), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            }))
    }
}

// exploration panel
impl TopologyAPI {
    pub async fn exp_wrt_str(&self, str: Street) -> anyhow::Result<ApiSample> {
        self.exp_wrt_obs(Observation::from(str)).await
    }

    pub async fn exp_wrt_obs(&self, obs: Observation) -> anyhow::Result<ApiSample> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT e.obs, \
             a.abs, \
             a.equity::REAL, \
             a.population::REAL / $2 AS density \
             FROM   {} e \
             JOIN   {} a ON a.abs = e.abs \
             WHERE  e.obs = $1",
                isomorphism(),
                abstraction()
            )
        });
        let n = obs.street().n_observations() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(sql.as_str(), &[&iso, &n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to observation: {}", e))?;
        Ok(api_sample_from_row(row))
    }

    pub async fn exp_wrt_abs(&self, abs: Abstraction) -> anyhow::Result<ApiSample> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH sample AS ( \
             SELECT a.abs, \
             a.population, \
             a.equity, \
             FLOOR(RANDOM() * a.population)::INTEGER AS i \
             FROM   {} a \
             WHERE  a.abs = $1 \
             ) \
             SELECT e.obs, \
             s.abs, \
             s.equity::REAL, \
             s.population::REAL / $2 AS density \
             FROM   sample s \
             JOIN   {} e ON e.abs = s.abs AND e.position = s.i \
             LIMIT  1",
                abstraction(),
                isomorphism()
            )
        });
        let n = abs.street().n_isomorphisms() as f32;
        let abs = i16::from(abs);
        let row = self
            .0
            .query_one(sql.as_str(), &[&abs, &n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to abstraction: {}", e))?;
        Ok(api_sample_from_row(row))
    }
}

// proximity lookups
impl TopologyAPI {
    pub async fn abs_nearby(&self, abs: Abstraction) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT   a.abs, m.dx \
             FROM     {} a \
             JOIN     {} m ON m.tri = get_pair_tri(a.abs, $1) \
             WHERE    a.abs != $1 \
             ORDER BY m.dx ASC \
             LIMIT    $2",
                abstraction(),
                metric()
            )
        });
        let abs = i16::from(abs);
        Ok(self
            .0
            .query(sql.as_str(), &[&abs, &N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }

    pub async fn obs_nearby(&self, obs: Observation) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT   a.abs, m.dx \
             FROM     {} e \
             JOIN     {} a ON a.street = get_street_abs(e.abs) \
             JOIN     {} m ON m.tri = get_pair_tri(a.abs, e.abs) \
             WHERE    e.obs = $1 \
             AND      a.abs != e.abs \
             ORDER BY m.dx ASC \
             LIMIT    $2",
                isomorphism(),
                abstraction(),
                metric()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        Ok(self
            .0
            .query(sql.as_str(), &[&iso, &N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions for observation: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
}

// similarity lookups
impl TopologyAPI {
    pub async fn obs_similar(&self, obs: Observation) -> anyhow::Result<Vec<Observation>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH target AS ( \
             SELECT abs, population \
             FROM   {} e \
             JOIN   {} a ON a.abs = e.abs \
             WHERE  e.obs = $1 \
             ) \
             SELECT   e.obs \
             FROM     {} e \
             JOIN     target t ON t.abs = e.abs \
             WHERE    e.obs != $1 \
             AND      e.position  < LEAST($2, t.population) \
             AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) \
             LIMIT    $2",
                isomorphism(),
                abstraction(),
                isomorphism()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        Ok(self
            .0
            .query(sql.as_str(), &[&iso, &N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch similar observations: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }

    pub async fn abs_similar(&self, abs: Abstraction) -> anyhow::Result<Vec<Observation>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH target AS ( \
             SELECT population \
             FROM   {} \
             WHERE  abs = $1 \
             ) \
             SELECT   obs \
             FROM     {} e, target t \
             WHERE    e.abs = $1 \
             AND      e.position  < LEAST($2, t.population) \
             AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) \
             LIMIT    $2",
                abstraction(),
                isomorphism()
            )
        });
        let abs = i16::from(abs);
        Ok(self
            .0
            .query(sql.as_str(), &[&abs, &N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observations similar to abstraction: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }

    pub async fn replace_obs(&self, obs: Observation) -> anyhow::Result<Observation> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH sample AS ( \
             SELECT e.abs, \
             a.population, \
             FLOOR(RANDOM() * a.population)::INTEGER AS i \
             FROM   {} e \
             JOIN   {} a ON a.abs = e.abs \
             WHERE  e.obs = $1 \
             ) \
             SELECT e.obs \
             FROM   sample s \
             JOIN   {} e ON e.abs = s.abs AND e.position = s.i \
             LIMIT  1",
                isomorphism(),
                abstraction(),
                isomorphism()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(sql.as_str(), &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("replace observation: {}", e))?;
        Ok(Observation::from(row.get::<_, i64>(0)))
    }
}

// neighborhood lookups
impl TopologyAPI {
    pub async fn nbr_any_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<ApiSample> {
        use rand::prelude::IndexedRandom;
        let rng = &mut rand::rng();
        let abs = Abstraction::all(wrt.street())
            .into_iter()
            .filter(|&x| x != wrt)
            .collect::<Vec<_>>()
            .choose(rng)
            .copied()
            .expect("more than one abstraction option");
        self.nbr_abs_wrt_abs(wrt, abs).await
    }

    pub async fn nbr_abs_wrt_abs(
        &self,
        wrt: Abstraction,
        abs: Abstraction,
    ) -> anyhow::Result<ApiSample> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH sample AS ( \
             SELECT r.abs, \
             r.population, \
             r.equity, \
             FLOOR(RANDOM() * r.population)::INTEGER AS i, \
             COALESCE(m.dx, 0) AS distance \
             FROM      {} r \
             LEFT JOIN {} m ON m.tri = get_pair_tri($1, $3) \
             WHERE     r.abs = $1 \
             ), \
             random_iso AS ( \
             SELECT e.obs, \
             e.abs, \
             s.equity, \
             s.population, \
             s.distance \
             FROM   sample s \
             JOIN   {} e ON e.abs = s.abs AND e.position = s.i \
             LIMIT  1 \
             ) \
             SELECT obs, \
             abs, \
             equity::REAL, \
             population::REAL / $2 AS density, \
             distance::REAL \
             FROM   random_iso",
                abstraction(),
                metric(),
                isomorphism()
            )
        });
        let n = wrt.street().n_isomorphisms() as f32;
        let abs = i16::from(abs);
        let wrt = i16::from(wrt);
        let row = self
            .0
            .query_one(sql.as_str(), &[&abs, &n, &wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor abstraction: {}", e))?;
        Ok(api_sample_from_row(row))
    }

    pub async fn nbr_obs_wrt_abs(
        &self,
        wrt: Abstraction,
        obs: Observation,
    ) -> anyhow::Result<ApiSample> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH given AS ( \
             SELECT obs, abs, get_pair_tri(abs, $3) AS tri \
             FROM   {} \
             WHERE  obs = $1 \
             ) \
             SELECT g.obs, \
             g.abs, \
             a.equity::REAL, \
             a.population::REAL / $2 AS density, \
             COALESCE(m.dx, 0)::REAL AS distance \
             FROM   given g \
             JOIN   {} m ON m.tri = g.tri \
             JOIN   {} a ON a.abs = g.abs \
             LIMIT  1",
                isomorphism(),
                metric(),
                abstraction()
            )
        });
        let n = wrt.street().n_isomorphisms() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let wrt = i16::from(wrt);
        let row = self
            .0
            .query_one(sql.as_str(), &[&iso, &n, &wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor observation: {}", e))?;
        Ok(api_sample_from_row(row))
    }
}

// k-nearest neighbors lookups
impl TopologyAPI {
    pub async fn kfn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH nearest AS ( \
             SELECT   a.abs, \
             a.population, \
             m.dx AS distance, \
             FLOOR(RANDOM() * a.population)::INTEGER AS sample \
             FROM     {} a \
             JOIN     {} m ON m.tri = get_pair_tri(a.abs, $1) \
             WHERE    a.street = $2 \
             AND      a.abs != $1 \
             ORDER BY m.dx DESC \
             LIMIT    $3 \
             ) \
             SELECT   e.obs, \
             n.abs, \
             a.equity::REAL, \
             a.population::REAL / $4 AS density, \
             n.distance::REAL \
             FROM     nearest n \
             JOIN     {} a ON a.abs = n.abs \
             JOIN     {} e ON e.abs = n.abs AND e.position = n.sample \
             ORDER BY n.distance DESC",
                abstraction(),
                metric(),
                abstraction(),
                isomorphism()
            )
        });
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql.as_str(), &[&wrt, &s, &N_NEIGHBORS, &n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-farthest neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }

    pub async fn knn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH nearest AS ( \
             SELECT   a.abs, \
             a.population, \
             m.dx AS distance, \
             FLOOR(RANDOM() * a.population)::INTEGER AS sample \
             FROM     {} a \
             JOIN     {} m ON m.tri = get_pair_tri(a.abs, $1) \
             WHERE    a.street = $2 \
             AND      a.abs != $1 \
             ORDER BY m.dx ASC \
             LIMIT    $3 \
             ) \
             SELECT   e.obs, \
             n.abs, \
             a.equity::REAL, \
             a.population::REAL / $4 AS density, \
             n.distance::REAL \
             FROM     nearest n \
             JOIN     {} a ON a.abs = n.abs \
             JOIN     {} e ON e.abs = n.abs AND e.position = n.sample \
             ORDER BY n.distance ASC",
                abstraction(),
                metric(),
                abstraction(),
                isomorphism()
            )
        });
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql.as_str(), &[&wrt, &s, &N_NEIGHBORS, &n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-nearest neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }

    pub async fn kgn_wrt_abs(
        &self,
        wrt: Abstraction,
        nbr: Vec<Observation>,
    ) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH input(obs, ord) AS ( \
             SELECT unnest($3::BIGINT[]), \
             generate_series(1, array_length($3, 1)) \
             ) \
             SELECT   e.obs, \
             e.abs, \
             a.equity::REAL, \
             a.population::REAL / $1 AS density, \
             m.dx::REAL AS distance \
             FROM     input i \
             JOIN     {} e ON e.obs = i.obs \
             JOIN     {} a ON a.abs = e.abs \
             JOIN     {} m ON m.tri = get_pair_tri(a.abs, $2) \
             ORDER BY i.ord \
             LIMIT    $4",
                isomorphism(),
                abstraction(),
                metric()
            )
        });
        let isos = nbr
            .into_iter()
            .map(Isomorphism::from)
            .map(i64::from)
            .collect::<Vec<_>>();
        let n = wrt.street().n_isomorphisms() as f32;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql.as_str(), &[&n, &wrt, &&isos, &N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch given neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
}

// histogram lookups
impl TopologyAPI {
    pub async fn hst_wrt_obs(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        if obs.street() == Street::Rive {
            self.hst_wrt_obs_on_river(obs).await
        } else {
            self.hst_wrt_obs_on_other(obs).await
        }
    }

    pub async fn hst_wrt_abs(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        if abs.street() == Street::Rive {
            self.hst_wrt_abs_on_river(abs).await
        } else {
            self.hst_wrt_abs_on_other(abs).await
        }
    }

    async fn hst_wrt_obs_on_river(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH sample AS ( \
             SELECT e.obs, \
             e.abs, \
             a.equity, \
             a.population, \
             FLOOR(RANDOM() * a.population)::INTEGER AS position \
             FROM   {} e \
             JOIN   {} a ON a.abs = e.abs \
             WHERE  e.abs = (SELECT abs FROM {} WHERE obs = $1) \
             LIMIT  1 \
             ) \
             SELECT s.obs, \
             s.abs, \
             s.equity::REAL, \
             1::REAL AS density \
             FROM   sample s",
                isomorphism(),
                abstraction(),
                isomorphism()
            )
        });
        let iso = i64::from(Isomorphism::from(obs));
        let rows = self
            .0
            .query(sql.as_str(), &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river observation distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }

    async fn hst_wrt_obs_on_other(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT e.obs, \
             e.abs, \
             a.equity \
             FROM   {} e \
             JOIN   {} a ON a.abs = e.abs \
             WHERE  e.obs = ANY($1)",
                isomorphism(),
                abstraction()
            )
        });
        let n = obs.street().n_children();
        let children = obs
            .children()
            .map(Isomorphism::from)
            .map(Observation::from)
            .collect::<Vec<_>>();
        let distinct = children
            .iter()
            .copied()
            .map(i64::from)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let rows = self
            .0
            .query(sql.as_str(), &[&distinct])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation distribution: {}", e))?
            .into_iter()
            .map(|row| {
                (
                    Observation::from(row.get::<_, i64>(0)),
                    Abstraction::from(row.get::<_, i16>(1)),
                    Probability::from(row.get::<_, f32>(2)),
                )
            })
            .map(|(obs, abs, equity)| (obs, (abs, equity)))
            .collect::<std::collections::BTreeMap<_, _>>();
        let hist = children
            .iter()
            .map(|child| rows.get(child).map(|row| (*child, row)))
            .map(|x| x.ok_or_else(|| anyhow::anyhow!("observation not found in database")))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .fold(
                std::collections::BTreeMap::<_, _>::new(),
                |mut btree, (obs, (abs, eqy))| {
                    btree.entry(abs).or_insert((obs, *eqy, 0)).2 += 1;
                    btree
                },
            )
            .into_iter()
            .map(|(abs, (obs, eqy, pop))| ApiSample {
                obs,
                abs: *abs,
                equity: eqy,
                density: pop as Probability / n as Probability,
                distance: 0.,
            })
            .collect::<Vec<_>>();
        Ok(hist)
    }

    async fn hst_wrt_abs_on_river(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH sample AS ( \
             SELECT a.abs, \
             a.population, \
             a.equity, \
             FLOOR(RANDOM() * a.population)::INTEGER AS position \
             FROM   {} a \
             WHERE  a.abs = $1 \
             LIMIT  1 \
             ) \
             SELECT e.obs, \
             e.abs, \
             s.equity::REAL, \
             1::REAL AS density \
             FROM   sample s \
             JOIN   {} e ON e.abs = s.abs AND e.position = s.position",
                abstraction(),
                isomorphism()
            )
        });
        let abs = &i16::from(abs);
        let rows = self
            .0
            .query(sql.as_str(), &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }

    async fn hst_wrt_abs_on_other(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH histogram AS ( \
             SELECT p.abs, \
             g.dx AS probability, \
             p.population, \
             p.equity, \
             FLOOR(RANDOM() * p.population)::INTEGER AS i \
             FROM   {} g \
             JOIN   {} p ON p.abs = g.next \
             WHERE  g.prev = $1 \
             ) \
             SELECT   e.obs, \
             t.abs, \
             t.equity::REAL, \
             t.probability AS density \
             FROM     histogram t \
             JOIN     {} e ON e.abs = t.abs AND e.position = t.i \
             ORDER BY t.probability DESC",
                transitions(),
                abstraction(),
                isomorphism()
            )
        });
        let abs = &i16::from(abs);
        let rows = self
            .0
            .query(sql.as_str(), &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
}
