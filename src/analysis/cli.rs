use super::analysis::Analysis;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
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
            let ref mut input = String::new();
            print!("> ");
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
        let args = input.split_whitespace().collect::<Vec<&str>>();
        match args.get(0).map(|s| *s) {
            Some("cluster") => {
                println!(
                    "cluster: {}",
                    args.get(1)
                        .map(|obs| Observation::try_from(*obs))
                        .transpose()?
                        .map(|obs| self.0.abstraction(obs))
                        .unwrap()
                        .await?
                )
            }
            Some("neighbors") => {
                println!(
                    "nearest neighbors: {}",
                    args.get(1)
                        .map(|abs| Abstraction::try_from(*abs))
                        .transpose()?
                        .map(|abs| self.0.neighborhood(abs))
                        .unwrap()
                        .await?
                        .iter()
                        .enumerate()
                        .map(|(i, abs)| format!("{:>2}. {}", i + 1, abs))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
            Some("constituents") => {
                println!(
                    "constituents: {}",
                    args.get(1)
                        .map(|abs| Abstraction::try_from(*abs))
                        .transpose()?
                        .map(|abs| self.0.membership(abs))
                        .unwrap()
                        .await?
                        .iter()
                        .enumerate()
                        .map(|(i, c)| format!("{}. {}", i + 1, c))
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            }
            Some("abs-distance") => {
                println!(
                    "abstraction distance: {:.4}",
                    args.get(1)
                        .map(|obs1| Observation::try_from(*obs1))
                        .transpose()?
                        .and_then(|obs1| args
                            .get(2)
                            .map(|obs2| Observation::try_from(*obs2))
                            .transpose()
                            .unwrap()
                            .map(|obs2| self.0.abs_distance(obs1, obs2)))
                        .unwrap()
                        .await?
                );
            }
            Some("obs-distance") => {
                println!(
                    "observation distance: {:.4}",
                    args.get(1)
                        .map(|obs1| Observation::try_from(*obs1))
                        .transpose()
                        .unwrap()
                        .map(|obs1| args
                            .get(2)
                            .map(|obs2| Observation::try_from(*obs2))
                            .transpose()
                            .unwrap()
                            .map(|obs2| self.0.obs_distance(obs1, obs2)))
                        .unwrap()
                        .unwrap()
                        .await?
                );
            }
            Some("help") => {
                println!("available commands:");
                println!("  cluster <observation>              - get cluster for observation");
                println!("  neighbors <abstraction> [k]        - get k nearest neighbors");
                println!("  constituents <abstraction>         - get constituents of abstraction");
                println!("  abs-distance <obs1> <obs2>         - get abstraction distance");
                println!("  obs-distance <obs1> <obs2>         - get observation distance");
                println!("  help                               - show this help");
                println!("  exit/quit                          - exit the program");
            }
            _ => println!("unknown command. type 'help' for usage."),
        }
        Ok(())
    }
}
