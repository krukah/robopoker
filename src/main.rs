use gameplay::engine::Table;

pub mod cards;
pub mod cfrm;
pub mod evaluation;
pub mod gameplay;
pub mod players;
pub mod strategy;

#[tokio::main]
async fn main() {
    let mut engine = Table::new();
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);
    engine.gain_seat(100);

    engine.play();
}
