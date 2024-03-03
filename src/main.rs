use engine::engine::Engine;

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let mut engine = Engine::new();
    engine.play();
}
