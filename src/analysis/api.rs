use crate::cards::*;
use crate::clustering::*;
use crate::database::*;
use crate::dto::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::save::*;
use crate::transport::*;
use crate::*;
use std::sync::Arc;
use tokio_postgres::Client;

const N_SAMPLES: i64 = 5;

pub struct API(Arc<Client>);

impl From<Arc<Client>> for API {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

// constructor
impl API {
    pub async fn new() -> Self {
        Self(crate::save::db().await)
    }
}

// global lookups
impl API {
    pub async fn obs_to_abs(&self, obs: Observation) -> anyhow::Result<Abstraction> {
        Ok(self.0.encode(Isomorphism::from(obs)).await)
    }
    pub async fn metric(&self, street: Street) -> anyhow::Result<Metric> {
        Ok(self.0.metric(street).await)
    }
}

// equity calculations
impl API {
    pub async fn abs_equity(&self, abs: Abstraction) -> anyhow::Result<Probability> {
        Ok(self.0.equity(abs).await)
    }
    #[rustfmt::skip]
    pub async fn obs_equity(&self, obs: Observation) -> anyhow::Result<Probability> {
        let iso = i64::from(Isomorphism::from(obs));
        const RIVER: &str = const_format::concatcp!(
            "SELECT equity       ",
            "FROM   ", ISOMORPHISM, " ",
            "WHERE  obs = $1"
        );
        const OTHER: &str = const_format::concatcp!(
            "SELECT SUM(t.dx * a.equity) ",
            "FROM   ", TRANSITIONS,  " t ",
            "JOIN   ", ISOMORPHISM,  " e ON e.abs = t.prev ",
            "JOIN   ", ABSTRACTION,  " a ON a.abs = t.next ",
            "WHERE  e.obs = $1"
        );
        let sql = if obs.street() == Street::Rive { RIVER } else { OTHER };
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
        Ok(self.0.distance(Pair::from((&abs1, &abs2))).await)
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
        Ok(self.0.population(abs).await)
    }
    #[rustfmt::skip]
    pub async fn obs_population(&self, obs: Observation) -> anyhow::Result<usize> {
        let iso = i64::from(Isomorphism::from(obs));
        const SQL: &str = const_format::concatcp!(
            "SELECT population   ",
            "FROM   ", ABSTRACTION,  " a ",
            "JOIN   ", ISOMORPHISM,  " e ON e.abs = a.abs ",
            "WHERE  e.obs = $1"
        );
        Ok(self
            .0
            .query_one(SQL, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observation population: {}", e))?
            .get::<_, i64>(0) as usize)
    }
}

// histogram aggregation via join
impl API {
    pub async fn abs_histogram(&self, abs: Abstraction) -> anyhow::Result<Histogram> {
        Ok(self.0.histogram(abs).await)
    }
    #[rustfmt::skip]
    pub async fn obs_histogram(&self, obs: Observation) -> anyhow::Result<Histogram> {
        let idx = i64::from(Isomorphism::from(obs));
        let mass = obs.street().n_children() as f32;
        const SQL: &str = const_format::concatcp!(
            "SELECT next, ",
                   "dx ",
            "FROM   ", TRANSITIONS,  " t ",
            "JOIN   ", ISOMORPHISM,  " e ON e.abs = t.prev ",
            "WHERE  e.obs = $1"
        );
        let street = obs.street().next();
        Ok(self
            .0
            .query(SQL, &[&idx])
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

// observation similarity lookups
impl API {
    #[rustfmt::skip]
    pub async fn obs_similar(&self, obs: Observation) -> anyhow::Result<Vec<Observation>> {
        let iso = i64::from(Isomorphism::from(obs));
        const SQL: &str = const_format::concatcp!(
            "WITH target AS ( ",
                "SELECT abs, population ",
                "FROM   ", ISOMORPHISM, " e ",
                "JOIN   ", ABSTRACTION, " a ON a.abs = e.abs ",
                "WHERE  e.obs = $1 ",
            ") ",
            "SELECT   e.obs ",
            "FROM     ", ISOMORPHISM, " e ",
            "JOIN     target t ON t.abs = e.abs ",
            "WHERE    e.obs != $1 ",
            "AND      e.position  < LEAST($2, t.population) ",
            "AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(SQL, &[&iso, &N_SAMPLES])
            .await
            .map_err(|e| anyhow::anyhow!("fetch similar observations: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    #[rustfmt::skip]
    pub async fn abs_similar(&self, abs: Abstraction) -> anyhow::Result<Vec<Observation>> {
        let abs = i16::from(abs);
        const SQL: &str = const_format::concatcp!(
            "WITH target AS ( ",
                "SELECT population ",
                "FROM   ", ABSTRACTION, " ",
                "WHERE  abs = $1 ",
            ") ",
            "SELECT   obs ",
            "FROM     ", ISOMORPHISM, " e, target t ",
            "WHERE    e.abs = $1 ",
            "AND      e.position  < LEAST($2, t.population) ",
            "AND      e.position >= FLOOR(RANDOM() * GREATEST(t.population - $2, 1)) ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(SQL, &[&abs, &N_SAMPLES])
            .await
            .map_err(|e| anyhow::anyhow!("fetch observations similar to abstraction: {}", e))?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    #[rustfmt::skip]
    pub async fn replace_obs(&self, obs: Observation) -> anyhow::Result<Observation> {
        const SQL: &str = const_format::concatcp!(
            "WITH sample AS ( ",
                "SELECT e.abs, ",
                       "a.population, ",
                       "FLOOR(RANDOM() * a.population)::INTEGER AS i ",
                "FROM   ", ISOMORPHISM, " e ",
                "JOIN   ", ABSTRACTION, " a ON a.abs = e.abs ",
                "WHERE  e.obs = $1 ",
            ") ",
            "SELECT e.obs ",
            "FROM   sample s ",
            "JOIN   ", ISOMORPHISM, " e ON e.abs = s.abs AND e.position = s.i ",
            "LIMIT  1"
        );
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(SQL, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("replace observation: {}", e))?;
        Ok(Observation::from(row.get::<_, i64>(0)))
    }
}

// proximity lookups
impl API {
    #[rustfmt::skip]
    pub async fn abs_nearby(&self, abs: Abstraction) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        let abs = i16::from(abs);
        const SQL: &str = const_format::concatcp!(
            "SELECT   a.abs, ",
                     "m.dx ",
            "FROM     ", ABSTRACTION, " a ",
            "JOIN     ", METRIC,      " m ON m.tri = get_pair_tri(a.abs, $1) ",
            "WHERE    a.abs != $1 ",
            "ORDER BY m.dx ASC ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(SQL, &[&abs, &N_SAMPLES])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
    #[rustfmt::skip]
    pub async fn obs_nearby(&self, obs: Observation) -> anyhow::Result<Vec<(Abstraction, Energy)>> {
        let iso = i64::from(Isomorphism::from(obs));
        const SQL: &str = const_format::concatcp!(
            "SELECT   a.abs, ",
                     "m.dx ",
            "FROM     ", ISOMORPHISM, " e ",
            "JOIN     ", ABSTRACTION, " a ON a.street = get_street_abs(e.abs) ",
            "JOIN     ", METRIC,      " m ON m.tri = get_pair_tri(a.abs, e.abs) ",
            "WHERE    e.obs = $1 ",
            "AND      a.abs != e.abs ",
            "ORDER BY m.dx ASC ",
            "LIMIT    $2"
        );
        Ok(self
            .0
            .query(SQL, &[&iso, &N_SAMPLES])
            .await
            .map_err(|e| anyhow::anyhow!("fetch nearby abstractions for observation: {}", e))?
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
}

// exploration panel
impl API {
    pub async fn exp_wrt_str(&self, str: Street) -> anyhow::Result<ApiSample> {
        self.exp_wrt_obs(Observation::from(str)).await
    }
    #[rustfmt::skip]
    pub async fn exp_wrt_obs(&self, obs: Observation) -> anyhow::Result<ApiSample> {
        const SQL: &str = const_format::concatcp!(
            "SELECT e.obs, ",
                   "a.abs, ",
                   "a.equity::REAL, ",
                   "a.population::REAL / $2 AS density ",
            "FROM   ", ISOMORPHISM, " e ",
            "JOIN   ", ABSTRACTION, " a ON a.abs = e.abs ",
            "WHERE  e.obs = $1"
        );
        let n = obs.street().n_observations() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let row = self
            .0
            .query_one(SQL, &[&iso, &n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to observation: {}", e))?;
        Ok(ApiSample::from(row))
    }
    #[rustfmt::skip]
    pub async fn exp_wrt_abs(&self, abs: Abstraction) -> anyhow::Result<ApiSample> {
        const SQL: &str = const_format::concatcp!(
            "WITH sample AS ( ",
                "SELECT a.abs, ",
                       "a.population, ",
                       "a.equity, ",
                       "FLOOR(RANDOM() * a.population)::INTEGER AS i ",
                "FROM   ", ABSTRACTION, " a ",
                "WHERE  a.abs = $1 ",
            ") ",
            "SELECT e.obs, ",
                   "s.abs, ",
                   "s.equity::REAL, ",
                   "s.population::REAL / $2 AS density ",
            "FROM   sample s ",
            "JOIN   ", ISOMORPHISM, " e ON e.abs = s.abs AND e.position = s.i ",
            "LIMIT  1"
        );
        let n = abs.street().n_isomorphisms() as f32;
        let abs = i16::from(abs);
        let row = self
            .0
            .query_one(SQL, &[&abs, &n])
            .await
            .map_err(|e| anyhow::anyhow!("explore with respect to abstraction: {}", e))?;
        Ok(ApiSample::from(row))
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
    #[rustfmt::skip]
    pub async fn nbr_abs_wrt_abs(
        &self,
        wrt: Abstraction,
        abs: Abstraction,
    ) -> anyhow::Result<ApiSample> {
        const SQL: &str = const_format::concatcp!(
            "WITH sample AS ( ",
                "SELECT r.abs, ",
                       "r.population, ",
                       "r.equity, ",
                       "FLOOR(RANDOM() * r.population)::INTEGER AS i, ",
                       "COALESCE(m.dx, 0) AS distance ",
                "FROM      ", ABSTRACTION, " r ",
                "LEFT JOIN ", METRIC,      " m ON m.tri = get_pair_tri($1, $3) ",
                "WHERE     r.abs = $1 ",
            "), ",
            "random_iso AS ( ",
                "SELECT e.obs, ",
                       "e.abs, ",
                       "s.equity, ",
                       "s.population, ",
                       "s.distance ",
                "FROM   sample s ",
                "JOIN   ", ISOMORPHISM, " e ON e.abs = s.abs AND e.position = s.i ",
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
            .query_one(SQL, &[&abs, &n, &wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor abstraction: {}", e))?;
        Ok(ApiSample::from(row))
    }
    #[rustfmt::skip]
    pub async fn nbr_obs_wrt_abs(
        &self,
        wrt: Abstraction,
        obs: Observation,
    ) -> anyhow::Result<ApiSample> {
        const SQL: &str = const_format::concatcp!(
            "WITH given AS ( ",
                "SELECT obs, abs, get_pair_tri(abs, $3) AS tri ",
                "FROM   ", ISOMORPHISM, " ",
                "WHERE  obs = $1 ",
            ") ",
            "SELECT g.obs, ",
                   "g.abs, ",
                   "a.equity::REAL, ",
                   "a.population::REAL / $2 AS density, ",
                   "COALESCE(m.dx, 0)::REAL AS distance ",
            "FROM   given g ",
            "JOIN   ", METRIC,      " m ON m.tri = g.tri ",
            "JOIN   ", ABSTRACTION, " a ON a.abs = g.abs ",
            "LIMIT  1"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let iso = i64::from(Isomorphism::from(obs));
        let wrt = i16::from(wrt);
        let row = self
            .0
            .query_one(SQL, &[&iso, &n, &wrt])
            .await
            .map_err(|e| anyhow::anyhow!("fetch neighbor observation: {}", e))?;
        Ok(ApiSample::from(row))
    }
}

// k-nearest neighbors lookups
impl API {
    #[rustfmt::skip]
    pub async fn kfn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "WITH nearest AS ( ",
                "SELECT   a.abs, ",
                         "a.population, ",
                         "m.dx AS distance, ",
                         "FLOOR(RANDOM() * a.population)::INTEGER AS sample ",
                "FROM     ", ABSTRACTION, " a ",
                "JOIN     ", METRIC,      " m ON m.tri = get_pair_tri(a.abs, $1) ",
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
            "JOIN     ", ABSTRACTION, " a ON a.abs = n.abs ",
            "JOIN     ", ISOMORPHISM, " e ON e.abs = n.abs AND e.position = n.sample ",
            "ORDER BY n.distance DESC"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(SQL, &[&wrt, &s, &N_SAMPLES, &n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-farthest neighbors: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
    }
    #[rustfmt::skip]
    pub async fn knn_wrt_abs(&self, wrt: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "WITH nearest AS ( ",
                "SELECT   a.abs, ",
                         "a.population, ",
                         "m.dx AS distance, ",
                         "FLOOR(RANDOM() * a.population)::INTEGER AS sample ",
                "FROM     ", ABSTRACTION, " a ",
                "JOIN     ", METRIC,      " m ON m.tri = get_pair_tri(a.abs, $1) ",
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
            "JOIN     ", ABSTRACTION, " a ON a.abs = n.abs ",
            "JOIN     ", ISOMORPHISM, " e ON e.abs = n.abs AND e.position = n.sample ",
            "ORDER BY n.distance ASC"
        );
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i16::from(wrt);
        let rows = self
            .0
            .query(SQL, &[&wrt, &s, &N_SAMPLES, &n])
            .await
            .map_err(|e| anyhow::anyhow!("fetch k-nearest neighbors: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
    }
    #[rustfmt::skip]
    pub async fn kgn_wrt_abs(
        &self,
        wrt: Abstraction,
        nbr: Vec<Observation>,
    ) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
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
            "JOIN     ", ISOMORPHISM, " e ON e.obs = i.obs ",
            "JOIN     ", ABSTRACTION, " a ON a.abs = e.abs ",
            "JOIN     ", METRIC,      " m ON m.tri = get_pair_tri(a.abs, $2) ",
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
            .query(SQL, &[&n, &wrt, &&isos, &N_SAMPLES])
            .await
            .map_err(|e| anyhow::anyhow!("fetch given neighbors: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
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
    #[rustfmt::skip]
    async fn hst_wrt_obs_on_river(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "WITH sample AS ( ",
                "SELECT e.obs, ",
                       "e.abs, ",
                       "a.equity, ",
                       "a.population, ",
                       "FLOOR(RANDOM() * a.population)::INTEGER AS position ",
                "FROM   ", ISOMORPHISM, " e ",
                "JOIN   ", ABSTRACTION, " a ON a.abs = e.abs ",
                "WHERE  e.abs = (SELECT abs FROM ", ISOMORPHISM, " WHERE obs = $1) ",
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
            .query(SQL, &[&iso])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river observation distribution: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
    }
    #[rustfmt::skip]
    async fn hst_wrt_obs_on_other(&self, obs: Observation) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "SELECT e.obs, ",
                   "e.abs, ",
                   "a.equity ",
            "FROM   ", ISOMORPHISM, " e ",
            "JOIN   ", ABSTRACTION, " a ON a.abs = e.abs ",
            "WHERE  e.obs = ANY($1)"
        );
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
            .query(SQL, &[&distinct])
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
                    btree.entry(abs).or_insert((obs, eqy, 0)).2 += 1;
                    btree
                },
            )
            .into_iter()
            .map(|(abs, (obs, eqy, pop))| ApiSample {
                obs: obs.to_string(),
                abs: abs.to_string(),
                equity: eqy.clone(),
                density: pop as Probability / n as Probability,
                distance: 0.,
            })
            .collect::<Vec<_>>();
        Ok(hist)
    }
    #[rustfmt::skip]
    async fn hst_wrt_abs_on_river(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "WITH sample AS ( ",
                "SELECT a.abs, ",
                       "a.population, ",
                       "a.equity, ",
                       "FLOOR(RANDOM() * a.population)::INTEGER AS position ",
                "FROM   ", ABSTRACTION, " a ",
                "WHERE  a.abs = $1 ",
                "LIMIT  1 ",
            ") ",
            "SELECT e.obs, ",
                   "e.abs, ",
                   "s.equity::REAL, ",
                   "1::REAL AS density ",
            "FROM   sample s ",
            "JOIN   ", ISOMORPHISM, " e ON e.abs = s.abs AND e.position = s.position"
        );
        let ref abs = i16::from(abs);
        let rows = self
            .0
            .query(SQL, &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch river abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
    }
    #[rustfmt::skip]
    async fn hst_wrt_abs_on_other(&self, abs: Abstraction) -> anyhow::Result<Vec<ApiSample>> {
        const SQL: &str = const_format::concatcp!(
            "WITH histogram AS ( ",
                "SELECT p.abs, ",
                       "g.dx AS probability, ",
                       "p.population, ",
                       "p.equity, ",
                       "FLOOR(RANDOM() * p.population)::INTEGER AS i ",
                "FROM   ", TRANSITIONS,  " g ",
                "JOIN   ", ABSTRACTION,  " p ON p.abs = g.next ",
                "WHERE  g.prev = $1 ",
            ") ",
            "SELECT   e.obs, ",
                     "t.abs, ",
                     "t.equity::REAL, ",
                     "t.probability AS density ",
            "FROM     histogram t ",
            "JOIN     ", ISOMORPHISM, " e ON e.abs = t.abs AND e.position = t.i ",
            "ORDER BY t.probability DESC"
        );
        let ref abs = i16::from(abs);
        let rows = self
            .0
            .query(SQL, &[abs])
            .await
            .map_err(|e| anyhow::anyhow!("fetch abstraction distribution: {}", e))?;
        Ok(rows.into_iter().map(ApiSample::from).collect())
    }
}

// blueprint lookups
impl API {
    #[rustfmt::skip]
    pub async fn policy(&self, recall: Recall) -> anyhow::Result<Option<ApiStrategy>> {
        const SQL: &str = const_format::concatcp!(
            "SELECT edge, ",
                   "policy ",
            "FROM   ", BLUEPRINT, " ",
            "WHERE  past    = $1 ",
            "AND    present = $2 ",
            "AND    future  = $3"
        );
        let recall = recall.validate()?;
        // Path order mismatch between training and inference:
        // - Training (Encoder::info): Node iterator traverses parent-by-parent (reverse chronological)
        //   collecting into Path stores most recent edge at higher-order nibbles
        // - Inference (Recall::path): scan forward through actions (chronological order)
        //   collecting into Path stores oldest edge at lower-order nibbles
        // Database contains training format, so reverse Recall::path() before querying
        // @history-reversed
        let present = self.obs_to_abs(recall.seen()).await?;
        let info = recall.bind(present);
        let ref history = i64::from(*info.history());
        let ref present = i16::from(*info.present());
        let ref choices = i64::from(*info.choices());
        let rows = self
            .0
            .query(SQL, &[history, present, choices])
            .await
            .map_err(|e| anyhow::anyhow!("fetch policy: {}", e))?;
        match rows.len() {
            0 => Ok(None),
            _ => Ok(Some(ApiStrategy::from(Strategy::from((
                info,
                rows.into_iter().map(Decision::from).collect::<Vec<_>>(),
            ))))),
        }
    }
}
