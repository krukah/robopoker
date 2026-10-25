Tier 2: does our blueprint choose what Pluribus chose?

Scoring is deliberately modest about what it can prove. Agreement with
Pluribus is not correctness — Pluribus is not a Nash equilibrium and was
playing a different game. A *low* agreement rate on a spot type is a lead
worth pulling; a high one is weak evidence of nothing being badly wrong.

Two numbers do the work:

- **Agreement** — how often our argmax matches. Cheap to read, but it throws
  away the distribution and rewards being confidently wrong less than it
  should.
- **Perplexity** — `exp` of the mean negative log-likelihood of Pluribus's
  actual action under our distribution. Reads as *"our blueprint was
  effectively choosing uniformly among this many actions"*. Unlike agreement
  it is sensitive to how much mass we put on the played action, so a policy
  that hedges correctly beats one that guesses right half the time and
  assigns zero the rest.

Neither means anything without a floor to compare against, so [`Agreement`]
carries two: [`Agreement::naive`], the score from always predicting the
single most common action, and [`Agreement::spread`], the mean size of the
choice set. A blueprint that cannot beat `naive` has told us nothing.
