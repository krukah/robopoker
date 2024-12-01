use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Query {
    Neighborhood { abstraction: String },
    Memberships { abstraction: String },
    Abstraction { observation: String },
    AbsDistance { obs1: String, obs2: String },
    ObsDistance { obs1: String, obs2: String },
}
