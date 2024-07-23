#![allow(dead_code)]

use super::action::Edge;

pub struct Data(pub usize);

pub struct Child {
    pub data: Data,
    pub edge: Edge,
}
