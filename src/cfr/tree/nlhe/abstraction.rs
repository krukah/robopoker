#![allow(unused)]

use std::collections::HashMap;

use crate::{
    cfr::tree::rps::player::Player, evaluation::strength::Strength, gameplay::action::Action,
};

/// the result of the final abstraction
struct Bucket;

/// ordinal ranking of all possible ( 52 nCk 2 ) hole cards. maps to probability of beating a randomly dealt villain card. effectively [0..1325] <=> [0, 1]
struct Equity;

/// perfect recall of past public and private information
struct PerfectRecall;

/// probably just a f32 at end of day, but generalizaton of distance
struct Distance;

/// distribution of equity uniformly sampled over unknown villain and board cards. elements of the EMD metric space
struct Potential;

/// smallest disrete unit of game state machine
struct Rotation(HashMap<Player, Strength>); // absence maybe doesnt represent folding well

trait Abstraction {
    /// top-level function that maps a perfect recall history to an abstracted bucket
    fn bucket(history: PerfectRecall) -> Bucket;
    /// expected hand strength of a private hand given public board cards
    fn ehs(bucket: Bucket) -> Equity;
    /// integration over the equity of children potentials
    fn mean(potential: &Potential) -> Equity;
    /// earth mover's distance between two potential equity distributions
    fn emd(a: &Potential, b: &Potential) -> Distance;
}

struct NolimData {
    runout: Rotation,
    player: Player,
}

// decide if how you wanna mke all your immutable self methods
// like, you could on one extreme
// store nothing but &'grow NullInfoSet
// and allow for Node::* traversals to piece together Node implementations
// or store everything you need to reason locally about the tree, locally
// player, bucket, payoff, possible actions

struct NolimEdge {
    action: Action,
}

// need to replay actions for perfect recall
struct MinimalData {}
struct MinimalEdge {}
