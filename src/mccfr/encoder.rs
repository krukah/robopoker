use super::bucket::Bucket;
use super::data::Data;
use super::node::Node;
use super::recall::Recall;
use super::tree::Leaf;
use super::tree::Tree;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::lookup::Lookup;
use crate::gameplay::game::Game;
use crate::Arbitrary;
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
        let game = recall.head();
        let abstraction = self.abstraction(&game);
        let bucket = recall.bucket(abstraction);
        bucket
    }

    /// lookup the Abstraction for a given Game. convert
    /// ( Game -> Observation -> Isomorphism ) -> Abstraction
    pub fn abstraction(&self, game: &Game) -> Abstraction {
        self.0
            .get(&Isomorphism::from(game.sweat()))
            .cloned()
            .expect(&format!("precomputed abstraction missing {}", game.sweat()))
    }
    /// unfiltered set of possible children of a Node,
    /// conditional on its History (# raises, street granularity).
    /// the head Node is attached to the Tree stack-recursively,
    /// while the leaf Data is generated here with help from Sampler.
    /// Rust's ownership makes this a bit awkward but for very good reason!
    /// It has forced me to decouple global (Path) from local (Data)
    /// properties of Tree sampling, which makes lots of sense and is stronger model.
    pub fn sample(&self, node: &Node) -> Vec<Leaf> {
        node.branches()
            .into_iter()
            .map(|(e, g)| (e, g, self.abstraction(&g)))
            .map(|(e, g, a)| (e, Data::from((g, a))))
            .map(|(e, d)| (e, d, node.index()))
            .map(|(e, d, n)| Leaf(d, e, n))
            .collect()
    }

    /// use encoder lookup to convert an unabstracted
    /// Recall of a game history into an abstracted Tree.
    /// each Game in the sequence converts to a Node, and
    /// each Action converts to an Edge.
    ///
    /// keep in mind that the Recall object is *not* omniscient,
    /// so some of the assumptions about the transparent self-play
    /// nature of Tree may not hold.
    #[allow(unused)]
    fn replay(&self, recall: &Recall) -> Tree {
        todo!("maybe useful during test-time search?")
    }
}

impl Arbitrary for Encoder {
    fn random() -> Self {
        const N: usize = 128;
        Self(
            (0..)
                .map(|_| Isomorphism::random())
                .map(|i| (i, Abstraction::random()))
                .filter(|(i, a)| i.0.street() == a.street())
                .take(N)
                .collect::<BTreeMap<_, _>>()
                .into(),
        )
    }
}

#[cfg(feature = "native")]
impl crate::save::upload::Table for Encoder {
    fn name() -> String {
        Lookup::name()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        Lookup::columns()
    }
    fn sources() -> Vec<String> {
        Lookup::sources()
    }
    fn creates() -> String {
        Lookup::creates()
    }
    fn indices() -> String {
        Lookup::indices()
    }
    fn copy() -> String {
        Lookup::copy()
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
    fn save(&self) {
        unimplemented!("saving happens at Lookup level. composed of 4 street-level Lookup saves")
    }
    fn grow(_: Street) -> Self {
        unimplemented!("you have no business making an encoding from scratch, learn from kmeans")
    }
}
