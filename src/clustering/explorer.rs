use super::abstraction::NodeAbstraction;
use super::layer::Layer;
use crate::cards::observation::NodeObservation;
use crate::cards::street::Street;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::play::action::Action;
use crate::play::game::Game;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

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
const BUFFER: usize = 1024 * 1024;

impl Explorer {
    pub fn download() -> Self {
        let mut map = BTreeMap::new();
        for street in Street::all() {
            println!("downloading street {}", street);
            let file = File::open(format!("centroid_{}.bin", street)).expect("file open");
            let mut reader = BufReader::with_capacity(BUFFER, file);
            let mut buffer = [0u8; 16];
            while reader.read_exact(&mut buffer).is_ok() {
                let obs_u64 = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
                let abs_u64 = u64::from_le_bytes(buffer[8..16].try_into().unwrap());
                let observation = NodeObservation::from(obs_u64 as i64);
                let abstraction = NodeAbstraction::from(abs_u64 as i64);
                map.insert(observation, abstraction);
            }
        }
        Self(map)
    }
    pub async fn upload() {
        Layer::outer()
            .await
            .upload() // river
            .inner()
            .upload() // turn
            .inner()
            .inner()
            .upload() // flop
            .inner()
            .upload() // preflop
    ;
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
