#![allow(dead_code)]
use engine::{engine::Engine, game::Hand, seat::Seat};

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    let mut hand = Hand::new(vec![
        Seat::new(1_000, 0),
        Seat::new(1_000, 1),
        Seat::new(1_000, 2),
        Seat::new(1_000, 3),
    ]);
    engine.play(&mut hand);
}
