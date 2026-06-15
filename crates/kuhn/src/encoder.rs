use super::*;
use mccfr::*;
use subgame::*;

#[derive(Default)]
pub struct KuhnEncoder;

impl CfrEncoder for KuhnEncoder {
    type T = KuhnTurn;
    type E = KuhnEdge;
    type G = KuhnGame;
    type I = KuhnInfo;
    const CHECK_RECALL: bool = true;

    fn seed(&self, game: &Self::G) -> Self::I {
        let acting = matches!(game.turn(), KuhnTurn::Player(_));
        kuhn_info(acting, if acting { game.hole_rank(0) } else { Rank::J }, History::Open)
    }

    fn resume<P>(&self, _past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        let acting = matches!(game.turn(), KuhnTurn::Player(_));
        let actor = match game.turn() {
            KuhnTurn::Player(i) => i,
            _ => 0,
        };
        kuhn_info(acting, game.hole_rank(actor), game.history())
    }

    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        match node.game().turn() {
            KuhnTurn::Terminal => vec![],
            KuhnTurn::Chance => node
                .game()
                .deals()
                .map(|c| (KuhnEdge::Deal(c), node.game().apply(KuhnEdge::Deal(c)), node.index()))
                .collect(),
            _ => node.branches(),
        }
    }
}

impl EmbeddedHistory for KuhnEncoder {}

impl<const W: usize> WorldRestrict<W> for KuhnEncoder {
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G {
        let KuhnTurn::Player(actor) = external else {
            return *observed;
        };
        Card::ALL
            .into_iter()
            .filter(|&c| c != observed.hole_card(1 - actor))
            .map(|c| observed.with_card(actor, c))
            .map(|root| (root, root.hole_rank(actor)))
            .find(|(_, secret)| belief.remember(secret, world))
            .map_or(*observed, |(root, _)| root)
    }
}

impl KuhnEncoder {
    /// Baseline posterior: uniform reach over all opponent hands.
    ///
    /// Enumerates every card the external player could hold (excluding
    /// the internal player's card) and assigns equal reach. Used as the
    /// prior when no actions have been observed.
    pub fn baseline(&self, game: &KuhnGame, external: KuhnTurn) -> Posterior<Rank> {
        let KuhnTurn::Player(actor) = external else {
            return Posterior::default();
        };
        Card::ALL
            .into_iter()
            .filter(|&c| c != game.hole_card(1 - actor))
            .map(super::card::Card::rank)
            .fold(Posterior::default(), |post, rank| post.add(rank, 1.0))
    }
}
