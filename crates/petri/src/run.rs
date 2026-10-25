use crate::*;
use mccfr::*;
use monge::Density;
use neural::*;
use pokerkit::*;

/// [`DeepCfr`] under external sampling, on the game `G` names, with the networks `P`.
pub struct Run<G, P>
where
    G: Probe,
    P: RegretNet<F = G::F> + StrategyNet + Send + Sync,
{
    solver: DeepCfr<G::N, P, ExternalSampling>,
}

impl<G, P> Lab for Run<G, P>
where
    G: Probe,
    P: RegretNet<F = G::F> + StrategyNet + Send + Sync,
{
    fn step(&mut self, epochs: usize) -> Frame {
        (0..epochs).for_each(|_| self.solver.step());
        self.frame(None)
    }

    fn checkpoint(&mut self) -> Frame {
        self.solver.profile_mut().net_mut().fit_strategy();
        self.frame(Some(self.solver.exploitability()))
    }
}

impl<G, P> Default for Run<G, P>
where
    G: Probe,
    P: RegretNet<F = G::F> + StrategyNet + Send + Sync + Default,
{
    fn default() -> Self {
        Self {
            solver: DeepCfr::new(G::N::default(), P::default()),
        }
    }
}

impl<G, P> Run<G, P>
where
    G: Probe,
    P: RegretNet<F = G::F> + StrategyNet + Send + Sync,
{
    fn frame(&self, exploitability: Option<Utility>) -> Frame {
        Frame::new(
            self.solver.profile().epoch(),
            self.solver.profile().iteration(),
            exploitability,
            G::infos()
                .iter()
                .map(|info| self.reading(info, exploitability.is_some()))
                .collect(),
        )
    }

    fn reading(&self, info: &InfoOf<G>, averaged: bool) -> Reading {
        let profile = self.solver.profile();
        let regret = profile.net().est_regret(info);
        let current = profile.iterated_strategy(info);
        let average = averaged.then(|| profile.averaged_strategy(info));
        Reading::new(
            G::label(info),
            format!("{:?}", G::default().turn(info)),
            G::tolerance(info),
            info.menu()
                .map(|ref edge| {
                    Lane::new(
                        format!("{edge:?}"),
                        regret.density(edge),
                        current.density(edge),
                        average.as_ref().map(|policy| policy.density(edge)),
                        G::nash(info, edge),
                    )
                })
                .collect(),
        )
    }
}
