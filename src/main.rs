use cfr::training::solver::Solver;

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

pub type Utility = f32;
pub type Probability = f32;

fn main() {
    Solver::new().solve(50_000);
}
