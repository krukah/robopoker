use super::bucket::Bucket;
use super::info::InfoSet;
use super::tree::Tree;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Partition(BTreeMap<Bucket, InfoSet>);

impl From<Tree> for Partition {
    fn from(tree: Tree) -> Self {
        let mut info = BTreeMap::new();
        let ref tree = Arc::new(tree);
        for node in tree
            .all()
            .iter()
            .filter(|n| n.children().len() > 0)
            .filter(|n| n.player() == tree.walker_onlyinpartition())
        {
            info.entry(node.bucket().clone())
                .or_insert_with(|| InfoSet::from(tree.clone()))
                .add(node.index());
        }
        Self(info)
    }
}

impl From<Partition> for Vec<InfoSet> {
    fn from(infosets: Partition) -> Self {
        infosets.0.into_values().collect()
    }
}
