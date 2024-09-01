#![allow(dead_code)]

use super::edge::Edge;

pub struct Data(pub usize);

pub struct Child {
    pub data: Data,
    pub edge: Edge,
}
