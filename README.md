# Overview

`robopoker` is a library to play, learn, analyze, track, and solve No-Limit Texas Hold'em. The guiding philosophy of this package was to use very tangible abstractions to represent the rules, mechanics, and strategies of NLHE. Every module is modeled as closely as possible to its real-world analogue, while also utilizing clever representations and transformations to be as memory- and compute-efficient as possible.
# Modules

## `cards`
This module offers basic data structures and algorithms for a standard 52-card French deck. We define cards at a very low level while still benefiting from high level abstractions, due to Rust's support of type-safe transformations between various representations of isomorphic data types.

For example, the `Rank`, `Suit`, `Card`, `Hand`, and `Deck` types encapsulate all the high-level features you expect: order, equality, shuffling, dealing, and hand evaluation are all derived from precise representations and efficient manipulations of memory that enable high-performance while preserving zero-cost abstractions. Isomorphic representations of these data structures across `u8, u16, u32, u64` types provide flexibility that is useful for random deck generation, hand evaluation, set-theoretic operations, and (de)serialization across network boundaries.

`Hand` as `u64` ends up allowing for very efficient bit manipulations, since all unordered subsets of cards are uniquely represented. Utility methods for iterating, drawing, inserting, union, and counting all emerge from natural bitwise equivalents.

`Evaluator` provides ranking poker hands while avoiding the high cost of explosive combinatorics. We lean heavily into idiomatic Rust by using **lazy evaluation over the `Option<HandStrength>` monad** to implement priority search of card rankings. In the future, it may be worth considering the time-space tradeoff between the current _lazy_ implementation and a possible _eager_ one to do lookups. 

## `clustering`
The literature [2, 3] suggests hierarchical k-means clustering for *information absraction*. This is the dimensionality reduction that we apply by grouping similar observed chance outcomes while being **completely agnostic to any strategy.** The main idea is to recursively, from the outer (river) up, decompose observations into the space of their lower-level abstractions. We build up each (non-river) layer as a distribution of distributions. By equipping the dimensionally-sparce space of information abstractions with an Earth mover's distance metric, we can learn clusters via k-means.

The outer (river) layer is clustered uniquely. Each River runout of 2 private and 5 public cards is evaluated against all possible 2-card villain hands to compute equity. Equities, measured as float percentage, are converted into percentile buckets, yielding the only variant of `Abstraction` that is *****naturally equipped with a distance metric*****. Perhaps a 7-card lookup table or a stochastic villain sampling would be orders of magnitude more performant, but given the amortized one-time cost of these calcuations, we can trade off convenience for accuracy. 

Ultimately, we are brute forcing this step by iterating over **the entire space of 3.1B distinguishable game states.** An impending optimization will be to reduce the space of `Observation` by enforcing strategic isomorphism. Even at this first preprocessing step, scale becomes a massive bottleneck. We benchmarked three persistence mechanisms for equity/abstraction calculations on a *(M1 CPU; 16GB RAM; 2TB DISK)* machine:

- `900 obs,abs / s: HashMap<Observation, Abstraction>` but crashes upon consuming ~80GB of swap space. Could try running on a rammy EC2.
- `550 obs,abs / s: Postgres` but is a very parallelizable approach. Ultimately we need to persist results to disk, so we might as well iterate over async queries across different process / thread boundaries.
- `200 obs,abs / s: Redis` but might perform better on a machine with more RAM. I was suprised at this being slower IO than Postgres, although perhaps we could optimize some configuration params. 




## `play`
This module offers an out-of-the-box way to play NLHE without crossing any network boundaries. The hierarchy of data structures follows from a tree-based abstraction of the game.
- `Seat` encapsulates public information related to a player.
- `Action` is an enum that transitions the state of the game. Chance actions (card draws) and choice actions (player decisions) are both edges between nodes of possible game states.
- `Rotation` is a path-invariant representation of the current game state. It is a minimal data structure, with most relevant information exposed via pure methods.
- `Game` is a path-aware representation of the current game hand. It is the smallest solvable subset of NLHE, and a direct representation of the game tree abstraction that is used by solvers and players alike.
- `Table` is a mechanism to advance that game state encapsulated by `Game` and `Rotation` according to the rules of the game, assuming input comes from a synchronized subroutine. For games to be played across a network boundary, custom implementation of the game loop must be used to account for distributed game state and fault tolerance.

## `cfrm`
Traits and implementation of a counter-factual regret minimization engine, automated range generation, and parametrized reinforcement learning. Currently, we implement a variation of CFR+ which uses positive regrets and updates strategies between players in distinct iteration steps. Working on a full tree builder and solver for Kuhn poker before moving on to large game abstractions that are actually useful in Limit Hold'em, and then No-Limit Hold'em.



## `api`
Coming soon. A distributed and scalable single-binary WebSocket-based HTTP server that allows players to play, learn, analyze, and track hands remotely.

[1] Regret Minimization in Games with Incomplete Information. Advances in Neural Information Processing Systems, 20. (https://proceedings.neurips.cc/paper/2007/file/08d98638c6fcd194a4b1e6992063e944-Paper.pdf) In NIPS.
[2] Discretization of Continuous Action Spaces in Extensive-Form Games. (http://www.cs.cmu.edu/~sandholm/discretization.aamas15.fromACM.pdf) In AAMAS.
[3] Potential-Aware Imperfect-Recall Abstraction with Earth Mover’s Distance in Imperfect-Information Games. (http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf) In AAAI.
