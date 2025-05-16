use super::api::API;
use super::query::Query;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::strength::Strength;
use crate::gameplay::abstraction::Abstraction;
use clap::Parser;
use std::io::Write;

pub struct CLI(API);

impl From<API> for CLI {
    fn from(api: API) -> Self {
        Self(api)
    }
}

impl CLI {
    pub async fn run() -> () {
        log::info!("entering analysis");
        let cli = Self(API::from(crate::db().await));
        loop {
            print!("> ");
            let ref mut input = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(input).unwrap();
            match input.trim() {
                "quit" => break,
                "exit" => break,
                _ => match cli.handle(input).await {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => continue,
                },
            }
        }
    }

    async fn handle(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        match Query::try_parse_from(std::iter::once("> ").chain(input.split_whitespace()))? {
            Query::Abstraction { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!("{}", self.0.obs_to_abs(obs).await?));
                }
                Err("invalid abstraction target".into())
            }

            Query::Distance { target1, target2 } => {
                if let (Ok(o1), Ok(o2)) = (
                    Observation::try_from(target1.as_str()),
                    Observation::try_from(target2.as_str()),
                ) {
                    return Ok(println!("{:.4}", self.0.obs_distance(o1, o2).await?));
                }
                if let (Ok(a1), Ok(a2)) = (
                    Abstraction::try_from(target1.as_str()),
                    Abstraction::try_from(target2.as_str()),
                ) {
                    return Ok(println!("{:.4}", self.0.abs_distance(a1, a2).await?));
                }
                Err("invalid distance targets".into())
            }

            Query::Equity { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!("{:.4}", self.0.obs_equity(obs).await?));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!("{:.4}", self.0.abs_equity(abs).await?));
                }
                Err("invalid equity target".into())
            }

            Query::Population { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!("{}", self.0.obs_population(obs).await?));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!("{}", self.0.abs_population(abs).await?));
                }
                Err("invalid population target".into())
            }

            Query::Centrality { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!("{:.4}", self.0.obs_centrality(obs).await?));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!("{:.4}", self.0.abs_centrality(abs).await?));
                }
                Err("invalid centrality target".into())
            }

            Query::Similar { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    let members = self
                        .0
                        .obs_similar(obs)
                        .await?
                        .iter()
                        .map(|obs| (obs, Strength::from(Hand::from(*obs))))
                        .map(|(o, s)| format!(" - {:<18} {}", o, s))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", members));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    let members = self
                        .0
                        .abs_similar(abs)
                        .await?
                        .iter()
                        .map(|obs| (obs, Strength::from(Hand::from(*obs))))
                        .map(|(o, s)| format!(" - {:<18} {}", o, s))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", members));
                }
                Err("invalid similarity target".into())
            }

            Query::Nearby { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    let neighborhood = self
                        .0
                        .obs_nearby(obs)
                        .await?
                        .iter()
                        .enumerate()
                        .map(|(i, (abs, dist))| format!("{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", neighborhood));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    let neighborhood = self
                        .0
                        .abs_nearby(abs)
                        .await?
                        .iter()
                        .enumerate()
                        .map(|(i, (abs, dist))| format!("{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", neighborhood));
                }
                Err("invalid neighborhood target".into())
            }

            Query::Composition { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    let distribution = self
                        .0
                        .obs_histogram(obs)
                        .await?
                        .distribution()
                        .iter()
                        .enumerate()
                        .map(|(i, (abs, dist))| format!("{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", distribution));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    let distribution = self
                        .0
                        .abs_histogram(abs)
                        .await?
                        .distribution()
                        .iter()
                        .enumerate()
                        .map(|(i, (abs, dist))| format!("{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("\n");
                    return Ok(println!("{}", distribution));
                }
                Err("invalid histogram target".into())
            }
        }
    }
}
