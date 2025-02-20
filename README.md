robopoker
===========================
[![license](https://img.shields.io/github/license/krukah/robopoker)](LICENSE)
[![build](https://github.com/krukah/robopoker/actions/workflows/rust.yml/badge.svg)](https://github.com/krukah/robopoker/actions/workflows/rust.yml)

`robopoker` is a Rust library to play, learn, analyze, track, and solve No-Limit Texas Hold'em.

# Overview

This started as a simple Rust project before evolving into a state-of-the-art poker solver and analysis tool seeking functional parity with Pluribus<sup>1</sup>, the first superhuman agent in multiplayer No Limit Texas Hold'em.
<table align="center">
<tr>
<td align="center">
    <img src="https://github.com/user-attachments/assets/5118eba3-3d64-42f8-ac07-5c83ff733439" height="200" alt="Training Progress"/>
    <br>
    <em>Monte Carlo Tree Search</em>
</td>
<td align="center">
    <img src="https://github.com/user-attachments/assets/90b491df-9482-483e-9475-4360f5a17add" height="200" alt="Strategy Growth"/>
    <br>
    <em>Equity Distributions</em>
</td>
</tr>
</table>

The guiding philosophy of this crate is to use very precise struct and trait abstractions to represent the rules, mechanics, and strategies of NLHE. Every module is modeled as closely as possible to its real-world analogue, while also utilizing clever representations and transformations to be as memory- and compute-efficient as possible. We lean heavily into idiomatic Rust by using lazy functional patterns, efficient data structure representations, infallible type conversions, thread-safe multi-processing, and strictly safe code.

The intended use case is a one-time resource-intensive training run that will save information abstractions, k-means clusters, distance metrics, and blueprint profiles to disk for use in later runs or analyses. To generate these datasets under arbitrary parametrization, the program will iterate through the following steps:

1. For each layer of hierarchical abstraction (`preflop`, `flop`, `turn`, `river`):
   - Generate isomorphic hand clusters by exhaustively iterating through strategically equivalent situations
   - Initialize k-means centroids using k-means++ seeding over abstract distribution space<sup>2</sup>
   - Run hierarchical k-means clustering to group hands into strategically similar situations
   - Calculate Earth Mover's Distance metrics via optimal transport<sup>5</sup> between all cluster pairs
   - Save abstraction results and distance metrics to disk

2. Run iterative Monte Carlo CFR training<sup>3</sup>:
   - Initialize regret tables and strategy profiles
   - Sample game trajectories using external sampling MCCFR
   - Update regret values and compute counterfactual values
   - Accumulate strategy updates with linear weighting
   - Periodically checkpoint blueprint strategy to disk
   - Continue until convergence criteria met

3. Perform real-time search during gameplay (in progress):
   - Load pre-computed abstractions and blueprint strategy
   - Use depth-limited subgame solving with blueprint as prior
   - Dynamically build local game trees
   - Run targeted Monte Carlo rollouts
   - Return optimal actions within time constraints

# System Requirements

The abstraction and counterfactual regret minimization algorithms are quite resource intensive.
- Hierarchical k-means requires holding all strategically isomorphic observations at a given street, as well as their projected distributions onto their future streets.
- Monte Carlo CFR requires sampling game trees with full game state information and accumulating regret and policy information

| Street     | Abstraction Size  | Metric Size |
|------------|-------------------|-------------|
| Preflop    |          4 KB     | 301 KB      |
| Flop       |         32 MB     | 175 KB      |
| Turn       |        347 MB     | 175 KB      |
| River      |       3.02 GB     | -           | 

# Modules

## `cards`

Core functionality for working with standard playing cards and Texas Hold'em rules:

- **Hand Evaluation**: Nanosecond hand strength calculation using lazy evaluation; fastest open-source hand evaluation algorithm; benchmarks outperform the popular Cactus Kev implementation
- **Equity Calculation**: Fast equity calculations between ranges of hands, supporting both exact enumeration and Monte Carlo simulation
- **Exhaustive Iteration**: Efficient iteration over cards, hands, decks, and private-public observations with lazy bitwise advancing
- **Distribution Analysis**: Tools for analyzing equity distributions and range vs range scenarios
- **Short Deck Support**: Full support for 36-card short deck variant with adjusted hand rankings and iterators
- **Bijective Representations**: Multiple card representations `(u8/u16/u32/u64)` allow for maximally efficient operations and transformations

## `gameplay`

A complete poker game engine implementation:

- **Standard Rules**: Full implementation of No-Limit Texas Hold'em rules and mechanics
- **Complex Showdowns**: Elegant handling and thorough testing of showdown edge cases like side pots, all-ins, dead cards, and multi-way ties
- **Flexible Payout Logic**: Configurable payout structures for different game formats
- **Decider Abstraction**: Generic trait system for implementing different player decision strategies
- **Functional Design**: Clean Node/Edge/Tree implementation for game state representation

## `clustering`

Advanced clustering capabilities for poker hand analysis:

- **Isomorphic Exhaustion**: Plays out *every one of 3.1T* possible situations by respecting symmetries and enforcing permutation invariance<sup>4</sup>
- **Earth Mover's Distance (EMD)**: Implementation of EMD metric for comparing hand distributions over equity and hierarchical abstraction clusters
- **Hierarchical K-means**: Multi-level clustering algorithm for creating strategic abstractions from bottom-up distribution clustering 
- **Optimal Transport**: High level abstractions for calculating Wasserstein distance between two arbitrary distributions supported over a joint metric space
- **Persistence**: Efficient serialization and deserialization of clustering results to/from disk using Postgres binary formats

## `mccfr`

Monte Carlo Counterfactual Regret Minimization solver:

- **Blueprint Convergence**: Previously demonstrated convergence on Rock-Paper-Scissors as validation
- **External Sampling**: Implementation of external sampling MCCFR variant
- **Dynamic Tree Building**: On-the-fly game tree construction for memory efficiency
- **Linear Strategy Weighting**: Efficient strategy updates using iterative weighting and discount schemes<sup>5</sup>
- **Persistence**: Efficient serialization and deserialization of blueprint results to/from disk using Postgres binary formats

## `analysis`

Tools for analyzing and querying results yielded from our training pipeline using PostgreSQL.

- **Data Upload**: Copies Postgres binary files into a database with extensive indexing for efficient lookups.
- **SQL Optimization**: Enables querying all isomorphisms, abstractions, EMD metrics, potential distributions, and blueprint strategies learned during prior training steps.
- **CLI Tool**: A command-line interface to perform basic queries, such as equity and distance calculations.
- **Server Struct**: Actix web instance serving HTTP requests to support an analysis frontend client (coming soon).

# References

1. (2019). Superhuman AI for multiplayer poker. [(Science)](https://science.sciencemag.org/content/early/2019/07/10/science.aay2400)
2. (2014). Potential-Aware Imperfect-Recall Abstraction with Earth Mover's Distance in Imperfect-Information Games. [(AAAI)](http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf)
3. (2007). Regret Minimization in Games with Incomplete Information. [(NIPS)](https://papers.nips.cc/paper/3306-regret-minimization-in-games-with-incomplete-information)
4. (2013). A Fast and Optimal Hand Isomorphism Algorithm. [(AAAI)](https://www.cs.cmu.edu/~waugh/publications/isomorphism13.pdf)
5. (2018). Near-linear time approximation algorithms for optimal transport via Sinkhorn iteration. [(NIPS)](https://arxiv.org/abs/1705.09634)
6. (2019). Solving Imperfect-Information Games via Discounted Regret Minimization. [(AAAI)](https://arxiv.org/pdf/1809.04040.pdf)
7. (2013). Action Translation in Extensive-Form Games with Large Action Spaces: Axioms, Paradoxes, and the Pseudo-Harmonic Mapping. [(IJCAI)](http://www.cs.cmu.edu/~sandholm/reverse%20mapping.ijcai13.pdf)
8. (2015). Discretization of Continuous Action Spaces in Extensive-Form Games. [(AAMAS)](http://www.cs.cmu.edu/~sandholm/discretization.aamas15.fromACM.pdf)
9. (2015). Regret-Based Pruning in Extensive-Form Games. [(NIPS)](http://www.cs.cmu.edu/~sandholm/regret-basedPruning.nips15.withAppendix.pdf)
10. (2018). Depth-Limited Solving for Imperfect-Information Games. [(NeurIPS)](https://arxiv.org/pdf/1805.08195.pdf)
11. (2017). Reduced Space and Faster Convergence in Imperfect-Information Games via Pruning. [(ICML)](http://www.cs.cmu.edu/~sandholm/reducedSpace.icml17.pdf)
12. (2017). Safe and Nested Subgame Solving for Imperfect-Information Games. [(NIPS)](https://www.cs.cmu.edu/~noamb/papers/17-NIPS-Safe.pdf)
