#![allow(dead_code)]
use std::rc::Rc;

use engine::{engine::Engine, hand::Hand, player::Robot, seat::Seat};

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    let actor = Rc::new(Robot {});
    let mut hand = Hand::new(vec![
        Seat::new(actor.clone(), 1_000, 0),
        Seat::new(actor.clone(), 1_000, 1),
        Seat::new(actor.clone(), 1_000, 2),
        Seat::new(actor.clone(), 1_000, 3),
    ]);
    engine.play(&mut hand);
}
