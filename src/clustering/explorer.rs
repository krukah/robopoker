use super::abstraction::NodeAbstraction;
use super::layer::Layer;
use crate::cards::observation::NodeObservation;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::play::action::Action;
use crate::play::game::Game;
use std::collections::BTreeMap;

pub struct Explorer(BTreeMap<NodeObservation, NodeAbstraction>);

impl Explorer {
    pub async fn download() -> Self {
        todo!("try to load ~1.2TB of Obs -> Abs map into memory, lmao")
    }
    pub async fn upload() {
        Layer::outer()
            .await
            .upload() // river
            .await
            .inner()
            .upload() // turn
            .await
            .inner()
            .upload() // flop
            .await
            .inner()
            .upload() // preflop
            .await;
    }

    pub fn children(&self, game: &Game, ref history: Vec<&Edge>) -> Vec<(Data, Edge)> {
        game.options()
            .into_iter()
            .map(|action| (game.consider(action), action))
            .map(|(child, birth)| self.explore(child, birth, history))
            .collect()
    }

    fn explore(&self, game: Game, action: Action, history: &Vec<&Edge>) -> (Data, Edge) {
        let ref edge = Edge::from(action);
        let mut history = history.clone();
        history.push(edge);
        let data = self.data(game, history);
        let edge = edge.to_owned();
        (data, edge)
    }
    fn data(&self, game: Game, path: Vec<&Edge>) -> Data {
        Data::from((
            game,
            Bucket::from(Abstraction::from((
                self.path_abstraction(&path),
                self.card_abstraction(&game),
            ))),
        ))
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

/// need to figure out how to  onsturct this
/// psuedo harmonic action mapping for path abstraction
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct PathAbstraction(u64);

/// just staple the past actions, via ::history(), together
struct PathObservation(Vec<Edge>);
/// combination of path and card observation
/// uncompressed, but Abstractor will "compress" it
/// from its learned hierachical KMEANS EMD mapping
/// and also psuedo harmonic action mapping
struct Observation(PathObservation, NodeObservation);
impl From<Observation> for (PathObservation, NodeObservation) {
    fn from(observation: Observation) -> Self {
        (observation.0, observation.1)
    }
}
