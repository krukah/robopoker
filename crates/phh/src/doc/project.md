Classifying a multiway hand against our heads-up game tree.

Our blueprint is `pokerkit::N = 2`. A 6-max log is not that game, but most of
it collapses onto that game: ~90% of Pluribus pots that see a flop see it
heads-up. The question is which of those collapse *faithfully*.

The discriminator is postflop action order. In 6-max, betting opens left of
the button, so the small blind acts first on every postflop street. In
heads-up the small blind **is** the button and acts last. So:

- **blind vs. blind** — SB acts first postflop, BB last. Heads-up has it the
  other way round. Not our game; [`Shape::Inverted`].
- **anything else** — the earlier seat acts first postflop and the later seat
  last, which is exactly the heads-up shape with the earlier seat playing the
  big blind's role and the later seat playing the button's.
  [`Shape::HeadsUp`].

What the reduction does not preserve is the pot: a folded blind leaves dead
money our tree cannot represent (0.5bb in 2,464 of the 3,693 reducible hands,
1bb in 511, 1.5bb in 595). [`Reduction`] carries the flop pot in big blinds so the caller
can project onto the nearest reachable heads-up preflop path and absorb the
difference there.
