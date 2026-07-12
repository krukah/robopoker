# deuce

Card representation, hand evaluation, and strategic abstraction primitives.

`deuce` is the foundational card layer. Every type is built around bijective integer encodings and branchless bit manipulation: a `Card` is one byte, a `Hand` is a 64-bit set, and hand evaluation is pure bit-twiddling. On top of these it builds the suit-isomorphism machinery that collapses the game's enormous observation space into strategically-distinct equivalence classes.

## Architecture

The core types form an encoding ladder from a single card up to a canonical, suit-reduced game state:

```mermaid
flowchart LR
  Rank["Rank<br/>2..A"] --> Card
  Suit["Suit<br/>clubs..spades"] --> Card
  Card["Card<br/>u8 in 0..52"] --> Hand["Hand<br/>u64 bitset"]
  Hand --> Hole["Hole<br/>2 cards"]
  Hand --> Board["Board<br/>3 to 5 cards"]
  Hole --> Obs["Observation<br/>pocket + public"]
  Board --> Obs
  Obs --> Iso["Isomorphism<br/>canonical form"]
  Hand --> Eval["Evaluator"]
  Eval --> Rnk["Ranking + Kickers"]
  Rnk --> Str["Strength<br/>comparable"]
```

Two pipelines run over these types. Evaluation turns any 5-to-7 card `Hand` into a comparable `Strength`; abstraction canonicalizes an `Observation` under the 24 suit permutations of the symmetric group:

```mermaid
flowchart TD
  subgraph eval["Hand evaluation"]
    H["Hand"] --> E["Evaluator::find_ranking<br/>straight flush down to high card"]
    E --> R["Ranking"]
    E --> K["find_kickers → Kickers"]
    R --> S["Strength, Ord"]
    K --> S
    S --> EQ["Observation::equity<br/>showdown win rate"]
  end
  subgraph iso["Suit canonicalization"]
    O["Observation"] --> P["Permutation::from<br/>co-lexicographic suit sort"]
    P --> T["permute → canonical Observation"]
    T --> I["Isomorphism"]
  end
```

`Evaluator` searches rankings strongest-to-weakest over the `Hand`'s `u64`, returning a `Ranking` (category plus defining ranks) and a `Kickers` bitmask; together they compose into an `Ord`-comparable `Strength`. An `Observation` is a player's card view (`pocket` + `public`) and serializes to `i64`. `Permutation` derives the canonical suit relabeling by sorting suits co-lexicographically, and `Isomorphism` applies it — reducing billions of river observations to ~123M distinct classes. `HandIterator` (Gosper's-hack combinations) drives `ObservationIterator` and `IsomorphismIterator` to enumerate a whole `Street` in constant space. A `shortdeck` feature switches the stack to the 36-card variant.
