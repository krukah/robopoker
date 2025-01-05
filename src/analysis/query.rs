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
        target: String,
    },

    #[command(
        about = "Find the distance between two targets (obs~obs or abs~abs)",
        alias = "dst"
    )]
    Distance {
        #[arg(required = true)]
        target1: String,
        #[arg(required = true)]
        target2: String,
    },

    #[command(
        about = "Find observations belonging to the same cluster of any given observation or abstraction",
        alias = "sim"
    )]
    Similar {
        #[arg(required = true)]
        target: String,
    },

    #[command(
        about = "Find abstractions in the neighborhood of any given observation or abstraction",
        alias = "nbr"
    )]
    Nearby {
        #[arg(required = true)]
        target: String,
    },

    #[command(
        about = "Find the equity of any given observation or abstraction",
        alias = "eqt"
    )]
    Equity {
        #[arg(required = true)]
        target: String,
    },

    #[command(
        about = "Find the population of any given observation or abstraction",
        alias = "pop"
    )]
    Population {
        #[arg(required = true)]
        target: String,
    },

    #[command(
        about = "Find the centrality of any given observation or abstraction",
        alias = "ctr"
    )]
    Centrality {
        #[arg(required = true)]
        target: String,
    },

    #[command(
        about = "Find the histogram of any given observation or abstraction",
        alias = "hst"
    )]
    Composition {
        #[arg(required = true)]
        target: String,
    },
}
