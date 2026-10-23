use crate::*;
use mccfr::Node;
use mccfr::*;
use subgame::*;

#[derive(Default)]
pub struct KuhnEncoder;

impl CfrSpec for KuhnEncoder {
    type F = KuhnInfo;
}

impl Encoding for KuhnEncoder {
    fn seed(&self, game: &GameOf<Self>) -> InfoOf<Self> {
        let acting = matches!(game.turn(), KuhnTurn::Player(_));
        Info::kuhn_info(acting, if acting { game.hole_rank(0) } else { Rank::J }, History::Open)
    }

    fn public<P>(&self, game: &GameOf<Self>, _: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        KuhnPublic::new(matches!(game.turn(), KuhnTurn::Player(_)), game.history())
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        game.hole_rank(match game.turn() {
            KuhnTurn::Player(i) => i,
            _ => 0,
        })
    }

    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        match node.game().turn() {
            KuhnTurn::Terminal => Branches::new(node.index(), *node.game(), std::iter::empty()),
            KuhnTurn::Chance => Branches::new(node.index(), *node.game(), node.game().deals().map(KuhnEdge::Deal)),
            _ => node.branches(),
        }
    }
}

impl<const D: usize> Frontier<D> for KuhnEncoder {}

impl<const W: usize> WorldRestrict<W> for KuhnEncoder {
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
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
            .map(crate::Card::rank)
            .fold(Posterior::default(), |post, rank| post.add(rank, 1.0))
    }
}
