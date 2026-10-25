Tier 3: what a logged decision was worth, against what ours was worth.

Tier 2 ([`crate::Agreement`]) can only say whether we *agreed* with the log.
Agreement is not strength: the log records what happened after Pluribus's
action, so the moment our bot deviates there is no continuation to score.

The way out is to stop looking for the continuation and generate it. At each
logged decision, play the hand out twice with our own strategy driving **both**
seats — once behind the action the log took, once behind the action we would
have taken — and difference the two:

```text
              ┌── log's action ──▶ our strategy plays it out ──▶ chips
  decision ───┤
              └── our action  ───▶ our strategy plays it out ──▶ chips
```

[`Duel::delta`] is the difference, in big blinds: **how much substituting the
logged decision for ours gains against our own strategy.** Both branches run
through the identical rollout, so whatever our value function gets wrong
largely cancels out of the difference.

# What the sign actually measures

δ is not a comparison of two players. Both continuations are played by *our*
strategy, so δ asks one question about one strategy: **is there a better move
here than the one we make, against ourselves?**

A converged blueprint is best-responding to itself. Every action in its
support has equal value against it and every action outside is worse, so
substituting any other action can only lose:

```text
  self-consistent strategy  ⟹  δ ≤ 0 for every substitution
```

So **δ > 0 is a positive measurement that we are not converged at that node —
the log found a profitable deviation against us.** That makes [`Ledger`] a
local best response with the log's real decisions as the probe set instead of
an exhaustive search, and [`Ledger::delta`] **a lower bound on our own
exploitability**, in big blinds per decision. Lower is better; zero is the
target; the bound is only ever loose in the safe direction, since a probe set
of a few thousand human-chosen actions cannot find every leak.

It needs a scale to be legible, and the cheapest one is a strategy that is
obviously bad. `cargo run -p phh --example rollout` duels the corpus against
one that always checks or calls: **+0.6188 bb/decision**. That is what "very
exploitable" reads as on this probe set.

Read [`Ledger::t`] before any of it: a sign inside two standard errors is
noise. And note the residual bias runs in our favor — the log's action was
chosen against five human professionals rather than against us, and our bot
plays the continuation — so δ ≈ 0 is weaker evidence of soundness than δ > 0
is of a leak.

# Where the cards come from

The PHH corpus deals every player's hole cards face-up in the log — all six,
in all 10,000 hands — and records however much of the board the dealer ran
out before the hand ended. So a [`World`] samples *only* the streets the log
never reached. Using the real cards is not cheating: no strategy in the
rollout ever sees them, they enter only at settlement. Holding them fixed
across both branches is the common-random-numbers trick, and it is what makes
the difference of two noisy means far tighter than either mean.
