use super::response::Row;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use crate::clustering::pair::Pair;
use crate::clustering::sinkhorn::Sinkhorn;
use crate::transport::coupling::Coupling;
use crate::Energy;
use crate::Probability;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct API(Arc<Client>);

impl From<Arc<Client>> for API {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl API {
    pub async fn new() -> Self {
        Self(crate::db().await)
    }

    // global lookups
    pub async fn encode(&self, obs: Observation) -> Result<Abstraction, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT abs
            FROM encoder
            WHERE obs = $1
        "#;
        Ok(self
            .0
            .query_one(SQL, &[&iso])
            .await?
            .get::<_, i64>(0)
            .into())
    }
    pub async fn metric(&self, street: Street) -> Result<Metric, E> {
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT
                a1.abs # a2.abs AS xor,
                m.dx            AS dx
            FROM abstraction a1
            JOIN abstraction a2
                ON a1.street = a2.street
            JOIN metric m
                ON (a1.abs # a2.abs) = m.xor
            WHERE
                a1.street   = $1 AND
                a1.abs     != a2.abs;
        "#;
        Ok(self
            .0
            .query(SQL, &[&street])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(xor, distance)| (Pair::from(xor), distance))
            .collect::<BTreeMap<Pair, Energy>>()
            .into())
    }
    pub async fn basis(&self, street: Street) -> Result<Vec<Abstraction>, E> {
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT a2.abs
            FROM abstraction a2
            JOIN abstraction a1 ON a2.street = a1.street
            WHERE a1.abs = $1;
        "#;
        Ok(self
            .0
            .query(SQL, &[&street])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Abstraction::from)
            .collect())
    }

    // equity calculations
    pub async fn abs_equity(&self, abs: Abstraction) -> Result<Probability, E> {
        let iso = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT equity
            FROM abstraction
            WHERE abs = $1
        "#;
        Ok(self
            .0
            .query_one(SQL, &[&iso])
            .await?
            .get::<_, f32>(0)
            .into())
    }
    pub async fn obs_equity(&self, obs: Observation) -> Result<Probability, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        let sql = if obs.street() == Street::Rive {
            r#"
                SELECT equity
                FROM encoder
                WHERE obs = $1
            "#
        } else {
            r#"
                SELECT SUM(t.dx * a.equity)
                FROM transitions t
                JOIN encoder     e ON e.abs = t.prev
                JOIN abstraction a ON a.abs = t.next
                WHERE e.obs = $1
            "#
        };
        Ok(self
            .0
            .query_one(sql, &[&iso])
            .await?
            .get::<_, f32>(0)
            .into())
    }

    // distance calculations
    pub async fn abs_distance(&self, abs1: Abstraction, abs2: Abstraction) -> Result<Energy, E> {
        if abs1.street() != abs2.street() {
            return Err(E::__private_api_timeout());
        }
        if abs1 == abs2 {
            return Ok(0 as Energy);
        }
        let xor = i64::from(Pair::from((&abs1, &abs2)));
        const SQL: &'static str = r#"
            SELECT m.dx
            FROM metric m
            WHERE $1 = m.xor;
        "#;
        Ok(self.0.query_one(SQL, &[&xor]).await?.get::<_, Energy>(0))
    }
    pub async fn obs_distance(&self, obs1: Observation, obs2: Observation) -> Result<Energy, E> {
        if obs1.street() != obs2.street() {
            return Err(E::__private_api_timeout());
        }
        let (ref hx, ref hy, ref metric) = tokio::try_join!(
            self.obs_histogram(obs1),
            self.obs_histogram(obs2),
            self.metric(obs1.street().next())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }

    // population lookups
    pub async fn abs_population(&self, abs: Abstraction) -> Result<usize, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT population
            FROM abstraction
            WHERE abs = $1
        "#;
        Ok(self.0.query_one(SQL, &[&abs]).await?.get::<_, i32>(0) as usize)
    }
    pub async fn obs_population(&self, obs: Observation) -> Result<usize, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT population
            FROM abstraction
            JOIN encoder ON encoder.abs = abstraction.abs
            WHERE obs = $1
        "#;
        Ok(self.0.query_one(SQL, &[&iso]).await?.get::<_, i64>(0) as usize)
    }

    // centrality (mean distance) lookups
    pub async fn abs_centrality(&self, abs: Abstraction) -> Result<Probability, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT centrality
            FROM abstraction
            WHERE abs = $1
        "#;
        Ok(self
            .0
            .query_one(SQL, &[&abs])
            .await?
            .get::<_, f32>(0)
            .into())
    }
    pub async fn obs_centrality(&self, obs: Observation) -> Result<Probability, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT centrality
            FROM abstraction
            JOIN encoder ON encoder.abs = abstraction.abs
            WHERE obs = $1
        "#;
        Ok(self
            .0
            .query_one(SQL, &[&iso])
            .await?
            .get::<_, f32>(0)
            .into())
    }

    // histogram aggregation via join
    pub async fn abs_histogram(&self, abs: Abstraction) -> Result<Histogram, E> {
        let idx = i64::from(abs);
        let mass = abs.street().n_children() as f32;
        const SQL: &'static str = r#"
            SELECT next, dx
            FROM transitions
            WHERE prev = $1
        "#;
        Ok(self
            .0
            .query(SQL, &[&idx])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (next, (dx * mass).round() as usize))
            .map(|(next, dx)| (Abstraction::from(next), dx))
            .fold(Histogram::default(), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            }))
    }
    pub async fn obs_histogram(&self, obs: Observation) -> Result<Histogram, E> {
        // Kd8s~6dJsAc
        let idx = i64::from(Observation::from(Isomorphism::from(obs)));
        let mass = obs.street().n_children() as f32;
        const SQL: &'static str = r#"
            SELECT next, dx
            FROM transitions
            JOIN encoder ON encoder.abs = transitions.prev
            WHERE encoder.obs = $1
        "#;
        Ok(self
            .0
            .query(SQL, &[&idx])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (next, (dx * mass).round() as usize))
            .map(|(next, dx)| (Abstraction::from(next), dx))
            .fold(Histogram::default(), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            }))
    }

    // observation similarity lookups
    pub async fn obs_similar(&self, obs: Observation) -> Result<Vec<Observation>, E> {
        // 8d8s~6dJs7c
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT obs
            FROM encoder
            WHERE abs = (
                SELECT abs
                FROM encoder
                WHERE obs = $1
            )
            AND obs != $1
            ORDER BY RANDOM()
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&iso])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    pub async fn abs_similar(&self, abs: Abstraction) -> Result<Vec<Observation>, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT obs
            FROM encoder
            WHERE abs = $1
            ORDER BY RANDOM()
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }

    // proximity lookups
    pub async fn abs_nearby(&self, abs: Abstraction) -> Result<Vec<(Abstraction, Energy)>, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT a1.abs, m.dx
            FROM abstraction    a1
            JOIN abstraction    a2 ON a1.street = a2.street
            JOIN metric         m  ON (a1.abs # $1) = m.xor
            WHERE
                a2.abs  = $1 AND
                a1.abs != $1
            ORDER BY m.dx ASC
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }
    pub async fn obs_nearby(&self, obs: Observation) -> Result<Vec<(Abstraction, Energy)>, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT a1.abs, m.dx
            FROM encoder        e
            JOIN abstraction    a2 ON e.abs = a2.abs
            JOIN abstraction    a1 ON a1.street = a2.street
            JOIN metric         m  ON (a1.abs # e.abs) = m.xor
            WHERE
                e.obs   = $1 AND
                a1.abs != e.abs
            ORDER BY m.dx ASC
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&iso])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }

    // HTTP endpoints
    pub async fn exploration_row(&self, street: Street) -> Result<Row, E> {
        let obs = Observation::from(street);
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        let n = street.n_observations() as f32;
        const SQL: &'static str = r#"
            SELECT
                e.obs,
                a.abs,
                a.equity::REAL          as equity,
                a.population::REAL / $2 as density,
                a.centrality::REAL      as centrality
            FROM encoder e
            JOIN abstraction a ON e.abs = a.abs
            WHERE e.obs = $1;
        "#;
        let row = self.0.query_one(SQL, &[&iso, &n]).await?;
        Ok(Row {
            obs: Observation::from(row.get::<_, i64>(0)).to_string(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        })
    }

    // neighborhood row replacements
    pub async fn calculate_obs(&self, wrt: Abstraction, obs: Observation) -> Result<Row, E> {
        // uniform within cluster
        let n = wrt.street().n_isomorphisms() as f32;
        let wrt = i64::from(wrt);
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT
                e.obs,
                e.abs,
                r.equity::REAL                      as equity,
                r.population::REAL / $2             as density,
                COALESCE(m.dx, 0)::REAL             as distance
            FROM encoder        e
            JOIN abstraction    r ON e.abs = r.abs
            JOIN metric         m ON m.xor = (e.abs # $3)
            WHERE e.obs = $1
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&iso, &n, &wrt]).await?;
        Ok(Row {
            obs: Observation::from(row.get::<_, i64>(0)).to_string(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        })
    }
    pub async fn calculate_abs(&self, wrt: Abstraction, abs: Abstraction) -> Result<Row, E> {
        // direct calculation, no sampling
        let n = wrt.street().n_isomorphisms() as f32;
        let abs = i64::from(abs);
        let wrt = i64::from(wrt);
        const SQL: &'static str = r#"
            WITH
            sample AS (
                SELECT  abs,
                        FLOOR(RANDOM() * a.population)::INT as choice
                FROM abstraction a
                WHERE a.abs = $1
            )
            SELECT
                e.obs                               as obs,
                r.abs                               as abs,
                r.equity::REAL                      as equity,
                r.population::REAL / $2             as density,
                COALESCE(m.dx, 0)::REAL             as distance
            FROM sample         s
            JOIN abstraction    r ON s.abs = r.abs
            JOIN metric         m ON m.xor = (s.abs # $3)
            JOIN encoder        e ON e.abs = (s.abs)
            OFFSET (SELECT choice FROM sample)
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&abs, &n, &wrt]).await?;
        Ok(Row {
            obs: Observation::from(row.get::<_, i64>(0)).to_string(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        })
    }
    pub async fn calculate_any(&self, wrt: Abstraction) -> Result<Row, E> {
        // uniform over abstraction space
        use rand::seq::SliceRandom;
        let ref mut rng = rand::thread_rng();
        let abs = Abstraction::all(wrt.street())
            .into_iter()
            .filter(|&x| x != wrt)
            .collect::<Vec<_>>()
            .choose(rng)
            .copied()
            .unwrap();
        self.calculate_abs(wrt, abs).await
    }

    // dice roll within same cluster
    pub async fn obs_swap(&self, obs: Observation) -> Result<Observation, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            WITH
            sample AS (
                SELECT e.abs,
                       a.population,
                       FLOOR(RANDOM() * (a.population - 1))::INT AS choice
                FROM abstraction a
                JOIN encoder e ON e.abs = a.abs
                WHERE e.obs = $1
                LIMIT 1
            )
            SELECT e.obs
            FROM sample
            JOIN encoder e ON   e.abs  = sample.abs
            WHERE               e.obs != $1
            OFFSET (SELECT choice FROM sample)
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&iso]).await?;
        Ok(Observation::from(row.get::<_, i64>(0)))
    }

    pub async fn table_neighborhood_kfn(&self, wrt: Abstraction) -> Result<Vec<Row>, E> {
        self.table_neighborhood_knn(wrt).await
    }

    pub async fn table_neighborhood_knn(&self, wrt: Abstraction) -> Result<Vec<Row>, E> {
        let n = wrt.street().n_isomorphisms() as f32;
        let s = wrt.street() as i16;
        let wrt = i64::from(wrt);
        const SQL: &'static str = r#"
            SELECT 
                a.abs,
                c.obs,
                a.equity::REAL          as equity,
                a.population::REAL / $1 as density,
                m.dx::REAL              as distance
            FROM abstraction a
            JOIN metric m ON m.xor = (a.abs # $2)
            CROSS JOIN LATERAL (
                SELECT c.obs 
                FROM encoder c
                WHERE c.abs = a.abs
                OFFSET FLOOR(RANDOM() * a.population)::INT
                LIMIT 1
            ) c
            WHERE a.street = $3
            ORDER BY m.dx ASC
            LIMIT 5;
        "#;
        let rows = self.0.query(SQL, &[&n, &wrt, &s]).await?;
        Ok(rows
            .iter()
            .map(|row| Row {
                abs: Abstraction::from(row.get::<_, i64>(0)).to_string(),
                obs: Observation::from(row.get::<_, i64>(1)).to_string(),
                equity: row.get::<_, f32>(2).into(),
                density: row.get::<_, f32>(3).into(),
                distance: row.get::<_, f32>(4).into(),
            })
            .collect())
    }
    pub async fn table_distribution(&self, abs: Abstraction) -> Result<Vec<Row>, E> {
        let abs_i64 = i64::from(abs);
        if abs.street() == Street::Rive {
            self.table_distribution_river(abs_i64).await
        } else {
            self.table_distribution_other(abs_i64).await
        }
    }
    async fn table_distribution_river(&self, abs: i64) -> Result<Vec<Row>, E> {
        let n = Street::Rive.n_isomorphisms() as f32;
        const SQL: &'static str = r#"
            SELECT 
                c.obs                   as obs,
                p.abs                   as abs,
                p.equity::REAL          as equity,
                p.population::REAL / $1 as density,
                p.centrality::REAL      as distance
            FROM abstraction g
            JOIN encoder     p ON p.abs = g.abs
            CROSS JOIN LATERAL (
                SELECT c.obs 
                FROM encoder c
                WHERE c.abs = p.abs
                OFFSET FLOOR(RANDOM() * p.population)::INT
                LIMIT 1
            ) c
            WHERE g.abs = $2
            LIMIT 5;
        "#;
        let rows = self.0.query(SQL, &[&n, &abs]).await?;
        Ok(rows
            .iter()
            .map(|row| Row {
                obs: Observation::from(row.get::<_, i64>(0)).to_string(),
                abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
                equity: Probability::from(Abstraction::from(row.get::<_, i64>(1))),
                density: row.get::<_, f32>(3).into(),
                distance: row.get::<_, f32>(4).into(),
            })
            .collect())
    }
    async fn table_distribution_other(&self, abs: i64) -> Result<Vec<Row>, E> {
        const SQL: &'static str = r#"
            SELECT 
                c.obs               as obs,
                p.abs               as abs,
                p.equity::REAL      as equity,
                g.dx::REAL          as density,
                p.centrality::REAL  as distance
            FROM transitions g
            JOIN abstraction p ON p.abs = g.next
            CROSS JOIN LATERAL (
                SELECT c.obs 
                FROM encoder c
                WHERE c.abs = p.abs
                OFFSET FLOOR(RANDOM() * p.population)::INT
                LIMIT 1
            ) c
            WHERE g.prev = $1
            LIMIT 128;
        "#;
        let rows = self.0.query(SQL, &[&abs]).await?;
        Ok(rows
            .iter()
            .map(|row| Row {
                obs: Observation::from(row.get::<_, i64>(0)).to_string(),
                abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
                equity: row.get::<_, f32>(2).into(),
                density: row.get::<_, f32>(3).into(),
                distance: row.get::<_, f32>(4).into(),
            })
            .collect())
    }
}
