use super::analysis::Analysis;
use super::query::Query;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::strength::Strength;
use crate::clustering::abstraction::Abstraction;
use clap::Parser;
use std::io::Write;
use tokio_postgres::Config;
use tokio_postgres::NoTls;

pub struct CLI(Analysis);

impl CLI {
    pub async fn new() -> Self {
        log::info!("connecting to db");
        let (client, connection) = Config::default()
            .host("localhost")
            .port(5432)
            .dbname("robopoker")
            .connect(NoTls)
            .await
            .expect("db connection");
        tokio::spawn(connection);
        Self(Analysis::from(client))
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
            Query::Abstraction { observation } => {
                let obs = Observation::try_from(observation.as_str())?;
                let abstraction = self.0.encode(obs).await?;
                Ok(println!("abstraction: {}", abstraction))
            }
            Query::AbsDistance { obs1, obs2 } => {
                let (o1, o2) = Observation::try_from(obs1.as_str())
                    .and_then(|o1| Observation::try_from(obs2.as_str()).map(|o2| (o1, o2)))?;
                let distance = self.0.abs_distance(o1, o2).await?;
                Ok(println!("abstraction distance: {:.4}", distance))
            }
            Query::ObsDistance { obs1, obs2 } => {
                let (o1, o2) = Observation::try_from(obs1.as_str())
                    .and_then(|o1| Observation::try_from(obs2.as_str()).map(|o2| (o1, o2)))?;
                let distance = self.0.obs_distance(o1, o2).await?;
                Ok(println!("observation distance: {:.4}", distance))
            }
            Query::Memberships { abstraction } => {
                let abs = Abstraction::try_from(abstraction.as_str())?;
                let memberships = self
                    .0
                    .membership(abs)
                    .await?
                    .iter()
                    .enumerate()
                    .map(|(i, obs)| (i + 1, obs, Strength::from(Hand::from(*obs))))
                    .map(|(i, o, s)| format!("\n{:>2}. {:<18} {}", i, o, s))
                    .collect::<Vec<String>>()
                    .join("");
                Ok(println!("membership: {}", memberships))
            }
            Query::Neighborhood { abstraction } => {
                let abs = Abstraction::try_from(abstraction.as_str())?;
                let neighborhood = self
                    .0
                    .vicinity(abs)
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
