use super::abstraction::NodeAbstraction;
use super::layer::Layer;
use crate::cards::observation::NodeObservation;
use crate::cards::street::Street;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use crate::mccfr::player::Player;
use crate::mccfr::profile::Profile;
use crate::play::game::Game;
use crate::Probability;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::collections::BTreeMap;
use std::hash::Hash;
/// need to figure out how to  onsturct this
/// psuedo harmonic action mapping for path abstraction
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct PathAbstraction(u64);

/// the product of
/// "information abstraction" and
/// "action absraction" are what we index the (regret, strategy, average, ...) on
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Abstraction {
    path: PathAbstraction,
    node: NodeAbstraction,
}
impl From<(PathAbstraction, NodeAbstraction)> for Abstraction {
    fn from(abstraction: (PathAbstraction, NodeAbstraction)) -> Self {
        Self {
            path: abstraction.0,
            node: abstraction.1,
        }
    }
}

pub struct Explorer(BTreeMap<NodeObservation, NodeAbstraction>);

impl Explorer {
    pub fn download() -> Self {
        let mut abstractions = BTreeMap::new();
        for street in Street::all() {
            for (o, a) in Layer::download_centroid(street.clone()) {
                abstractions.insert(o, a);
            }
        }
        Self(abstractions)
    }
    pub async fn upload() {
        Layer::outer()
            .await
            .upload() // river
            .inner()
            .upload() // turn
            .inner()
            .upload() // flop
            .inner()
            .upload() // preflop
    ;
    }

    /// sample children of a Node, according to the distribution defined by Profile.
    /// we use external chance sampling, AKA explore all children of the traversing Player,
    /// while only probing a single child for non-traverser Nodes.
    /// this lands us in a high-variance, cheap-traversal, low-memory solution,
    /// compared to chance sampling, internal sampling, or full tree sampling.
    ///
    /// i think this could also be modified into a recursive CFR calcuation
    pub fn sample(&self, node: &Node, profile: &Profile) -> Vec<(Data, Edge)> {
        let mut children = self.children(node);
        // terminal nodes have no children and we sample all possible actions for the traverser
        if node.player() == profile.walker() || children.is_empty() {
            children
        }
        // choose random child uniformly. this is specific to the game of poker,
        // where each action at chance node/info/buckets is uniformly likely.
        else if node.player() == Player::chance() {
            let ref mut rng = profile.rng(node);
            let n = children.len();
            let choice = rng.gen_range(0..n);
            let chosen = children.remove(choice);
            vec![chosen]
        }
        // choose child according to reach probabilities in strategy profile.
        // on first iteration, this is equivalent to sampling uniformly.
        else {
            let ref mut rng = profile.rng(node);
            let policy = children
                .iter()
                .map(|(_, edge)| profile.policy(node, edge))
                .collect::<Vec<Probability>>();
            let choice = WeightedIndex::new(policy)
                .expect("at least one policy > 0")
                .sample(rng);
            let chosen = children.remove(choice);
            vec![chosen]
        }
    }

    /// produce the children of a Node.
    /// we may need some Trainer-level references to produce children
    fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        let ref game = node.datum().game();
        let ref path = node.history().into_iter().collect::<Vec<&Edge>>();
        game.children()
            .into_iter()
            .map(|(g, a)| (g, Edge::from(a)))
            .map(|(g, e)| self.extend(g, e, path))
            .collect()
    }

    /// extend a path with an Edge
    /// wrap the (Game, Bucket) in a Data
    fn extend(&self, game: Game, edge: Edge, path: &Vec<&Edge>) -> (Data, Edge) {
        let mut history = path.clone();
        history.push(&edge);
        let data = self.data(game, history);
        (data, edge)
    }
    /// generate a Bucket from Game
    /// wrap the (Game, Bucket) in a Data
    fn data(&self, game: Game, path: Vec<&Edge>) -> Data {
        Data::from((game, self.bucket(&game, &path)))
    }
    fn bucket(&self, ref game: &Game, ref path: &Vec<&Edge>) -> Bucket {
        Bucket::from(Abstraction::from((
            self.path_abstraction(path),
            self.card_abstraction(game),
        )))
    }
    fn card_abstraction(&self, game: &Game) -> NodeAbstraction {
        let ref observation = NodeObservation::from(game);
        self.0
            .get(observation)
            .copied()
            .expect("download should have all Node observations")
    }
    fn path_abstraction(&self, path: &Vec<&Edge>) -> PathAbstraction {
        todo!("pseudoharmonic action mapping for path abstraction")
    }
}