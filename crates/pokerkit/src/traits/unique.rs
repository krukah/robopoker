#![allow(dead_code)]

use super::*;

pub trait Unique<T = Self> {
    fn id(&self) -> ID<T>;
}
