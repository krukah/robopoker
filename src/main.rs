use engine::engine::Engine;

pub mod cards;
pub mod engine;
pub mod evaluation;
pub mod solver;

fn main() {
    let players = todo!();
    let engine = Engine::new(players);
    engine.run();
}
