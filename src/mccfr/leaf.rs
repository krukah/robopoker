use crate::gameplay::edge::Edge;
use crate::gameplay::game::Game;
use petgraph::adj::NodeIndex;

#[derive(Debug, Clone, Copy)]
pub struct Leaf {
    head: NodeIndex,
    edge: Edge,
    game: Game,
}
