use crate::*;
use mccfr::Node;
use mccfr::*;
use subgame::*;

#[derive(Default)]
pub struct LeducEncoder;

impl CfrSpec for LeducEncoder {
    type F = LeducInfo;
}

impl Encoding for LeducEncoder {
    fn seed(&self, game: &GameOf<Self>) -> InfoOf<Self> {
        let acting = matches!(game.turn(), LeducTurn::Player(_));
        LeducPublic::leduc_info(acting, if acting { game.hole_rank(0) } else { Rank::J }, None, Spot::Open, None)
    }

    fn public<P>(&self, game: &GameOf<Self>, _: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        let (r1, r2) = game.spots();
        LeducPublic::new(matches!(game.turn(), LeducTurn::Player(_)), game.board_rank(), r1, r2)
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        game.hole_rank(match game.turn() {
            LeducTurn::Player(i) => i,
            _ => 0,
        })
    }

    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        match node.game().turn() {
            LeducTurn::Terminal => Branches::new(node.index(), *node.game(), std::iter::empty()),
            LeducTurn::Chance => Branches::new(node.index(), *node.game(), node.game().deals().map(LeducEdge::Deal)),
            _ => node.branches(),
        }
    }
}

impl<const D: usize> Frontier<D> for LeducEncoder {}

impl<const W: usize> WorldRestrict<W> for LeducEncoder {
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        let LeducTurn::Player(actor) = external else {
            return *observed;
        };
        Card::ALL
            .into_iter()
            .filter(|&c| c != observed.hole_card(1 - actor))
            .filter(|&c| observed.board() != Some(c))
            .map(|c| observed.with_card(actor, c))
            .map(|root| (root, root.hole_rank(actor)))
            .find(|(_, secret)| belief.remember(secret, world))
            .map_or(*observed, |(root, _)| root)
    }
}

impl LeducEncoder {
    pub fn baseline(&self, game: &LeducGame, external: LeducTurn) -> Posterior<Rank> {
        let LeducTurn::Player(actor) = external else {
            return Posterior::default();
        };
        Card::ALL
            .into_iter()
            .filter(|&c| c != game.hole_card(1 - actor))
            .filter(|&c| game.board() != Some(c))
            .map(crate::Card::rank)
            .fold(Posterior::default(), |post, rank| post.add(rank, 1.0))
    }
}
