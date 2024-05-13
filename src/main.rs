use gameplay::engine::Table;

mod cards;
mod cfrm;
mod evaluation;
mod gameplay;
mod players;

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
