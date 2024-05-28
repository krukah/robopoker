use crate::cfr::rps::trainer::RpsTrainer;
use cfr::traits::learning::trainer::Trainer;
use gameplay::engine::Table;

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

#[tokio::main]
async fn main() {
    RpsTrainer::new().train(10_000);

    let mut engine = Table::new();
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);

    engine.play();
}
