use cfr::{clustering::layer::Layer, training::solver::Solver};

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

pub type Utility = f32;
pub type Probability = f32;

#[allow(unused_variables)]
fn main() {
    let ref river = Layer::river();
    let ref turn = Layer::upper(river);
    let ref flop = Layer::upper(turn);
    let ref pref = Layer::upper(flop);

    Solver::new().solve(50_000);
}
