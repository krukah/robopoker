use super::{edge::Edge, turn::Turn};

impl crate::cfr::traits::info::Info for Turn {
    type E = Edge;
    type T = Turn;

    fn choices(&self) -> Vec<Self::E> {
        if *self == Turn::Terminal {
            vec![]
        } else {
            vec![Edge::R, Edge::P, Edge::S]
        }
    }
}
