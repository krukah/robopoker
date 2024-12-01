use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Query {
    #[command(
        about = "find the abstractions of any given observation",
        alias = "abs"
    )]
    Abstraction {
        #[arg(required = true)]
        observation: String,
    },
    #[command(
        about = "find the observations in any given abstraction",
        alias = "mem"
    )]
    Memberships {
        #[arg(required = true)]
        abstraction: String,
    },
    #[command(
        about = "find the neighborhood of any given abstraction",
        alias = "nbr"
    )]
    Neighborhood {
        #[arg(required = true)]
        abstraction: String,
    },
    #[command(
        about = "find the abstraction distance between two observations",
        alias = "dab"
    )]
    AbsDistance {
        #[arg(required = true)]
        obs1: String,
        #[arg(required = true)]
        obs2: String,
    },
    #[command(
        about = "find the observation distance between two observations",
        alias = "dob"
    )]
    ObsDistance {
        #[arg(required = true)]
        obs1: String,
        #[arg(required = true)]
        obs2: String,
    },
}
