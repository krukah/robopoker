# kuhn

Kuhn poker solver for MCCFR framework validation

Part of the [robopoker](https://github.com/krukah/robopoker) workspace — see the top-level README for the full architecture and crate map.

## Nash equilibrium reference

The reference the solver tests in `src/cfr/solver.rs` check against.

All tests use 2^18 iterations. 6-card Kuhn ({J,Q,K} × {♠,♥}) has 12 info
sets and 30 deals; same-rank deals (J♠ vs J♥) tie at showdown, shifting the
equilibrium away from classical 3-card Kuhn. Tolerances are calibrated
empirically as **μ + 4σ** rounded up to 0.005, targeting <1% flake.

Pure strategies (invariant to deck size):

- **J facing bet**: always fold (EV(call) = −12/7 < −1 = EV(fold))
- **K facing bet**: always call (worst case ties K, otherwise wins)
- **Q opening**: always check (EV(bet) < EV(check) when Q only gets called by K)
- **K after check**: always bet (value bet the nuts)
- **J facing check-bet**: always fold (analogous to J facing bet)
- **K facing check-bet**: always call (analogous to K facing bet)

Mixed strategies (unique, all fractions of 31).

From any rank's perspective, opponent has: same rank 1/5, each other 2/5.

| Variable | Info  | Edge | Value | Indifference condition          |
|----------|-------|------|-------|---------------------------------|
| a        | J\|   | Bet  | 9/31  | P1-Q indiff facing bet: k = 3a |
| k        | K\|   | Bet  | 27/31 | P1-Q indiff facing bet: k = 3a |
| c₁       | Q\|B  | Call | 17/31 | P1-J indiff at Check + P1-Q    |
| c₂       | Q\|XB | Call | 23/31 | P1-J & P1-Q indiff at Check    |
| b        | J\|X  | Bet  | 9/31  | P0-Q indiff at CheckBet        |
| d        | Q\|X  | Bet  | 8/31  | P0-J & P0-K indiff at Open     |

Derivation sketch:
1. P1-Q facing bet indiff → k = 3a
2. P1-J indiff at Check → c₂ = (3+5a)/6
3. P1-Q indiff at Check → c₂ = 2k−1
4. Solving (1)-(3): a = 9/31, k = 27/31, c₂ = 23/31
5. P0-Q indiff at CheckBet → 6b+d = 2
6. P0-K indiff at Open → c₁ = b+d
7. P0-J indiff at Open → b = 6c₁−3
8. Solving (5)-(7): b = 9/31, d = 8/31, c₁ = 17/31

Game value: EV(P0) = -19/465 ≈ -0.041 (P1 has position). Compare 3-card
Kuhn's -1/18 ≈ -0.056 — same-rank ties cut P0's disadvantage by ~26%.

P1-Q is *analytically* indifferent at Check (EV(bet) = EV(check) for all
d), so d's regret signal is weak and converges slowly — hence the wider
0.18 tolerance on d.
