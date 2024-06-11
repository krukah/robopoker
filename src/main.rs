use cfr::training::trainer::X;

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

pub type Utility = f32;
pub type Probability = f32;

fn main() {
    let mut trainer = X::new(50_000);
    trainer.train();
}
