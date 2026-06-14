use super::*;
use atlas::*;
use regret::*;

#[derive(Default)]
pub struct LeducEncoder;

impl CfrEncoder for LeducEncoder {
    type T = LeducTurn;
    type E = LeducEdge;
    type G = LeducGame;
    type I = LeducInfo;
    const CHECK_RECALL: bool = true;

    fn seed(&self, game: &Self::G) -> Self::I {
        let acting = matches!(game.turn(), LeducTurn::Player(_));
        leduc_info(acting, if acting { game.hole_rank(0) } else { Rank::J }, None, Spot::Open, None)
    }

    fn resume<P>(&self, _past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        let acting = matches!(game.turn(), LeducTurn::Player(_));
        let actor = match game.turn() {
            LeducTurn::Player(i) => i,
            _ => 0,
        };
        let (r1, r2) = game.spots();
        leduc_info(acting, game.hole_rank(actor), game.board_rank(), r1, r2)
    }

    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        match node.game().turn() {
            LeducTurn::Terminal => vec![],
            LeducTurn::Chance => node
                .game()
                .deals()
                .map(|c| (LeducEdge::Deal(c), node.game().apply(LeducEdge::Deal(c)), node.index()))
                .collect(),
            _ => node.branches(),
        }
    }
}

impl EmbeddedHistory for LeducEncoder {}

impl<const W: usize> WorldRestrict<W> for LeducEncoder {
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G {
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
            .map(super::card::Card::rank)
            .fold(Posterior::default(), |post, rank| post.add(rank, 1.0))
    }
}
