use super::bucket::Bucket;
use super::tree::Tree;
use crate::mccfr::info::Info;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Partition(BTreeMap<Bucket, Info>);

impl From<Tree> for Partition {
    fn from(tree: Tree) -> Self {
        let mut info = BTreeMap::new();
        let mut tree = tree;
        tree.partition();
        let tree = Arc::new(tree);
        for node in tree.all().iter().filter(|n| n.player() == tree.walker()) {
            info.entry(node.bucket())
                .or_insert_with(|| Info::from(tree.clone()))
                .add(node.index());
        }
        Self(info)
    }
}

impl From<Partition> for Vec<Info> {
    fn from(infosets: Partition) -> Self {
        infosets.0.into_values().collect()
    }
}
