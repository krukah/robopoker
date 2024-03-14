#![allow(dead_code)]
use std::rc::Rc;

use engine::{
    engine::Engine,
    player::{Human, Robot},
};

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    let human = Rc::new(Human);
    let robot = Rc::new(Robot);

    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, human.clone());

    engine.start();
}
