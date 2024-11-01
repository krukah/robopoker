robopoker
===========================
[![license](https://img.shields.io/github/license/krukah/robopoker)](LICENSE)
[![build](https://github.com/krukah/robopoker/actions/workflows/rust.yml/badge.svg)](https://github.com/krukah/robopoker/actions/workflows/rust.yml)

`robopoker` is a Rust library to play, learn, analyze, track, and solve No-Limit Texas Hold'em.

The guiding philosophy of this package was to use very tangible abstractions to represent the rules, mechanics, and strategies of NLHE. Every module is modeled as closely as possible to its real-world analogue, while also utilizing clever representations and transformations to be as memory- and compute-efficient as possible. Focus on Rust idiomatics is also a goal, avoiding use of unsafe and exploiting strong typing.

# System Requirements

- Minimum 8GB RAM for shortdeck. 50GB+ recommended for full.
- Multi-core CPU. Clustering and CFR scale embarassingly.

# Modules

## `cards`

Core functionality for working with standard playing cards and Texas Hold'em rules:

- **Hand Evaluation**: Nanosecond 5-7 card hand strength evaluation using bit manipulation. Outperforms the popular Cactus Kev implementation
- **Equity Calculation**: Fast equity calculations between ranges of hands, supporting both exact enumeration and Monte Carlo simulation
- **Exhaustive Iteration**: Efficient iteration over cards, hands, decks, and private-public observations with lazy bitwise advancing
- **Distribution Analysis**: Tools for analyzing equity distributions and range vs range scenarios
- **Bijective Representations**: Multiple card representations (u8/u16/u32/u64) allowing for maximally efficient operations and transformations

## `gameplay`

A complete poker game engine implementation:

- **Standard Rules**: Full implementation of No-Limit Texas Hold'em rules and mechanics
- **Complex Showdowns**: Elegant handling and thorough testing of showdown edge cases like side pots, all-ins, dead cards, and multi-way ties
- **Flexible Payout Logic**: Configurable payout structures for different game formats
- **Decider Abstraction**: Generic trait system for implementing different player decision strategies
- **Functional Design**: Clean Node/Edge/Tree implementation for game state representation

## `clustering`

Advanced clustering capabilities for poker hand analysis:

- **Isomorphic Exhaustion**: Plays out *every one of 3.1T* possible situations by respecting symmetries and enforcing permutation invariance
- **Earth Mover's Distance (EMD)**: Implementation of EMD metric for comparing hand distributions over equity and hierarchical abstraction clusters
- **Hierarchical K-means**: Multi-level clustering algorithm for creating strategic abstractions from bottom-up distribution clustering 
- **Optimal Transport**: High level abstractions for calculating Wasserstein distance between two arbitrary distributions supported over a joint metric space
- **Persistence**: Efficient serialization and deserialization of clustering results to/from disk using Postgres binary formats

## `mccfr`

Monte Carlo Counterfactual Regret Minimization solver:

- **RPS Convergence**: Previously demonstrated convergence on Rock-Paper-Scissors as validation
- **External Sampling**: Implementation of external sampling MCCFR variant
- **Dynamic Tree Building**: On-the-fly game tree construction for memory efficiency
- **Linear Strategy Weighting**: Efficient strategy updates using iterative weighting and discount schemes
- **Persistence**: Efficient serialization and deserialization of blueprint results to/from disk using Postgres binary formats

## `api`

Coming soon. A distributed and scalable single-binary WebSocket-based HTTP server that allows players to play, learn, analyze, and track hands remotely.

## References

1. (2007). Regret Minimization in Games with Incomplete Information. [(NIPS)](https://papers.nips.cc/paper/3306-regret-minimization-in-games-with-incomplete-information)
2. (2015). Discretization of Continuous Action Spaces in Extensive-Form Games. [(AAMAS)](http://www.cs.cmu.edu/~sandholm/discretization.aamas15.fromACM.pdf)
3. (2014). Potential-Aware Imperfect-Recall Abstraction with Earth Mover's Distance in Imperfect-Information Games. [(AAAI)](http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf)
4. (2019). Superhuman AI for multiplayer poker. [(Science)](https://science.sciencemag.org/content/early/2019/07/10/science.aay2400)
5. (2019). Solving Imperfect-Information Games via Discounted Regret Minimization. [(AAAI)](https://arxiv.org/pdf/1809.04040.pdf)
6. (2018). Depth-Limited Solving for Imperfect-Information Games. [(NeurIPS)](https://arxiv.org/pdf/1805.08195.pdf)
7. (2017). Safe and Nested Subgame Solving for Imperfect-Information Games. [(NIPS)](https://www.cs.cmu.edu/~noamb/papers/17-NIPS-Safe.pdf)
8. (2017). Reduced Space and Faster Convergence in Imperfect-Information Games via Pruning. [(ICML)](http://www.cs.cmu.edu/~sandholm/reducedSpace.icml17.pdf)
9. (2015). Regret-Based Pruning in Extensive-Form Games. [(NIPS)](http://www.cs.cmu.edu/~sandholm/regret-basedPruning.nips15.withAppendix.pdf)
10. (2014). Potential-Aware Imperfect-Recall Abstraction with Earth Mover's Distance in Imperfect-Information Games. [(AAAI)](http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf)
11. (2013). Action Translation in Extensive-Form Games with Large Action Spaces: Axioms, Paradoxes, and the Pseudo-Harmonic Mapping. [(IJCAI)](http://www.cs.cmu.edu/~sandholm/reverse%20mapping.ijcai13.pdf)
12. (2013). A Fast and Optimal Hand Isomorphism Algorithm. [(AAAI)](https://www.cs.cmu.edu/~waugh/publications/isomorphism13.pdf)
