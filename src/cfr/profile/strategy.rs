use crate::cfr::traits::action::E;
use crate::Probability;
use std::collections::HashMap;

pub(crate) struct S(pub HashMap<E, Probability>);

impl S {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
