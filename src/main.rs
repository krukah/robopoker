use engine::{engine::Engine, seat::Seat};

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    for i in 0..4 {
        engine.add(Seat::new(1000, i));
    }
    engine.play();
}
