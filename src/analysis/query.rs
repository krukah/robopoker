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
        about = "Find the histogram of any given observation or abstraction",
        alias = "hst"
    )]
    Composition {
        #[arg(required = true)]
        target: String,
    },

    #[command(about = "Convert an integer to a Path representation", alias = "pth")]
    Path {
        #[arg(required = true)]
        value: i64,
    },

    #[command(about = "Convert an integer to an Edge representation", alias = "edg")]
    Edge {
        #[arg(required = true)]
        value: u8,
    },

    #[command(
        about = "Convert an integer to an Abstraction representation",
        alias = "abi"
    )]
    AbsFromInt {
        #[arg(required = true)]
        value: i64,
    },

    #[command(
        about = "Convert an integer to an Observation representation",
        alias = "obi"
    )]
    ObsFromInt {
        #[arg(required = true)]
        value: i64,
    },

    #[command(
        about = "Convert an integer to an Isomorphism representation",
        alias = "iso"
    )]
    Isomorphism {
        #[arg(required = true)]
        value: i64,
    },
}
