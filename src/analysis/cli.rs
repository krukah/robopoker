use super::api::API;
use super::query::Query;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::strength::Strength;
use crate::clustering::abstraction::Abstraction;
use clap::Parser;
use std::io::Write;

pub struct CLI(API);

impl CLI {
    pub async fn new() -> Self {
        log::info!("connecting to db (CLI)");
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
        Self(API::from(client))
    }

    pub async fn run(&self) -> () {
        log::info!("entering analysis");
        loop {
            print!("> ");
            let ref mut input = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(input).unwrap();
            match input.trim() {
                "quit" => break,
                "exit" => break,
                _ => match self.handle(input).await {
                    Ok(_) => continue,
                    Err(e) => eprintln!("{}", e),
                },
            }
        }
    }

    async fn handle(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        match Query::try_parse_from(std::iter::once("> ").chain(input.split_whitespace()))? {
            Query::AbsDistance { obs1, obs2 } => {
                let o1 = Observation::try_from(obs1.as_str())?;
                let o2 = Observation::try_from(obs2.as_str())?;
                let distance = self.0.abs_distance(o1, o2).await?;
                Ok(println!("abstraction distance: {:.4}", distance))
            }

            Query::ObsDistance { obs1, obs2 } => {
                let o1 = Observation::try_from(obs1.as_str())?;
                let o2 = Observation::try_from(obs2.as_str())?;
                let distance = self.0.obs_distance(o1, o2).await?;
                Ok(println!("observation distance: {:.4}", distance))
            }

            Query::Abstraction { observation } => {
                let obs = Observation::try_from(observation.as_str())?;
                let abstraction = self.0.encode(obs).await?;
                Ok(println!("abstraction: {}", abstraction))
            }

            Query::Isomorphisms { observation } => {
                let obs = Observation::try_from(observation.as_str())?;
                let equivalents = self
                    .0
                    .isomorphisms(obs)
                    .await?
                    .iter()
                    .map(|o| format!("\n - {}", o))
                    .collect::<Vec<String>>()
                    .join("");
                Ok(println!("equivalents:\n{}", equivalents))
            }

            Query::Constituents { abstraction } => {
                let abs = Abstraction::try_from(abstraction.as_str())?;
                let memberships = self
                    .0
                    .constituents(abs)
                    .await?
                    .iter()
                    .enumerate()
                    .map(|(i, obs)| (i + 1, obs, Strength::from(Hand::from(*obs))))
                    .map(|(i, o, s)| format!("\n{:>2}. {:<18} {}", i, o, s))
                    .collect::<Vec<String>>()
                    .join("");
                Ok(println!("constituents: {}", memberships))
            }

            Query::Neighborhood { abstraction } => {
                let abs = Abstraction::try_from(abstraction.as_str())?;
                let neighborhood = self
                    .0
                    .neighborhood(abs)
                    .await?
                    .iter()
                    .enumerate()
                    .map(|(i, (abs, dist))| format!("\n{:>2}. {} ({:.4})", i + 1, abs, dist))
                    .collect::<Vec<String>>()
                    .join("");
                Ok(println!("neighborhood: {}", neighborhood))
            }
        }
    }
}
