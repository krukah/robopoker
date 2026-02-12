# robopoker

[![license](https://img.shields.io/github/license/krukah/robopoker)](LICENSE)
[![build](https://github.com/krukah/robopoker/actions/workflows/ci.yml/badge.svg)](https://github.com/krukah/robopoker/actions/workflows/ci.yml)

A Rust toolkit for game-theoretically optimal poker strategies, implementing state-of-the-art algorithms for No-Limit Texas Hold'em with functional parity to Pluribus<sup>1</sup>.

## Visual Tour

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

## Features

- **Fastest open-source hand evaluator** - Nanosecond evaluation outperforming Cactus Kev
- **Strategic abstraction** - Hierarchical k-means clustering of 3.1T poker situations
- **Optimal transport** - Earth Mover's Distance via Sinkhorn algorithm
- **MCCFR solver** - External sampling with dynamic tree construction
- **PostgreSQL persistence** - Binary format serialization for efficiency
- **Short deck support** - 36-card variant with adjusted rankings

## Quick Start

Add robopoker to your `Cargo.toml`:

```toml
[dependencies]
rbp = "0.1"

# Or individual crates:
rbp-cards = "0.1"
rbp-gameplay = "0.1"
rbp-mccfr = "0.1"
```

### Basic Usage

```rust
use rbp::cards::*;
use rbp::gameplay::*;

// Create a hand and evaluate it
let hand = Hand::from("AcKsQhJdTc9h8s");
let strength = hand.evaluate();

// Work with observations
let obs = Observation::from(Street::Flop);
let equity = obs.equity();
```

## Crate Overview

| Crate | Description |
|-------|-------------|
| [`rbp`](crates/rbp) | Facade re-exporting all public crates |
| [`rbp-core`](crates/rbp-core) | Type aliases, constants, shared traits |
| [`rbp-cards`](crates/rbp-cards) | Card primitives, hand evaluation, equity |
| [`rbp-transport`](crates/rbp-transport) | Optimal transport (Sinkhorn, EMD) |
| [`rbp-mccfr`](crates/rbp-mccfr) | Game-agnostic CFR framework |
| [`rbp-gameplay`](crates/rbp-gameplay) | Poker game engine |
| [`rbp-clustering`](crates/rbp-clustering) | K-means abstraction |
| [`rbp-nlhe`](crates/rbp-nlhe) | No-Limit Hold'em solver |
| [`rbp-dto`](crates/rbp-dto) | API request/response types |
| [`rbp-pg`](crates/rbp-pg) | PostgreSQL serialization |
| [`rbp-database`](crates/rbp-database) | Database pipeline |
| [`rbp-autotrain`](crates/rbp-autotrain) | Training orchestration |
| [`rbp-workers`](crates/rbp-workers) | Distributed training |
| [`rbp-gameroom`](crates/rbp-gameroom) | Async game coordinator |
| [`rbp-players`](crates/rbp-players) | Player implementations |
| [`rbp-analysis`](crates/rbp-analysis) | Query API |
| [`rbp-server`](crates/rbp-server) | HTTP server |
| [`rbp-auth`](crates/rbp-auth) | JWT authentication |
| [`rbp-records`](crates/rbp-records) | Hand history |

## Architecture

### Core Layer

**`rbp-cards`** — Card representation, hand evaluation, and strategic primitives:
- Bijective card representations (`u8`/`u16`/`u32`/`u64`) for efficient operations
- Lazy hand strength evaluation in nanoseconds
- Equity calculation via enumeration and Monte Carlo
- Exhaustive iteration over cards, hands, decks, and observations
- Short deck (36-card) variant support

**`rbp-transport`** — Optimal transport algorithms:
- Sinkhorn iteration for near-linear Wasserstein approximation<sup>5</sup>
- Greenhorn optimization for sparse distributions
- Generic `Measure` abstraction for arbitrary metric spaces

**`rbp-mccfr`** — Game-agnostic CFR framework:
- State primitives: `Turn`, `Edge`, `Game`, `Info`, `Tree`
- Strategy representation: `Encoder`, `Profile`, `InfoSet`
- Training: `Solver` trait with pluggable algorithms
- Schemes: `RegretSchedule`, `PolicySchedule`, `SamplingScheme`
- Subgame solving with safe search

### Domain Layer

**`rbp-gameplay`** — Complete poker game engine:
- Full No-Limit Texas Hold'em rules
- Complex showdown handling (side pots, all-ins, ties)
- Bet sizing abstraction via `Size` enum (`SPR(n,d)` / `BBs(n)`)
- Clean Node/Edge/Tree game state representation

**`rbp-clustering`** — Hand abstraction via clustering:
- Hierarchical k-means with Elkan acceleration
- Earth Mover's Distance between distributions
- Isomorphic exhaustion of 3.1T situations<sup>4</sup>
- PostgreSQL binary persistence

**`rbp-nlhe`** — Concrete NLHE solver:
- `NlheSolver<R, W, S>` with pluggable regret/policy/sampling
- `NlheEncoder` for state→info mapping
- `NlheProfile` for regret/strategy storage
- Production config: `Flagship` type alias

### Infrastructure Layer

**`rbp-pg`** — PostgreSQL integration:
- Binary format serialization via `Row` trait
- Schema definitions via `Schema` trait
- Streaming I/O via `COPY IN` binary protocol
- `Hydrate` trait for database loading

**`rbp-database`** — Database pipeline:
- `Source` trait for SELECT queries
- `Sink` trait for INSERT/UPDATE operations
- Training stage tracking and validation

**`rbp-autotrain`** — Training orchestration:
- Two-phase: clustering then MCCFR
- Fast (in-memory) and slow (distributed) modes
- Graceful interrupts and resumable state
- Timed training via `TRAIN_DURATION`

## Training Pipeline

1. **Hierarchical Abstraction** (per street: river → turn → flop → preflop):
   - Generate isomorphic hand clusters
   - Initialize k-means centroids via k-means++<sup>2</sup>
   - Run clustering to group strategically similar hands
   - Calculate EMD metrics via optimal transport<sup>5</sup>
   - Save abstractions to PostgreSQL

2. **MCCFR Training**<sup>3</sup>:
   - Sample game trajectories via external sampling
   - Update regret values and counterfactual values
   - Accumulate strategy with linear weighting
   - Checkpoint blueprint strategy to database

3. **Real-time Search** (in progress):
   - Depth-limited subgame solving<sup>10</sup>
   - Blueprint strategy as prior
   - Targeted Monte Carlo rollouts

## System Requirements

| Street  | Abstraction Size | Metric Size |
| ------- | ---------------- | ----------- |
| Preflop | 4 KB             | 301 KB      |
| Flop    | 32 MB            | 175 KB      |
| Turn    | 347 MB           | 175 KB      |
| River   | 3.02 GB          | -           |

**Recommended:**
- Training: 16 vCPU, 120GB RAM
- Database: PostgreSQL 14+ with 8 vCPU, 64GB RAM
- Analysis: 1 vCPU, 4GB RAM

## Feature Flags

| Feature | Description |
|---------|-------------|
| `database` | PostgreSQL integration |
| `server` | Server dependencies (Actix, Tokio, Rayon) |
| `shortdeck` | 36-card short deck variant |

## Building

```bash
# Build all crates
cargo build --workspace

# Build with database features
cargo build --workspace --features database

# Run tests
cargo test --workspace

# Generate documentation
cargo doc --workspace --no-deps --open
```

## References

1. (2019). Superhuman AI for multiplayer poker. [(Science)](https://science.sciencemag.org/content/early/2019/07/10/science.aay2400)
2. (2014). Potential-Aware Imperfect-Recall Abstraction with Earth Mover's Distance in Imperfect-Information Games. [(AAAI)](http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf)
3. (2007). Regret Minimization in Games with Incomplete Information. [(NIPS)](https://papers.nips.cc/paper/3306-regret-minimization-in-games-with-incomplete-information)
4. (2013). A Fast and Optimal Hand Isomorphism Algorithm. [(AAAI)](https://www.cs.cmu.edu/~waugh/publications/isomorphism13.pdf)
5. (2018). Near-linear time approximation algorithms for optimal transport via Sinkhorn iteration. [(NIPS)](https://arxiv.org/abs/1705.09634)
6. (2019). Solving Imperfect-Information Games via Discounted Regret Minimization. [(AAAI)](https://arxiv.org/pdf/1809.04040.pdf)
7. (2013). Action Translation in Extensive-Form Games with Large Action Spaces. [(IJCAI)](http://www.cs.cmu.edu/~sandholm/reverse%20mapping.ijcai13.pdf)
8. (2015). Discretization of Continuous Action Spaces in Extensive-Form Games. [(AAMAS)](http://www.cs.cmu.edu/~sandholm/discretization.aamas15.fromACM.pdf)
9. (2015). Regret-Based Pruning in Extensive-Form Games. [(NIPS)](http://www.cs.cmu.edu/~sandholm/regret-basedPruning.nips15.withAppendix.pdf)
10. (2018). Depth-Limited Solving for Imperfect-Information Games. [(NeurIPS)](https://arxiv.org/pdf/1805.08195.pdf)
11. (2017). Reduced Space and Faster Convergence in Imperfect-Information Games via Pruning. [(ICML)](http://www.cs.cmu.edu/~sandholm/reducedSpace.icml17.pdf)
12. (2017). Safe and Nested Subgame Solving for Imperfect-Information Games. [(NIPS)](https://www.cs.cmu.edu/~noamb/papers/17-NIPS-Safe.pdf)

## License

MIT License - see [LICENSE](LICENSE) for details.
