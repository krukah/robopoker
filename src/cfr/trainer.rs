#![allow(unused)]
use super::profile::Profile;
use super::tree::Tree;

pub struct Trainer {
    tree: Tree,
    profile: Profile,
}

impl Trainer {
    fn tree_from_ref_mut_profile(profile: &mut Profile) -> Tree {
        todo!()
    }
}
