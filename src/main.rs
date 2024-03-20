use gameplay::engine::Engine;
use players::{human::Human, robot::Robot};
use std::rc::Rc;

pub mod cards;
pub mod evaluation;
pub mod gameplay;
pub mod players;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    let human = Rc::new(Human);
    let robot = Rc::new(Robot);

    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());
    engine.gain_seat(10000, robot.clone());

    engine.start();
}
