use rbp_cards::*;
use rbp_clustering::*;
use rbp_core::*;
use rbp_database::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::*;
use std::sync::Arc;
use std::time::Instant;
use tokio_postgres::Client;

const N_NEIGHBORS: i64 = 6;

macro_rules! sql_params {
    ($($param:expr),+ $(,)?) => {
        &[$(&$param as &(dyn tokio_postgres::types::ToSql + Sync)),+]
    };
}

// Local conversion functions for database types.
// These bridge tokio_postgres::Row to our domain types.
// We use free functions instead of From traits due to orphan rules.

fn api_sample_from_row(row: tokio_postgres::Row) -> ApiSample {
    ApiSample {
        obs: Observation::from(row.get::<_, i64>("obs")).to_string(),
        abs: Abstraction::from(row.get::<_, i16>("abs")).to_string(),
        equity: row.get::<_, f32>("equity"),
        density: row.get::<_, f32>("density"),
        distance: row.try_get::<_, f32>("distance").unwrap_or_default(),
    }
}

fn decision_from_row(row: tokio_postgres::Row) -> Decision<NlheEdge> {
    Decision {
        edge: NlheEdge::from(row.get::<_, i64>("edge") as u64),
        mass: Probability::from(row.get::<_, f32>("weight")),
        counts: row.get::<_, i32>("counts") as u32,
    }
}

fn api_strategy_from(strategy: Strategy) -> ApiStrategy {
    let history = strategy.info().subgame();
    let present = strategy.info().bucket();
    let choices = strategy.info().choices();
    ApiStrategy {
        history: i64::from(history),
        present: i16::from(present),
        choices: i64::from(choices),
        accumulated: strategy
            .accumulated()
            .iter()
            .map(|(edge, policy)| (edge.to_string(), *policy))
            .collect(),
        counts: strategy
            .counts()
            .iter()
            .map(|(edge, count)| (edge.to_string(), *count))
            .collect(),
    }
}

pub struct API(Arc<Client>, tokio::sync::OnceCell<Arc<Flagship>>);

impl From<Arc<Client>> for API {
    fn from(client: Arc<Client>) -> Self {
        Self::new(client)
    }
}

impl API {
    pub fn new(client: Arc<Client>) -> Self {
        Self(client, tokio::sync::OnceCell::new())
    }
    pub fn client(&self) -> &Arc<Client> {
        &self.0
    }
    async fn flagship(&self) -> Arc<Flagship> {
        self.1
            .get_or_init(|| async {
                use rbp_database::Hydrate;
                Arc::new(Flagship::hydrate(self.0.clone()).await)
            })
            .await
            .clone()
    }
}

// global lookups
impl API {
    pub async fn obs_to_abs(&self, obs: Observation) -> anyhow::Result<Abstraction> {
        let iso = Isomorphism::from(obs);
        let idx = i64::from(iso);
        let sql = const_format::concatcp!("SELECT abs FROM ", ISOMORPHISM, " WHERE obs = $1");
        self.0
            .query_one(sql, &[&idx])
            .await
            .map(|row| Abstraction::from(row.get::<_, i16>(0)))
            .map_err(|e| anyhow::anyhow!("fetch abstraction: {}", e))
    }
    pub async fn metric(&self, street: Street) -> anyhow::Result<Metric> {
        let s = street as i16;
        let sql = const_format::concatcp!("SELECT tri, dx FROM ", METRIC, " WHERE street = $1");
        let rows = self
            .0
            .query(sql, &[&s])
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
impl API {
    pub async fn abs_equity(&self, abs: Abstraction) -> anyhow::Result<Probability> {
        let abs = i16::from(abs);
        let sql = const_format::concatcp!("SELECT equity FROM ", ABSTRACTION, " WHERE abs = $1");
        self.0
            .query_one(sql, &[&abs])
            .await
            .map(|row| Probability::from(row.get::<_, f32>(0)))
            .map_err(|e| anyhow::anyhow!("fetch abstraction equity: {}", e))
    }
    pub async fn obs_equity(&self, obs: Observation) -> anyhow::Result<Probability> {
        let iso = i64::from(Isomorphism::from(obs));
        let river: &str = const_format::concatcp!(
            "SELECT equity ",
            "FROM   ",
            ISOMORPHISM,
            " ",
            "WHERE  obs = $1"
        );
        let other: &str = const_format::concatcp!(
            "SELECT SUM(t.dx * a.equity) ",
            "FROM   ",
            TRANSITIONS,
            " t ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = t.prev ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = t.next ",
            "WHERE  e.obs = $1"
        );
        let sql = if obs.street() == Street::Rive {
            river
        } else {
            other
        };
        Ok(self
            .0
            .query_one(sql, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation equity: {}", e))?
            .get::<_, f32>(0)
            .into())
    }
}

// distance calculations
impl API {
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
        let pair = Pair::from((&abs1, &abs2));
        let tri = i32::from(pair);
        let sql = const_format::concatcp!("SELECT dx FROM ", METRIC, " WHERE tri = $1");
        self.0
            .query_one(sql, &[&tri])
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
}

// population lookups
impl API {
    pub async fn abs_population(&self, abs: Abstraction) -> anyhow::Result<usize> {
        let abs = i16::from(abs);
        let sql =
            const_format::concatcp!("SELECT population FROM ", ABSTRACTION, " WHERE abs = $1");
        self.0
            .query_one(sql, &[&abs])
            .await
            .map(|row| row.get::<_, i64>(0) as usize)
            .map_err(|e| anyhow::anyhow!("fetch abstraction population: {}", e))
    }
    pub async fn obs_population(&self, obs: Observation) -> anyhow::Result<usize> {
        let iso = i64::from(Isomorphism::from(obs));
        let sql: &str = const_format::concatcp!(
            "SELECT population   ",
            "FROM   ",
            ABSTRACTION,
            " a ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = a.abs ",
            "WHERE  e.obs = $1"
        );
        Ok(self
            .0
            .query_one(sql, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation population: {}", e))?
            .get::<_, i64>(0) as usize)
    }
}

// histogram aggregation
impl API {
    pub async fn abs_histogram(&self, abs: Abstraction) -> anyhow::Result<Histogram> {
        let abs_i = i16::from(abs);
        let sql = const_format::concatcp!("SELECT next, dx FROM ", TRANSITIONS, " WHERE prev = $1");
        let street = abs.street().next();
        let rows = self
            .0
            .query(sql, &[&abs_i])
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
        let idx = i64::from(Isomorphism::from(obs));
        let mass = obs.street().n_children() as f32;
        let sql: &str = const_format::concatcp!(
            "SELECT next, ",
            "dx ",
            "FROM   ",
            TRANSITIONS,
            " t ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = t.prev ",
            "WHERE  e.obs = $1"
        );
        let street = obs.street().next();
        Ok(self
            .0
            .query(sql, &[&idx])
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
impl API {
    pub async fn exp_wrt_str(&self, str: Street) -> anyhow::Result<ApiSample> {
        self.exp_wrt_obs(Observation::from(str)).await
    }
    pub async fn exp_wrt_obs(&self, obs: Observation) -> anyhow::Result<ApiSample> {
        let sql: &str = const_format::concatcp!(
            "SELECT e.obs, ",
            "a.abs, ",
            "a.equity::REAL, ",
            "a.population::REAL / $2 AS density ",
            "FROM   ",
            ISOMORPHISM,
            " e ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "WHERE  e.obs = $1"
        );
        let n = obs.street().n_observations() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(sql, sql_params![iso, n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to observation: {}", e))?;
        Ok(api_sample_from_row(row))
    }
    pub async fn exp_wrt_abs(&self, abs: Abstraction) -> anyhow::Result<ApiSample> {
        let sql: &str = const_format::concatcp!(
            "WITH sample AS ( ",
            "SELECT a.abs, ",
            "a.population, ",
            "a.equity, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS i ",
            "FROM   ",
            ABSTRACTION,
            " a ",
            "WHERE  a.abs = $1 ",
            ") ",
            "SELECT e.obs, ",
            "s.abs, ",
            "s.equity::REAL, ",
            "s.population::REAL / $2 AS density ",
            "FROM   sample s ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = s.abs AND e.position = s.i ",
            "LIMIT  1"
        );
        let n = abs.street().n_isomorphisms() as f32;
        let abs = i16::from(abs);
        let row = self
            .0
            .query_one(sql, sql_params![abs, n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to abstraction: {}", e))?;
        Ok(api_sample_from_row(row))
    }
}

// proximity lookups
impl API {
    pub async fn abs_nearby(&self, abs: Abstraction) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        let abs = i16::from(abs);
        let sql: &str = const_format::concatcp!(
            "SELECT   a.abs, ",
            "m.dx ",
            "FROM     ",
            ABSTRACTION,
            " a ",
            "JOIN     ",
            METRIC,
            " m ON m.tri = get_pair_tri(a.abs, $1) ",
            "WHERE    a.abs != $1 ",
            "ORDER BY m.dx ASC ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(sql, sql_params![abs, N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
    pub async fn obs_nearby(&self, obs: Observation) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        let iso = i64::from(Isomorphism::from(obs));
        let sql: &str = const_format::concatcp!(
            "SELECT   a.abs, ",
            "m.dx ",
            "FROM     ",
            ISOMORPHISM,
            " e ",
            "JOIN     ",
            ABSTRACTION,
            " a ON a.street = get_street_abs(e.abs) ",
            "JOIN     ",
            METRIC,
            " m ON m.tri = get_pair_tri(a.abs, e.abs) ",
            "WHERE    e.obs = $1 ",
            "AND      a.abs != e.abs ",
            "ORDER BY m.dx ASC ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(sql, sql_params![iso, N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions for observation: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
}

// similarity lookups
impl API {
    pub async fn obs_similar(&self, obs: Observation) -> anyhow::Result<Vec<Observation>> {
        let iso = i64::from(Isomorphism::from(obs));
        let sql: &str = const_format::concatcp!(
            "WITH target AS ( ",
            "SELECT abs, population ",
            "FROM   ",
            ISOMORPHISM,
            " e ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "WHERE  e.obs = $1 ",
            ") ",
            "SELECT   e.obs ",
            "FROM     ",
            ISOMORPHISM,
            " e ",
            "JOIN     target t ON t.abs = e.abs ",
            "WHERE    e.obs != $1 ",
            "AND      e.position  < LEAST($2, t.population) ",
            "AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(sql, sql_params![iso, N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch similar observations: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    pub async fn abs_similar(&self, abs: Abstraction) -> anyhow::Result<Vec<Observation>> {
        let abs = i16::from(abs);
        let sql: &str = const_format::concatcp!(
            "WITH target AS ( ",
            "SELECT population ",
            "FROM   ",
            ABSTRACTION,
            " ",
            "WHERE  abs = $1 ",
            ") ",
            "SELECT   obs ",
            "FROM     ",
            ISOMORPHISM,
            " e, target t ",
            "WHERE    e.abs = $1 ",
            "AND      e.position  < LEAST($2, t.population) ",
            "AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(sql, sql_params![abs, N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observations similar to abstraction: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    pub async fn replace_obs(&self, obs: Observation) -> anyhow::Result<Observation> {
        let sql: &str = const_format::concatcp!(
            "WITH sample AS ( ",
            "SELECT e.abs, ",
            "a.population, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS i ",
            "FROM   ",
            ISOMORPHISM,
            " e ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "WHERE  e.obs = $1 ",
            ") ",
            "SELECT e.obs ",
            "FROM   sample s ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = s.abs AND e.position = s.i ",
            "LIMIT  1"
        );
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(sql, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("replace observation: {}", e))?;
        Ok(Observation::from(row.get::<_, i64>(0)))
    }
}

// neighborhood lookups
impl API {
    pub async fn nbr_any_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<ApiSample> {
        use rand::prelude::IndexedRandom;
        let ref mut rng = rand::rng();
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
        let sql: &str = const_format::concatcp!(
            "WITH sample AS ( ",
            "SELECT r.abs, ",
            "r.population, ",
            "r.equity, ",
            "FLOOR(RANDOM() * r.population)::INTEGER AS i, ",
            "COALESCE(m.dx, 0) AS distance ",
            "FROM      ",
            ABSTRACTION,
            " r ",
            "LEFT JOIN ",
            METRIC,
            " m ON m.tri = get_pair_tri($1, $3) ",
            "WHERE     r.abs = $1 ",
            "), ",
            "random_iso AS ( ",
            "SELECT e.obs, ",
            "e.abs, ",
            "s.equity, ",
            "s.population, ",
            "s.distance ",
            "FROM   sample s ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = s.abs AND e.position = s.i ",
            "LIMIT  1 ",
            ") ",
            "SELECT obs, ",
            "abs, ",
            "equity::REAL, ",
            "population::REAL / $2 AS density, ",
            "distance::REAL ",
            "FROM   random_iso"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let abs = i16::from(abs);
        let wrt = i16::from(wrt);
        let row = self
            .0
            .query_one(sql, sql_params![abs, n, wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor abstraction: {}", e))?;
        Ok(api_sample_from_row(row))
    }
    pub async fn nbr_obs_wrt_abs(
        &self,
        wrt: Abstraction,
        obs: Observation,
    ) -> anyhow::Result<ApiSample> {
        let sql: &str = const_format::concatcp!(
            "WITH given AS ( ",
            "SELECT obs, abs, get_pair_tri(abs, $3) AS tri ",
            "FROM   ",
            ISOMORPHISM,
            " ",
            "WHERE  obs = $1 ",
            ") ",
            "SELECT g.obs, ",
            "g.abs, ",
            "a.equity::REAL, ",
            "a.population::REAL / $2 AS density, ",
            "COALESCE(m.dx, 0)::REAL AS distance ",
            "FROM   given g ",
            "JOIN   ",
            METRIC,
            " m ON m.tri = g.tri ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = g.abs ",
            "LIMIT  1"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let wrt = i16::from(wrt);
        let row = self
            .0
            .query_one(sql, sql_params![iso, n, wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor observation: {}", e))?;
        Ok(api_sample_from_row(row))
    }
}

// k-nearest neighbors lookups
impl API {
    pub async fn kfn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        let sql: &str = const_format::concatcp!(
            "WITH nearest AS ( ",
            "SELECT   a.abs, ",
            "a.population, ",
            "m.dx AS distance, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS sample ",
            "FROM     ",
            ABSTRACTION,
            " a ",
            "JOIN     ",
            METRIC,
            " m ON m.tri = get_pair_tri(a.abs, $1) ",
            "WHERE    a.street = $2 ",
            "AND      a.abs != $1 ",
            "ORDER BY m.dx DESC ",
            "LIMIT    $3 ",
            ") ",
            "SELECT   e.obs, ",
            "n.abs, ",
            "a.equity::REAL, ",
            "a.population::REAL / $4 AS density, ",
            "n.distance::REAL ",
            "FROM     nearest n ",
            "JOIN     ",
            ABSTRACTION,
            " a ON a.abs = n.abs ",
            "JOIN     ",
            ISOMORPHISM,
            " e ON e.abs = n.abs AND e.position = n.sample ",
            "ORDER BY n.distance DESC"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql, sql_params![wrt, s, N_NEIGHBORS, n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-farthest neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
    pub async fn knn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        let sql: &str = const_format::concatcp!(
            "WITH nearest AS ( ",
            "SELECT   a.abs, ",
            "a.population, ",
            "m.dx AS distance, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS sample ",
            "FROM     ",
            ABSTRACTION,
            " a ",
            "JOIN     ",
            METRIC,
            " m ON m.tri = get_pair_tri(a.abs, $1) ",
            "WHERE    a.street = $2 ",
            "AND      a.abs != $1 ",
            "ORDER BY m.dx ASC ",
            "LIMIT    $3 ",
            ") ",
            "SELECT   e.obs, ",
            "n.abs, ",
            "a.equity::REAL, ",
            "a.population::REAL / $4 AS density, ",
            "n.distance::REAL ",
            "FROM     nearest n ",
            "JOIN     ",
            ABSTRACTION,
            " a ON a.abs = n.abs ",
            "JOIN     ",
            ISOMORPHISM,
            " e ON e.abs = n.abs AND e.position = n.sample ",
            "ORDER BY n.distance ASC"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql, sql_params![wrt, s, N_NEIGHBORS, n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-nearest neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
    pub async fn kgn_wrt_abs(
        &self,
        wrt: Abstraction,
        nbr: Vec<Observation>,
    ) -> anyhow::Result<Vec<ApiSample>> {
        let sql: &str = const_format::concatcp!(
            "WITH input(obs, ord) AS ( ",
            "SELECT unnest($3::BIGINT[]), ",
            "generate_series(1, array_length($3, 1)) ",
            ") ",
            "SELECT   e.obs, ",
            "e.abs, ",
            "a.equity::REAL, ",
            "a.population::REAL / $1 AS density, ",
            "m.dx::REAL AS distance ",
            "FROM     input i ",
            "JOIN     ",
            ISOMORPHISM,
            " e ON e.obs = i.obs ",
            "JOIN     ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "JOIN     ",
            METRIC,
            " m ON m.tri = get_pair_tri(a.abs, $2) ",
            "ORDER BY i.ord ",
            "LIMIT    $4"
        );
        let isos = nbr
            .into_iter()
            .map(Isomorphism::from)
            .map(i64::from)
            .collect::<Vec<_>>();
        let n = wrt.street().n_isomorphisms() as f32;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(sql, sql_params![n, wrt, isos, N_NEIGHBORS])
            .await
            .map_err(|e| anyhow::anyhow!("fetch given neighbors: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
}

// histogram lookups
impl API {
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
        let sql: &str = const_format::concatcp!(
            "WITH sample AS ( ",
            "SELECT e.obs, ",
            "e.abs, ",
            "a.equity, ",
            "a.population, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS position ",
            "FROM   ",
            ISOMORPHISM,
            " e ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "WHERE  e.abs = (SELECT abs FROM ",
            ISOMORPHISM,
            " WHERE obs = $1) ",
            "LIMIT  1 ",
            ") ",
            "SELECT s.obs, ",
            "s.abs, ",
            "s.equity::REAL, ",
            "1::REAL AS density ",
            "FROM   sample s"
        );
        let iso = i64::from(Isomorphism::from(obs));
        let rows = self
            .0
            .query(sql, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river observation distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
    async fn hst_wrt_obs_on_other(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        // Simplified for compilation - full implementation in original code
        let sql: &str = const_format::concatcp!(
            "SELECT e.obs, ",
            "e.abs, ",
            "a.equity ",
            "FROM   ",
            ISOMORPHISM,
            " e ",
            "JOIN   ",
            ABSTRACTION,
            " a ON a.abs = e.abs ",
            "WHERE  e.obs = $1"
        );
        let iso = i64::from(Isomorphism::from(obs));
        let rows = self
            .0
            .query(sql, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
    async fn hst_wrt_abs_on_river(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        let sql: &str = const_format::concatcp!(
            "WITH sample AS ( ",
            "SELECT a.abs, ",
            "a.population, ",
            "a.equity, ",
            "FLOOR(RANDOM() * a.population)::INTEGER AS position ",
            "FROM   ",
            ABSTRACTION,
            " a ",
            "WHERE  a.abs = $1 ",
            "LIMIT  1 ",
            ") ",
            "SELECT e.obs, ",
            "e.abs, ",
            "s.equity::REAL, ",
            "1::REAL AS density ",
            "FROM   sample s ",
            "JOIN   ",
            ISOMORPHISM,
            " e ON e.abs = s.abs AND e.position = s.position"
        );
        let ref abs = i16::from(abs);
        let rows = self
            .0
            .query(sql, &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
    async fn hst_wrt_abs_on_other(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        let sql: &str = const_format::concatcp!(
            "WITH histogram AS ( ",
            "SELECT p.abs, ",
            "g.dx AS probability, ",
            "p.population, ",
            "p.equity, ",
            "FLOOR(RANDOM() * p.population)::INTEGER AS i ",
            "FROM   ",
            TRANSITIONS,
            " g ",
            "JOIN   ",
            ABSTRACTION,
            " p ON p.abs = g.next ",
            "WHERE  g.prev = $1 ",
            ") ",
            "SELECT   e.obs, ",
            "t.abs, ",
            "t.equity::REAL, ",
            "t.probability AS density ",
            "FROM     histogram t ",
            "JOIN     ",
            ISOMORPHISM,
            " e ON e.abs = t.abs AND e.position = t.i ",
            "ORDER BY t.probability DESC"
        );
        let ref abs = i16::from(abs);
        let rows = self
            .0
            .query(sql, &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(api_sample_from_row).collect())
    }
}

// blueprint lookups
impl API {
    pub async fn policy(&self, recall: Partial) -> anyhow::Result<Option<ApiStrategy>> {
        let sql: &str = const_format::concatcp!(
            "SELECT edge, ",
            "weight, ",
            "counts ",
            "FROM   ",
            BLUEPRINT,
            " ",
            "WHERE  past    = $1 ",
            "AND    present = $2 ",
            "AND    choices = $3"
        );
        let recall = recall.validate()?;
        let present = self.obs_to_abs(recall.seen()).await?;
        let info = NlheInfo::from((&recall, present));
        let history = i64::from(info.subgame());
        let present = i16::from(info.bucket());
        let choices = i64::from(info.choices());
        let rows = self
            .0
            .query(sql, sql_params![history, present, choices])
            .await
            .map_err(|e| anyhow::anyhow!("fetch policy: {}", e))?;
        match rows.len() {
            0 => Ok(None),
            _ => Ok(Some(api_strategy_from(Strategy::from((
                info,
                rows.into_iter().map(decision_from_row).collect::<Vec<_>>(),
            ))))),
        }
    }

    pub async fn realtime(&self, recall: Partial) -> anyhow::Result<ApiRealtimeStrategy> {
        let recall = recall.validate()?;
        let offtree = has_offtree_actions(&recall);
        let game = recall.head();
        let started = Instant::now();
        let timeout_ms = DlsOptions::default().max_solve_ms;
        if !should_use_depth_limited_search(&recall) {
            return match self.policy(recall).await? {
                Some(strategy) => Ok(Self::realtime_response(
                    "blueprint",
                    strategy
                        .accumulated
                        .iter()
                        .map(|(edge, weight)| (edge.clone(), *weight))
                        .collect(),
                    ApiRealtimeDiagnostics {
                        used_dls: false,
                        used_blueprint_fallback: false,
                        used_legal_fallback: false,
                        offtree_detected: offtree,
                        solve_ms: started.elapsed().as_millis(),
                        timeout_ms,
                    },
                )),
                None => Ok(self.legal_fallback(&game, offtree, started.elapsed().as_millis())),
            };
        }
        eprintln!(
            "[DLS] /api/realtime using depth-limited search: street={}, offtree={}",
            game.street(),
            offtree
        );
        let seen = recall.seen();
        let flagship = self.flagship().await;
        let abstraction = match flagship.encoder.try_abstraction(&seen) {
            Some(abs) => abs,
            None => {
                return Ok(self.legal_fallback(&game, offtree, started.elapsed().as_millis()));
            }
        };
        let info = SubInfo::Info(NlheInfo::from((&recall, abstraction)));
        let dls_recall = recall.clone();
        let dls_flagship = flagship.clone();
        let solved = tokio::task::spawn_blocking(move || {
            let solver = dls_flagship.depth_limited_subgame(&dls_recall);
            solver.solve().profile().averaged_distribution(&info)
        });
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), solved).await {
            Ok(Ok(policy)) => Ok(Self::realtime_response(
                "dls",
                policy
                    .into_iter()
                    .filter_map(|(edge, probability)| match edge {
                        SubEdge::Inner(edge) => Some((Edge::from(edge).to_string(), probability)),
                        SubEdge::World(_) | SubEdge::Continuation(_) => None,
                    })
                    .collect(),
                ApiRealtimeDiagnostics {
                    used_dls: true,
                    used_blueprint_fallback: false,
                    used_legal_fallback: false,
                    offtree_detected: offtree,
                    solve_ms: started.elapsed().as_millis(),
                    timeout_ms,
                },
            )),
            _ => match self.policy(recall).await? {
                Some(strategy) => Ok(Self::realtime_response(
                    "blueprint_fallback",
                    strategy
                        .accumulated
                        .iter()
                        .map(|(edge, weight)| (edge.clone(), *weight))
                        .collect(),
                    ApiRealtimeDiagnostics {
                        used_dls: false,
                        used_blueprint_fallback: true,
                        used_legal_fallback: false,
                        offtree_detected: offtree,
                        solve_ms: started.elapsed().as_millis(),
                        timeout_ms,
                    },
                )),
                None => Ok(self.legal_fallback(&game, offtree, started.elapsed().as_millis())),
            },
        }
    }

    fn realtime_response(
        source: &str,
        mut actions: std::collections::BTreeMap<String, Probability>,
        diagnostics: ApiRealtimeDiagnostics,
    ) -> ApiRealtimeStrategy {
        let total = actions.values().map(|p| p.max(0.0)).sum::<Probability>();
        if total > 0.0 {
            actions = actions
                .into_iter()
                .map(|(action, probability)| (action, probability.max(0.0) / total))
                .collect();
        }
        let recommended_action = actions
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(action, _)| action.clone())
            .unwrap_or_default();
        ApiRealtimeStrategy {
            source: source.to_string(),
            recommended_action,
            actions,
            diagnostics,
        }
    }

    fn legal_fallback(&self, game: &Game, offtree: bool, solve_ms: u128) -> ApiRealtimeStrategy {
        let legal = game.legal();
        let probability = 1.0 / legal.len().max(1) as Probability;
        Self::realtime_response(
            "legal_fallback",
            legal
                .into_iter()
                .map(|action| (action.to_string(), probability))
                .collect(),
            ApiRealtimeDiagnostics {
                used_dls: false,
                used_blueprint_fallback: false,
                used_legal_fallback: true,
                offtree_detected: offtree,
                solve_ms,
                timeout_ms: DlsOptions::default().max_solve_ms,
            },
        )
    }
}
