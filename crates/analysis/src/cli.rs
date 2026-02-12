//! Interactive CLI for poker analysis.
//!
//! Provides commands for type conversions and database queries.
use crate::*;
use clap::Parser;
use rbp_cards::*;
use rbp_gameplay::*;
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
        let cli = Self(API::from(rbp_database::db().await));
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
            Query::Path { value } => {
                let path = Path::from(value);
                println!("Path({})", value);
                println!("  Display:  {}", path);
                println!("  Length:   {}", path.length());
                println!("  Aggro:    {}", path.aggression());
                println!("  Edges:    {:?}", Vec::<Edge>::from(path));
                Ok(())
            }
            Query::Edge { value } => {
                let edge = Edge::from(value);
                println!("Edge({})", value);
                println!("  Display:  {}", edge);
                println!("  Is choice: {}", edge.is_choice());
                println!("  Is aggro:  {}", edge.is_aggro());
                Ok(())
            }
            Query::AbsFromInt { value } => {
                let abs = Abstraction::from(value);
                println!("Abstraction({})", value);
                println!("  Display:  {}", abs);
                println!("  Street:   {}", abs.street());
                println!("  Index:    {}", abs.index());
                Ok(())
            }
            Query::ObsFromInt { value } => {
                println!("Observation({})", value);
                match std::panic::catch_unwind(|| Observation::from(value)) {
                    Ok(obs) => {
                        println!("  Display:  {}", obs);
                        println!("  Street:   {}", obs.street());
                        println!("  i64:      {}", i64::from(obs));
                        Ok(())
                    }
                    Err(_) => {
                        println!("  Error: Invalid observation encoding (assertions failed)");
                        println!("  Note: Observations require valid poker hand representations");
                        Ok(())
                    }
                }
            }
            Query::Isomorphism { value } => {
                println!("Isomorphism({})", value);
                match std::panic::catch_unwind(|| {
                    let iso = Isomorphism::from(value);
                    let obs = Observation::from(iso);
                    (iso, obs)
                }) {
                    Ok((iso, obs)) => {
                        println!("  Observation: {}", obs);
                        println!("  Street:      {}", obs.street());
                        println!("  i64:         {}", i64::from(iso));
                        Ok(())
                    }
                    Err(_) => {
                        println!("  Error: Invalid isomorphism encoding (assertions failed)");
                        println!("  Note: Isomorphisms require valid poker hand representations");
                        Ok(())
                    }
                }
            }
        }
    }
}
