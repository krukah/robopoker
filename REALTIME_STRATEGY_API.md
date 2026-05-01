# Realtime Strategy API

This document describes the intended API for using the bot as a real-time decision
tool while playing a heads-up hand. The current server already exposes
`POST /api/blueprint` for blueprint lookup only. A Pluribus-style realtime
endpoint should be added separately, because realtime DLS is an on-demand solve,
not a database lookup.

## Seat And Blind Semantics

The heads-up engine uses zero-based player indices:

- `P0` means `Turn::Choice(0)`.
- `P1` means `Turn::Choice(1)`.

In the first hand, `P0` is the dealer/button and posts the small blind. `P1`
posts the big blind. This is visible in `Game::default()` and `Game::root()`:
the default dealer is `0`, and `Game::root()` posts blinds by calling
`game.posts()` twice.

In heads-up play:

- Preflop: the dealer/small blind acts first after blinds.
- Postflop: the non-dealer/big blind acts first.
- Between hands: the dealer rotates.

So for the first hand:

```text
P0 = button / small blind
P1 = big blind
```

After each completed hand, the dealer rotates, so do not assume `P0` is always
small blind across a session. For a standalone single-hand API request, the
current code assumes the canonical first-hand setup unless the request format is
extended to include dealer/button state.

## Existing API: Blueprint Lookup

### Endpoint

```http
POST /api/blueprint
Content-Type: application/json
```

### Purpose

Fetches the stored blueprint strategy for the current infoset from the database.
It does not run realtime search and does not run DLS.

### Behavior

```text
request
-> build Partial/Recall
-> map observation to abstraction bucket
-> build NlheInfo(past, present, choices)
-> query BLUEPRINT table
-> return strategy or null
```

This endpoint is useful when you want the raw offline blueprint strategy.

## Proposed API: Realtime DLS Advice

### Endpoint

```http
POST /api/realtime
Content-Type: application/json
```

### Purpose

Returns an action distribution for the current hand state using the stronger
runtime path:

```text
blueprint + Pluribus-style depth-limited subgame solving
```

DLS should be attempted first. Blueprint lookup should be used as fallback, not
as a gate before DLS.

Why: DLS depends on the blueprint for abstraction, ranges, continuation policy,
and fallback values. It is not a replacement for a missing blueprint; it is a
runtime refinement of it.

## Request

```json
{
  "turn": "P0",
  "seen": "Ah Kh ~ Qs 7d 2c",
  "past": [
    "CALL 1",
    "CHECK",
    "RAISE 50"
  ],
  "mode": "sample"
}
```

### Fields

`turn`

The hero/player perspective. Use `P0` or `P1`.

`seen`

Hero private cards plus public board, in the same observation format accepted by
the existing blueprint API. This must describe exactly what the hero can see.

`past`

Action history from the post-blind root to the current state, excluding blinds.
The API should parse this using `Action::try_from`, then build a `Partial` with
`Partial::try_build`.

`mode`

Optional. Suggested values:

- `sample`: return the full distribution and a sampled/recommended action.
- `argmax`: return the full distribution and choose the highest-probability action.

## Response

```json
{
  "source": "dls",
  "recommended_action": "CALL 50",
  "actions": [
    {
      "action": "FOLD",
      "probability": 0.08
    },
    {
      "action": "CALL 50",
      "probability": 0.67
    },
    {
      "action": "RAISE 150",
      "probability": 0.25
    }
  ],
  "diagnostics": {
    "used_dls": true,
    "used_blueprint_fallback": false,
    "used_legal_fallback": false,
    "offtree_detected": true,
    "solve_ms": 284,
    "timeout_ms": 5000
  }
}
```

### Response Fields

`source`

One of:

- `dls`: DLS solve completed and produced a policy.
- `blueprint_fallback`: DLS failed or timed out, and blueprint policy was used.
- `legal_fallback`: neither DLS nor blueprint was available, so a legal action
  fallback was used.

`recommended_action`

The action selected according to `mode`.

`actions`

The action distribution to use for decision making.

`diagnostics`

Debug metadata for integration and safety. This is important while testing
against real opponents or Slumbot, because off-tree actions and abstraction
coverage issues are expected.

## Recommended Decision Flow

```text
1. Parse request into Turn, Observation, and Vec<Action>.
2. Build Partial with Partial::try_build.
3. Try observation abstraction with NlheEncoder::try_abstraction.
4. Try DLS:
   - construct depth_limited_subgame(recall)
   - solve with timeout
   - return SubInfo::Info policy if available
5. If DLS fails or times out, fallback to blueprint policy.
6. If blueprint is unavailable, fallback to legal action or return an error.
```

Important: DLS should not be used only when blueprint misses. DLS is the primary
real-time strategy, and blueprint is its base/fallback.

## DLS Internals

The current DLS implementation follows this structure:

```text
Partial/Recall
-> current betting round root
-> subgame with current-street prefix replay
-> frontier at depth limit
-> continuation choices:
   - blueprint
   - fold-biased blueprint
   - call-biased blueprint
   - raise-biased blueprint
-> rollout evaluator
-> local value cache
-> subgame CFR
-> action distribution for current info
```

The continuation rollout can fall back to blueprint frontier EV if the evaluator
cannot produce a value. This avoids no-output cases during live play.

## Off-Tree Actions

No-limit poker allows arbitrary raise sizes, but the blueprint uses an action
abstraction. Existing code maps raw `Action` values into canonical `Edge` values
using:

```text
Game::edgify
Game::actionize
Game::snap
```

The realtime endpoint should report `offtree_detected = true` when an action had
to be canonicalized. In the current implementation, off-tree support is
conservative: it detects and logs the situation, then uses the existing
canonicalization path so database keys are not broken.

Future work can add the exact off-tree action into the realtime subgame action
set, but that should remain separate from blueprint database keys.

## Error Responses

Invalid request format:

```json
{
  "error": "invalid recall format"
}
```

Invalid action sequence:

```json
{
  "error": "invalid action sequence: illegal action RAISE 50 at Choice(1)"
}
```

No available strategy:

```json
{
  "error": "strategy unavailable"
}
```

In production, prefer returning a legal fallback action over failing hard unless
the input itself is invalid.

## Implementation Notes

- The endpoint should live near the existing analysis API handlers.
- It should reuse the existing `GetPolicy` request shape if possible, with an
  optional `mode` field.
- It should load/use the same `Flagship` blueprint profile that `RealTimePlayer`
  uses.
- It should expose diagnostics during testing and Slumbot integration.
- It should use Rust `1.90` in this repo.

