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
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => continue,
                },
            }
        }
    }

    async fn handle(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        match Query::try_parse_from(std::iter::once("> ").chain(input.split_whitespace()))? {
            Query::Abstraction { target } => {
                let obs = Observation::try_from(target.as_str())?;
                let abstraction = self.0.abstraction(obs).await?;
                Ok(println!("abstraction: {}", abstraction))
            }

            Query::Distance { target1, target2 } => {
                if let (Ok(o1), Ok(o2)) = (
                    Observation::try_from(target1.as_str()),
                    Observation::try_from(target2.as_str()),
                ) {
                    return Ok(println!(
                        "observation distance: {:.4}",
                        self.0.obs_distance(o1, o2).await?
                    ));
                }
                if let (Ok(a1), Ok(a2)) = (
                    Abstraction::try_from(target1.as_str()),
                    Abstraction::try_from(target2.as_str()),
                ) {
                    return Ok(println!(
                        "abstraction distance: {:.4}",
                        self.0.abs_distance(a1, a2).await?
                    ));
                }
                Err("invalid distance targets".into())
            }

            Query::Similar { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    let members = self
                        .0
                        .obs_similar(obs)
                        .await?
                        .iter()
                        .map(|obs| (obs, Strength::from(Hand::from(*obs))))
                        .map(|(o, s)| format!("\n - {:<18} {}", o, s))
                        .collect::<Vec<String>>()
                        .join("");
                    return Ok(println!("similar observations: {}", members));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    let members = self
                        .0
                        .abs_similar(abs)
                        .await?
                        .iter()
                        .map(|obs| (obs, Strength::from(Hand::from(*obs))))
                        .map(|(o, s)| format!("\n - {:<18} {}", o, s))
                        .collect::<Vec<String>>()
                        .join("");
                    return Ok(println!("abstraction membership: {}", members));
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
                        .map(|(i, (abs, dist))| format!("\n{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("");
                    return Ok(println!("observation neighborhood: {}", neighborhood));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    let neighborhood = self
                        .0
                        .abs_nearby(abs)
                        .await?
                        .iter()
                        .enumerate()
                        .map(|(i, (abs, dist))| format!("\n{:>2}. {} ({:.4})", i + 1, abs, dist))
                        .collect::<Vec<String>>()
                        .join("");
                    return Ok(println!("abstraction neighborhood: {}", neighborhood));
                }
                Err("invalid neighborhood target".into())
            }

            Query::Equity { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!(
                        "observation equity: {:.4}",
                        self.0.obs_equity(obs).await?
                    ));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!(
                        "abstraction equity: {:.4}",
                        self.0.abs_equity(abs).await?
                    ));
                }
                Err("invalid equity target".into())
            }

            Query::Population { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!(
                        "observation population: {}",
                        self.0.obs_population(obs).await?
                    ));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!(
                        "abstraction population: {}",
                        self.0.abs_population(abs).await?
                    ));
                }
                Err("invalid population target".into())
            }

            Query::Centrality { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!(
                        "mean observation distance: {:.4}",
                        self.0.obs_centrality(obs).await?
                    ));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!(
                        "mean abstraction distance: {:.4}",
                        self.0.abs_centrality(abs).await?
                    ));
                }
                Err("invalid centrality target".into())
            }

            Query::Densities { target } => {
                if let Ok(obs) = Observation::try_from(target.as_str()) {
                    return Ok(println!(
                        "observation histogram:\n{:#?}",
                        self.0.obs_histogram(obs).await?.distribution()
                    ));
                }
                if let Ok(abs) = Abstraction::try_from(target.as_str()) {
                    return Ok(println!(
                        "abstraction histogram:\n{:#?}",
                        self.0.abs_histogram(abs).await?.distribution()
                    ));
                }
                Err("invalid histogram target".into())
            }
        }
    }
}
