use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Query {
    Cluster { observation: String },
    Neighbors { abstraction: String },
    Constituents { abstraction: String },
    AbsDistance { obs1: String, obs2: String },
    ObsDistance { obs1: String, obs2: String },
}
