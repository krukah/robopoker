use super::bucket::Bucket;
use crate::mccfr::info::Info;
use crate::mccfr::node::Node;
use std::collections::BTreeMap;

pub struct Partition(pub BTreeMap<Bucket, Info>);
impl Partition {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn witness(&mut self, node: &Node) {
        self.0
            .entry(node.bucket().clone())
            .or_insert_with(Info::new)
            .add(node.index());
    }
}

// impl<'tree> Partition<'tree> {
//     fn graph(&self) -> &'tree DiGraph<Data, Edge> {
//         todo!("once Info comes with lifetime this can be implemented trivially")
//     }
// }
// impl From<&Tree> for Partition {
//     fn from(tree: &Tree) -> Self {

//     }
// }
