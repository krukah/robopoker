use gameplay::engine::Engine;
use players::{human::Human, robot::Robot};
use std::rc::Rc;

pub mod cards;
pub mod evaluation;
pub mod gameplay;
pub mod players;
pub mod strategy;

fn main() {
    let mut engine = Engine::new();
    let human = Rc::new(Human);
    let robot = Rc::new(Robot);

    // engine.gain_seat(100, human.clone());
    engine.gain_seat(100, robot.clone());
    engine.gain_seat(100, robot.clone());
    engine.gain_seat(100, robot.clone());
    engine.gain_seat(100, robot.clone());
    engine.gain_seat(100, robot.clone());

    engine.start();
}
