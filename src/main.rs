use cfr::training::trainer::Trainer;

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

pub type Utility = f32;
pub type Probability = f32;

fn main() {
    Trainer::train(50_000);
}
