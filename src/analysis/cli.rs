use super::analysis::Analysis;
use super::query::Query;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::strength::Strength;
use crate::clustering::abstraction::Abstraction;
use crate::Pipe;
use clap::Parser;
use std::io::Write;
use tokio_postgres::Config;
use tokio_postgres::NoTls;

pub struct CLI(Analysis);

impl CLI {
    pub async fn new() -> Self {
        let (client, connection) = Config::default()
            .host("localhost")
            .dbname("robopoker")
            .connect(NoTls)
            .await
            .expect("db connection");
        tokio::spawn(connection);
        Self(Analysis::new(client))
    }

    pub async fn run(&self) -> () {
        loop {
            log::info!("launching analysis");
            print!("> ");
            let ref mut input = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(input).unwrap();
            match input.trim() {
                "quit" => break,
                "exit" => break,
                _ => match self.handle(input).await {
                    Err(e) => eprintln!("handle error: {}", e),
                    Ok(_) => continue,
                },
            }
        }
    }

    async fn handle(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        match Query::try_parse_from(std::iter::once("> ").chain(input.split_whitespace()))? {
            Query::Abstraction { observation } => Ok(println!(
                "abstraction: {}",
                Observation::try_from(observation.as_str())
                    .map_err(|e| format!("invalid observation: {}", e))?
                    .pipe(|obs| self.0.abstractable(obs))
                    .await?
            )),
            Query::Memberships { abstraction } => Ok(println!(
                "membership: \n{}",
                Abstraction::try_from(abstraction.as_str())
                    .map_err(|e| format!("invalid abstraction: {}", e))?
                    .pipe(|abs| self.0.constituents(abs))
                    .await?
                    .iter()
                    .enumerate()
                    .map(|(i, obs)| format!(
                        "{:>2}. {:<18} {}",
                        i + 1,
                        obs,
                        Strength::from(Hand::from(*obs))
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            )),
            Query::Neighborhood { abstraction } => Ok(println!(
                "neighborhood: \n{}",
                Abstraction::try_from(abstraction.as_str())
                    .map_err(|e| format!("invalid abstraction: {}", e))?
                    .pipe(|abs| self.0.neighborhood(abs))
                    .await?
                    .iter()
                    .enumerate()
                    .map(|(i, (abs, dist))| format!("{:>2}. {} ({:.4})", i + 1, abs, dist))
                    .collect::<Vec<String>>()
                    .join("\n")
            )),
            Query::AbsDistance { obs1, obs2 } => Ok(println!(
                "abstraction distance: {:.4}",
                Observation::try_from(obs1.as_str())
                    .and_then(|o1| Observation::try_from(obs2.as_str()).map(|o2| (o1, o2)))
                    .map_err(|e| format!("invalid observation: {}", e))?
                    .pipe(|(o1, o2)| self.0.abs_distance(o1, o2))
                    .await?
            )),
            Query::ObsDistance { obs1, obs2 } => Ok(println!(
                "observation distance: {:.4}",
                Observation::try_from(obs1.as_str())
                    .and_then(|o1| Observation::try_from(obs2.as_str()).map(|o2| (o1, o2)))
                    .map_err(|e| format!("invalid observation: {}", e))?
                    .pipe(|(o1, o2)| self.0.obs_distance(o1, o2))
                    .await?
            )),
        }
    }
}
