use super::bucket::Bucket;
use super::data::Data;
use super::node::Node;
use super::path::Path;
use super::recall::Recall;
use super::tree::Branch;
use super::tree::Tree;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::lookup::Lookup;
use crate::gameplay::game::Game;
use crate::Arbitrary;
use crate::Save;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Encoder(BTreeMap<Isomorphism, Abstraction>);

impl Encoder {
    /// generate a random root Game and use our learned
    /// clustering to lookup the corresponding Abstraction.
    /// then embed them together into a Data. note that we don't
    /// generate the Bucket yet, that happens in Tree, but maybe
    /// we should do it here.
    pub fn seed(&self) -> Data {
        let game = Game::root();
        let info = self.abstraction(&game);
        Data::from((game, info))
    }

    /// - use pseudo-harmonic mapping to convert Vec<Action> -> Vec<Edge> -> Path
    /// - use learned encoder lookup to convert Game -> Abstraction.
    /// - use variant of Node::continuations() to get Vec<Edge> -> Path
    pub fn bucket(&self, recall: &Recall) -> Bucket {
        let history = self.pseudoharmonics(recall);
        let present = self.abstraction(&recall.game());
        let choices = self.choices(recall);
        Bucket::from((history, present, choices))
    }

    /// lookup the Abstraction for a given Game. convert
    /// ( Game -> Observation -> Isomorphism ) -> Abstraction
    pub fn abstraction(&self, game: &Game) -> Abstraction {
        self.0
            .get(&Isomorphism::from(game.sweat()))
            .cloned()
            .expect(&format!("precomputed abstraction missing {}", game.sweat()))
    }

    /// use encoder lookup to convert an unabstracted
    /// Recall of a game history into an abstracted Tree.
    /// each Game in the sequence converts to a Node, and
    /// each Action converts to an Edge.
    ///
    /// keep in mind that the Recall object is *not* omniscient,
    /// so some of the assumptions about the transparent self-play
    /// nature of Tree may not hold.
    fn replay(&self, recall: &Recall) -> Tree {
        todo!("create a Tree from the vector of Actions in the Spot")
    }

    /// lossy conversion from granular Action to coarse Edge.
    /// we depend on the pot size as of the Game state where
    /// the Action is applied, and always compare the size of the
    /// Action::Raise(_) to the pot to yield an [Odds] value.
    fn pseudoharmonics(&self, recall: &Recall) -> Path {
        todo!("use pseudo-harmonic mapping to convert Recall -> Vec<(Game, Action)> -> Vec<Edge> -> Path")
    }

    /// under the game tree constraints parametrized in lib.rs,
    /// what are the possible continuations of the Game given its
    /// full history? i.e. can we raise, and by how much.
    fn choices(&self, recall: &Recall) -> Path {
        todo!("use variant of Node::continuations() to get Vec<Edge> -> Path.
            note: Node::edgifies, Node::actionize, Node::continuations
                              these could kinda move to Game, there's just a ::subgame() dependency in ::raises()")
    }

    /// unfiltered set of possible children of a Node,
    /// conditional on its History (# raises, street granularity).
    /// the head Node is attached to the Tree stack-recursively,
    /// while the leaf Data is generated here with help from Sampler.
    /// Rust's ownership makes this a bit awkward but for very good reason!
    /// It has forced me to decouple global (Path) from local (Data)
    /// properties of Tree sampling, which makes lots of sense and is stronger model.
    /// broadly goes from Edge -> Action -> Game -> Abstraction
    pub fn branches(&self, node: &Node) -> Vec<Branch> {
        node.branches()
            .into_iter()
            .map(|(e, g)| (e, g, self.abstraction(&g)))
            .map(|(e, g, x)| (e, Data::from((g, x))))
            .map(|(e, d)| (e, d, node.index()))
            .map(|(e, d, n)| Branch(d, e, n))
            .collect()
    }
}

impl Save for Encoder {
    fn name() -> &'static str {
        unreachable!("saving happens at a lower level, composed of 4 street-level Lookup saves")
    }
    fn save(&self) {
        unreachable!("saving happens at a lower level, composed of 4 street-level Lookup saves")
    }
    fn make(_: Street) -> Self {
        unreachable!("you have no buisiness making an encoding from scratch")
    }
    fn done(_: Street) -> bool {
        Street::all()
            .iter()
            .copied()
            .all(|street| Lookup::done(street))
    }
    fn load(_: Street) -> Self {
        Self(
            Street::all()
                .iter()
                .copied()
                .map(Lookup::load)
                .map(BTreeMap::from)
                .fold(BTreeMap::default(), |mut map, l| {
                    map.extend(l);
                    map
                })
                .into(),
        )
    }
}

impl Arbitrary for Encoder {
    fn random() -> Self {
        const S: usize = 128;
        Self(
            (0..)
                .map(|_| Isomorphism::random())
                .map(|i| (i, Abstraction::random()))
                .filter(|(i, a)| i.0.street() == a.street())
                .take(S)
                .collect::<BTreeMap<_, _>>()
                .into(),
        )
    }
}
