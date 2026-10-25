Turning a reducible hand into decisions our blueprint can be asked about.

One [`Spot`] is a single postflop decision: the hero's view of everything
that had happened when it was their turn, plus what they actually did. Ask a
strategy what it would do at [`Spot::witness`] and compare against
[`Spot::played`] — that comparison is Tier 2 ([`crate::Agreement`]). Play the
hand out from [`Spot::game`] behind each of the two actions and difference
the results — that is Tier 3 ([`crate::Ledger`]).

Three things have to line up for the question to be fair:

- **The preflop is substituted**, not translated — see [`crate::Projection::project`].
- **The hero sees only what they had seen.** The arrangement is truncated to
  the decision's street, so a flop question cannot leak the turn card.
- **Chip amounts divide** by the same scale [`crate::Trace::walk`] uses, and all-in
  is read off the log rather than off the divide.

What a `Spot` carries beyond the hero's view — both players' hole cards, the
board the dealer actually ran out — is never shown to a strategy. It is there
so a rollout can settle a showdown, and so [`crate::Duel::worlds`] only has to
sample the streets the log never reached.
