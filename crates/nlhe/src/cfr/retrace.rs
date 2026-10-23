use crate::*;
use kicker::*;
use mccfr::*;

/// A [`Choice`] at each decision along a recall's own walk: the board-faithful sibling of
/// [`Encoding::replay`]. `replay` re-derives each state by applying its edge, and a chance
/// edge is a card-less [`Edge::Draw`], so it deals a **random** board past a street boundary.
pub trait Retrace
where
    Self: Encoding<F: CfrInfo<X: CfrPublic<G = NlheGame>>>,
{
    fn retrace<R>(&self, recall: &R) -> Vec<Choice<InfoOf<Self>>>
    where
        R: Recall,
    {
        recall
            .walk()
            .into_iter()
            .scan(Vec::new(), |past, step| {
                let game = NlheGame::from(step.game());
                let turn = NlheTurn::from(step.turn());
                let edge = NlheEdge::from(step.edge());
                let info = self.encode(&game, past.iter().rev().copied());
                past.push(edge);
                Some((!turn.is_chance()).then(|| Choice::new(turn, info, edge)))
            })
            .flatten()
            .collect()
    }
}
