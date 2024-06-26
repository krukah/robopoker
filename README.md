# Overview

`robopoker` is a library to play, learn, analyze, track, and solve No-Limit Texas Hold'em. The guiding philosophy of this package was to use very tangible abstractions to represent the rules, mechanics, and strategies of NLHE. Every module is modeled as closely as possible to its real-world analogue, while also utilizing clever representations and transformations to be as memory- and compute-efficient as possible.
# Modules

## `cards`
This module offers basic data structures and algorithms for a standard 52-card French deck. We define cards at a very low level while still benefiting from high level abstractions, due to Rust's support of type-safe transformations between various representations of isomorphic data types.

For example, the `Rank`, `Suit`, `Deck`, and `Card` types encapsulate all the high-level features you expect: order, equality, shuffling, dealing, and hand evaluation are all derived from precise representations and efficient manipulations of memory that enable high-performance while preserving zero-cost abstractions. Isomorphic representations of these data structures across `u8, u16, u32, u64` types provide flexibility that is useful for random deck generation, hand evaluation, set-theoretic operations, and (de)serialization across network boundaries.

## `evaluation`
This module offers efficient evaluation of traditional NLHE poker hands while avoiding the high cost of explosive combinatorics. We lean heavily into idiomatic Rust by using functional programming and lazy evaluation to search the space of hand strengths. In the future, it may be worth considering the time-space tradeoff between two possible methods of hand evaluation:
- a `LazyEvaluator` that does an ordered scan for the presence of a `StraightFlush, FourOfAKind, ...` in any given card set (50ns time, 0MB space)
- a `LookupEvaluator` that does a one-time calculation for all `n Choose k` card sets of interest and to statically generate a perfect hash lookup table (2ns time, 500MB space)

For now, we stick with the memory-less lazy evaluation, with flexibility to migrate to a lookup implementation if benchmarks suggest substantial performance improvements.

## `gameplay`
This module offers an out-of-the-box way to play NLHE without crossing any network boundaries. The hierarchy of data structures follows from a tree-based abstraction of the game.
- `Seat` encapsulates public information related to a player.
- `Action` is an enum that transitions the state of the game. Chance actions (card draws) and choice actions (player decisions) are both edges between nodes of possible game states.
- `Node` is a path-invariant representation of the current game state. It is a minimal data structure, with most relevant information exposed via pure methods.
- `Hand` is a path-aware representation of the current game hand. It is the smallest solvable subset of NLHE, and a direct representation of the game tree abstraction that is used by solvers and players alike.
- `Table` is a mechanism to advance that game state encapsulated by `Hand` and `Node` according to the rules of the game, assuming input comes from a synchronized subroutine. For games to be played across a network boundary, custom implementation of the game loop must be used to account for distributed game state and fault tolerance.

## `cfrm`
Traits and implementation of a counter-factual regret minimization engine, automated range generation, and parametrized reinforcement learning. Currently, we implement a variation of CFR+ which uses positive regrets and updates strategies between players in distinct iteration steps. Working on a full tree builder and solver for Kuhn poker before moving on to large game abstractions that are actually useful in Limit Hold'em, and then No-Limit Hold'em.



## `api`
Coming soon. A distributed and scalable single-binary WebSocket-based HTTP server that allows players to play, learn, analyze, and track hands remotely.

[1] Regret Minimization in Games with Incomplete Information. Advances in Neural Information Processing Systems, 20. Zinkevich, M., Bowling, M., Burch, N., Cao, Y., Johanson, M., Tamblyn, I., & Rocco, M. (2007).
