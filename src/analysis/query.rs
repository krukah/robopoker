use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Query {
    #[command(
        about = "Find the abstractions of any given observation",
        alias = "abs"
    )]
    Abstraction {
        #[arg(required = true)]
        observation: String,
    },
    #[command(
        about = "Find the equivalences of any given observation",
        alias = "eqv"
    )]
    Equivalents {
        #[arg(required = true)]
        observation: String,
    },
    #[command(
        about = "Find the observations in any given abstraction",
        alias = "mem"
    )]
    Membership {
        #[arg(required = true)]
        abstraction: String,
    },
    #[command(
        about = "Find the neighborhood of any given abstraction",
        alias = "nbr"
    )]
    Neighborhood {
        #[arg(required = true)]
        abstraction: String,
    },
    #[command(
        about = "Find the abstraction distance between two observations",
        alias = "dab"
    )]
    AbsDistance {
        #[arg(required = true)]
        obs1: String,
        #[arg(required = true)]
        obs2: String,
    },
    #[command(
        about = "Find the observation distance between two observations",
        alias = "dob"
    )]
    ObsDistance {
        #[arg(required = true)]
        obs1: String,
        #[arg(required = true)]
        obs2: String,
    },
}
