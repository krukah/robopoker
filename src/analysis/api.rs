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

impl API {
    pub async fn new() -> Self {
        log::info!("connecting to db (API)");
        let (client, connection) = tokio_postgres::Config::default()
            .port(5432)
            .host("localhost")
            .user("postgres")
            .dbname("robopoker")
            .password("postgrespassword")
            .connect(tokio_postgres::NoTls)
            .await
            .expect("db connection");
        tokio::spawn(connection);
        Self(Arc::new(client))
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
        // dob Kd8s~6dJsAc QhQs~QdQcAc
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
            FROM abstraction a1
            JOIN abstraction a2 ON a1.street = a2.street
            JOIN metric m ON (a1.abs # $1) = m.xor
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
            FROM encoder e
            JOIN abstraction a2 ON e.abs = a2.abs
            JOIN abstraction a1 ON a1.street = a2.street
            JOIN metric m ON (a1.abs # e.abs) = m.xor
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
    pub async fn row_wrt_street(&self, street: Street) -> Result<Row, E> {
        let obs = Observation::from(street);
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        let n = street.n_observations() as f32;
        const SQL: &'static str = r#"
            SELECT 
                e.obs,
                a.abs,
                a.equity,
                a.population::REAL / $2 as density,   
                a.centrality
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
    pub async fn row_wrt_abs_via_abs(&self, abs: Abstraction) -> Result<Row, E> {
        let street = abs.street();
        // Get all abstractions for this street except the input abs
        let all = Abstraction::all(street)
            .into_iter()
            .filter(|&x| x != abs)
            .map(i64::from)
            .collect::<Vec<i64>>();
        let abs = i64::from(abs);
        let n = street.n_observations() as f32;
        const SQL: &'static str = r#"
            WITH randabs AS (
                SELECT abs 
                FROM (
                    SELECT DISTINCT abs
                    FROM abstraction 
                    WHERE abs = ANY($3)
                ) sub
                ORDER BY random()
                LIMIT 1
            )
            SELECT 
                e.obs                   as obs,
                a.abs                   as abs,
                a.equity                as equity,
                a.population::REAL / $2 as density,
                COALESCE(m.dx, 0)      as distance
            FROM randabs ra
            JOIN abstraction a ON a.abs = ra.abs
            JOIN encoder e ON e.abs = a.abs
            JOIN metric m ON (a.abs # $1) = m.xor
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&abs, &n, &all]).await?;
        Ok(Row {
            obs: Observation::from(row.get::<_, i64>(0)).to_string(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        })
    }
    pub async fn row_wrt_abs_via_obs(&self, abs: Abstraction, obs: Observation) -> Result<Row, E> {
        let street = abs.street();
        // Get all abstractions for this street
        let all = Abstraction::all(street)
            .into_iter()
            .map(i64::from)
            .collect::<Vec<i64>>();
        let abs_i64 = i64::from(abs);
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        let n = obs.street().n_observations() as f32;
        const SQL: &'static str = r#"
            WITH target AS (
                SELECT 
                    e.obs,
                    a.abs,
                    a.equity,
                    a.population::REAL / $3 as density
                FROM encoder e
                JOIN abstraction a ON e.abs = a.abs
                WHERE e.obs = $2
                  AND a.abs = ANY($4)
            )
            SELECT 
                t.obs,
                t.abs,
                t.equity,
                t.density,
                COALESCE(m.dx, 0) as distance
            FROM target t
            INNER JOIN metric m ON (t.abs # $1) = m.xor
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&abs_i64, &iso, &n, &all]).await?;
        Ok(Row {
            obs: Observation::from(row.get::<_, i64>(0)).to_string(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        })
    }

    pub async fn replace_obs(&self, obs: Observation) -> Result<Observation, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        // First get the abstraction for this observation
        const SQL: &'static str = r#"
            WITH t AS (
                SELECT a.abs,
                       a.population,
                       FLOOR(RANDOM() * (a.population - 1))::INT AS rando
                FROM abstraction a
                JOIN encoder e ON e.abs = a.abs
                WHERE e.obs = $1
                LIMIT 1
            )
            SELECT e2.obs
            FROM encoder e2
            JOIN t ON e2.abs = t.abs
            WHERE e2.obs != $1
            OFFSET (SELECT rando FROM t)
            LIMIT 1;
        "#;
        let row = self.0.query_one(SQL, &[&iso]).await?;
        Ok(Observation::from(row.get::<_, i64>(0)))
    }

    pub async fn table_neighborhood_knn(&self, abs: Abstraction) -> Result<Vec<Row>, E> {
        let street = abs.street();
        let all = Abstraction::all(street)
            .into_iter()
            .filter(|&x| x != abs)
            .map(i64::from)
            .collect::<Vec<i64>>();
        let n = street.n_observations() as f32;
        let abs = i64::from(abs);
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT 
                a.abs,
                (SELECT obs FROM encoder e2 WHERE e2.abs = a.abs LIMIT 1) as obs,
                a.equity,
                a.population::REAL / $3 as density,
                m.dx as distance
            FROM abstraction a
            JOIN metric m ON m.xor = (a.abs # $1)
            WHERE a.street = $2
              AND a.abs = ANY($4)
            ORDER BY m.dx ASC
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs, &street, &n, &all])
            .await?
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
    pub async fn table_neighborhood_kfn(&self, abs: Abstraction) -> Result<Vec<Row>, E> {
        let street = abs.street();
        let all = Abstraction::all(street)
            .into_iter()
            .filter(|&x| x != abs)
            .map(i64::from)
            .collect::<Vec<i64>>();
        let n = street.n_observations() as f32;
        let abs = i64::from(abs);
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT 
                a.abs,
                (SELECT obs FROM encoder e2 WHERE e2.abs = a.abs LIMIT 1) as obs,
                a.equity,
                a.population::REAL / $3 as density,
                m.dx as distance
            FROM abstraction a
            JOIN metric m ON m.xor = (a.abs # $1)
            WHERE a.street = $2
              AND a.abs = ANY($4)
            ORDER BY m.dx DESC
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs, &street, &n, &all])
            .await?
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
        const SQL: &'static str = r#"
            SELECT 
                e.obs               as obs,
                a.abs               as abs,
                1.0::REAL           as equity,
                1.0::REAL           as density,
                a.centrality::REAL  as centrality
            FROM abstraction a
            JOIN encoder e ON e.abs = a.abs 
            WHERE a.abs = $1
            LIMIT 5;
        "#;
        let rows = self.0.query(SQL, &[&abs]).await?;
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
                e.obs           as obs,
                t.next          as abs,
                a.equity        as equity,
                t.dx            as density,
                a.centrality    as centrality
            FROM transitions t
            JOIN abstraction a ON a.abs = t.next
            CROSS JOIN LATERAL (
                SELECT obs 
                FROM encoder e2
                WHERE e2.abs = t.next
                LIMIT 1
            ) e
            WHERE t.prev = $1
            ORDER BY t.dx DESC;
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

impl From<Client> for API {
    fn from(client: Client) -> Self {
        Self(Arc::new(client))
    }
}
