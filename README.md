# robopoker

[![license](https://img.shields.io/github/license/krukah/robopoker)](LICENSE)
[![build](https://github.com/krukah/robopoker/actions/workflows/ci.yml/badge.svg)](https://github.com/krukah/robopoker/actions/workflows/ci.yml)

`robopoker` is a Rust library and application suite to play, learn, analyze, track, and solve No-Limit Texas Hold'em.

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

The project consists of three main components:

1. **Training Pipeline**: A unified `trainer` binary that orchestrates clustering and MCCFR training, with PostgreSQL as the source of truth for all abstractions and strategies.

2. **Analysis Platform**: An HTTP API server (`analyze`) and interactive CLI (`convert`) for querying training results, plus a Leptos web frontend (`explore`) for visualization.

3. **Game Hosting**: A WebSocket server (`hosting`) for running live poker games with pluggable player implementations (compute, human, network).

## Training Pipeline

The training pipeline generates strategic abstractions and blueprint strategies:

1. For each layer of hierarchical abstraction (`preflop`, `flop`, `turn`, `river`):
   - Generate isomorphic hand clusters by exhaustively iterating through strategically equivalent situations
   - Initialize k-means centroids using k-means++ seeding over abstract distribution space <sup>2</sup>
   - Run hierarchical k-means clustering to group hands into strategically similar situations
   - Calculate Earth Mover's Distance metrics via optimal transport<sup>5</sup> between all cluster pairs
   - Save abstraction results and distance metrics to PostgreSQL

2. Run iterative Monte Carlo CFR training<sup>3</sup>:
   - Initialize regret tables and strategy profiles
   - Sample game trajectories using external sampling MCCFR
   - Update regret values and compute counterfactual values
   - Accumulate strategy updates with linear weighting
   - Periodically checkpoint blueprint strategy to database
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

| Street  | Abstraction Size | Metric Size |
| ------- | ---------------- | ----------- |
| Preflop | 4 KB             | 301 KB      |
| Flop    | 32 MB            | 175 KB      |
| Turn    | 347 MB           | 175 KB      |
| River   | 3.02 GB          | -           |

**Recommended Resources:**

- Training: 16 vCPU, 120GB RAM (configurable via Terraform)
- Database: PostgreSQL with 8 vCPU, 64GB RAM for production workloads
- Analysis: 1 vCPU, 4GB RAM for serving queries

# Modules

## Core

### `cards`

Core functionality for working with standard playing cards and Texas Hold'em rules:

- **Hand Evaluation**: Nanosecond hand strength calculation using lazy evaluation; fastest open-source hand evaluation algorithm; benchmarks outperform the popular Cactus Kev implementation
- **Equity Calculation**: Fast equity calculations between ranges of hands, supporting both exact enumeration and Monte Carlo simulation
- **Exhaustive Iteration**: Efficient iteration over cards, hands, decks, and private-public observations with lazy bitwise advancing
- **Distribution Analysis**: Tools for analyzing equity distributions and range vs range scenarios
- **Short Deck Support**: Full support for 36-card short deck variant with adjusted hand rankings and iterators
- **Bijective Representations**: Multiple card representations `(u8/u16/u32/u64)` allow for maximally efficient operations and transformations

### `gameplay`

A complete poker game engine implementation:

- **Standard Rules**: Full implementation of No-Limit Texas Hold'em rules and mechanics
- **Complex Showdowns**: Elegant handling and thorough testing of showdown edge cases like side pots, all-ins, dead cards, and multi-way ties
- **Flexible Payout Logic**: Configurable payout structures for different game formats
- **Decider Abstraction**: Generic trait system for implementing different player decision strategies
- **Functional Design**: Clean Node/Edge/Tree implementation for game state representation

### `clustering`

Advanced clustering capabilities for poker hand analysis:

- **Isomorphic Exhaustion**: Plays out _every one of 3.1T_ possible situations by respecting symmetries and enforcing permutation invariance<sup>4</sup>
- **Earth Mover's Distance (EMD)**: Implementation of EMD metric for comparing hand distributions over equity and hierarchical abstraction clusters
- **Hierarchical K-means**: Multi-level clustering algorithm with Elkan acceleration for creating strategic abstractions
- **Persistence**: Efficient serialization and deserialization of clustering results using PostgreSQL binary formats

### `transport`

Optimal transport algorithms for computing distribution distances:

- **Sinkhorn Iteration**: Near-linear time approximation of Wasserstein distance<sup>5</sup>
- **Greenhorn Algorithm**: Optimized Sinkhorn variant for sparse distributions
- **Greedy Matching**: Fast approximate coupling for large-scale comparisons
- **Measure Abstraction**: Generic support for arbitrary metric spaces

### `mccfr`

Monte Carlo Counterfactual Regret Minimization solver:

- **Generic Trait System**: Extensible `Encoder`, `Profile`, and `Solver` traits for any extensive-form game
- **RPS Validation**: Demonstrated convergence on Rock-Paper-Scissors as a correctness check
- **NLHE Implementation**: Full No-Limit Hold'em solver with external sampling MCCFR
- **Dynamic Tree Building**: On-the-fly game tree construction for memory efficiency
- **Linear Strategy Weighting**: Efficient strategy updates using iterative weighting and discount schemes<sup>6</sup>
- **Caching**: Optional tree caching for faster repeated traversals

## Training

### `autotrain`

Unified training pipeline orchestrating clustering and MCCFR:

- **CLI Interface**: `--status`, `--fast`, `--slow`, `--cluster` modes
- **Two-Phase Training**: Clustering phase followed by blueprint training
- **Graceful Interrupts**: Press 'Q' to cleanly stop training and checkpoint
- **Resumable State**: Training progress persisted to PostgreSQL for recovery
- **Epoch Management**: Configurable training epochs with progress tracking

### `workers`

Distributed training infrastructure:

- **Worker Pool**: Parallel training workers for MCCFR iterations
- **Memory Tracking**: Real-time statistics on memory usage and throughput
- **Record Serialization**: Efficient training record encoding for database writes

## Persistence

### `database`

PostgreSQL abstraction layer:

- **Source Trait**: Read interface for SELECT queries (memory, encoding, equity, metrics)
- **Sink Trait**: Write interface for INSERT/UPDATE operations
- **Stage Management**: Training stage tracking and validation
- **Connection Pooling**: Efficient connection management with `Arc<Client>`

### `save`

Data persistence layer:

- **PostgreSQL Backend**: Binary format serialization for high-throughput writes
- **Schema Management**: Table definitions and migrations for training artifacts
- **Streaming I/O**: Efficient bulk upload via `COPY IN` binary protocol
- **Hydration**: Deserialization of abstractions, profiles, and encoders from database

## Analysis & Visualization

### `analysis`

Tools for querying training results via PostgreSQL:

- **HTTP API**: Actix-web server on port 8888 for programmatic access
- **SQL Optimization**: Indexed queries for isomorphisms, abstractions, EMD metrics, and blueprint strategies
- **Query Interface**: Structured request/response types for API clients

### `client`

Leptos web frontend for interactive analysis:

- **Reactive UI**: Leptos-based client-side rendering with signals
- **Visualization Components**: PolicyDisplay, FeltSurface, NeighborhoodTable, Histogram
- **Context System**: Shared state for Chance, Choice, Policy, and Street
- **TailwindCSS**: Styled with utility-first CSS via Trunk build system
- **Multiple Views**: Distributions, topology, card displays, and neighborhood exploration

### `dto`

Data transfer objects for client-server communication:

- **Request Types**: Structured API request serialization
- **Response Types**: Typed API response deserialization

## Live Play

### `gameroom`

Async runtime for live poker games:

- **Room Coordinator**: Central game state and action history management
- **Actor Model**: Tokio-based async player task wrappers
- **Player Trait**: Async decision abstraction for pluggable player types
- **Event Broadcasting**: Real-time game events to all participants
- **Turn Orchestration**: Timeout handling and turn management

### `hosting`

HTTP/WebSocket server for game hosting:

- **REST API**: HTTP endpoints for game management
- **WebSocket Support**: Real-time bidirectional communication
- **Casino Coordinator**: Multi-room game management
- **Connection Handling**: Player session lifecycle management

### `players`

Concrete player implementations:

- **Human Player**: Interactive decision-making with input validation
- **Extensible Design**: Framework for compute, network, and random players

### `search`

Real-time subgame solving (in progress):

- Reserved for depth-limited solving during live gameplay
- Will integrate with blueprint strategy as a prior

# Getting Started

## Prerequisites

- Rust 1.90+ (Edition 2024)
- PostgreSQL 14+ (for training and analysis)
- Node.js (for Trunk/TailwindCSS frontend builds)

## Build Commands

```bash
# Build with default features (database)
cargo build --release

# Run training pipeline
cargo run --bin trainer --release -- --fast

# Start analysis server
cargo run --bin analyze --release

# Start Leptos frontend (requires trunk)
trunk serve

# Run interactive CLI
cargo run --bin convert --release

# Run benchmarks
cargo bench --features benchmark
```

# Binaries

| Binary    | Features   | Description                                        |
| --------- | ---------- | -------------------------------------------------- |
| `trainer` | `database` | Unified training pipeline for clustering and MCCFR |
| `analyze` | `database` | HTTP REST API server on port 8888                  |
| `convert` | `database` | Interactive CLI for type conversions and queries   |
| `explore` | `client`   | Leptos web frontend (WASM)                         |
| `hosting` | `database` | WebSocket server for live games                    |

# Feature Flags

| Feature     | Description                                    |
| ----------- | ---------------------------------------------- |
| `database`  | PostgreSQL integration (default)               |
| `server`    | Server-side dependencies (Actix, Tokio, Rayon) |
| `client`    | Leptos frontend with CSR                       |
| `disk`      | Legacy file-based persistence (deprecated)     |
| `shortdeck` | 36-card short deck variant                     |
| `benchmark` | Criterion benchmarks                           |

# Infrastructure

The project includes Terraform configurations for AWS deployment in `infra/`:

- **ECS Clusters**: Fargate tasks for trainer and analyzer services
- **RDS PostgreSQL**: Managed database for training artifacts
- **CloudFront CDN**: Static asset distribution for the frontend
- **Route53 DNS**: Domain management
- **ECR Registry**: Container image storage

## Deployment

```bash
# Deploy trainer
.github/workflows/deploy-trainer.yml

# Deploy analyzer
.github/workflows/deploy-analyze.yml

# Deploy frontend
.github/workflows/deploy-explore.yml
```

## Docker

Multi-stage Dockerfile supporting three targets:

```bash
# Build trainer image
docker build --target trainer -t robopoker-trainer .

# Build analyzer image
docker build --target analyzer -t robopoker-analyzer .

# Build frontend image
docker build --target explorer -t robopoker-explorer .
```

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
