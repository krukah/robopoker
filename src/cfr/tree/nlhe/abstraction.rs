#![allow(unused)]

/// the result of the final abstraction
struct Bucket;

/// ordinal ranking of all possible ( 52 nCk 2 ) hole cards. maps to probability of beating a randomly dealt villain card. effectively [0..1325] <=> [0, 1]
struct Equity;

/// public information i.e. board cards
struct Public;

/// private information i.e. hole cards
struct Private;

/// perfect recall of past public and private information
struct History;

/// distribution of equity uniformly sampled over unknown villain and board cards. elements of the EMD metric space
struct Potential;

/// deterministic outcome of deck draw
struct Runout;

trait Abstraction {
    /// top-level function that maps a perfect recall history to an abstracted bucket
    fn bucket(history: History) -> Bucket;

    /// expected hand strength of a private hand given public board cards
    fn ehs(hand: Private, board: Public) -> Equity;

    /// earth mover's distance between two potential equity distributions
    fn emd(a: &Potential, b: &Potential) -> f32;

    /// integration over the equity of children potentials
    fn mean(potential: &Potential) -> Equity;
}
