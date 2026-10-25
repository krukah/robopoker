use crate::*;
use mccfr::*;

pub struct TrainingBatch<F>
where
    F: CfrInfo,
{
    regrets: Vec<Sample<F, Regret>>,
    weights: Vec<Sample<F, Policy>>,
}

impl<F> TrainingBatch<F>
where
    F: CfrInfo,
{
    pub(crate) fn new() -> Self {
        Self {
            regrets: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn feed<N>(self, walker: TurnOf<F>, net: &mut N)
    where
        N: RegretNet<F = F> + StrategyNet,
    {
        let Self { regrets, weights } = self;
        regrets.into_iter().for_each(|sample| net.see_regret(walker, sample));
        weights.into_iter().for_each(|sample| net.see_strategy(sample));
    }

    pub(crate) fn capture<S>(&mut self, solver: &S, infoset: &InfoSet<F>, walker: TurnOf<F>)
    where
        S: DeepSolver<F = F>,
    {
        let Self { regrets, weights } = self;
        let turn = infoset.head().game().turn();
        if turn == walker {
            regrets.push(solver.regret(infoset));
        } else if turn.is_opponent(&walker) {
            weights.push(solver.policy(infoset));
        }
    }

    #[cfg(feature = "server")]
    pub(crate) fn append(&mut self, mut other: Self) {
        let Self { regrets, weights } = self;
        let Self {
            regrets: ref mut more,
            weights: ref mut also,
        } = other;
        regrets.append(more);
        weights.append(also);
    }
}
