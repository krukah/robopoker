use cfr::{clustering::layer::Layer, training::solver::Solver};

mod cards;
mod cfr;
mod evaluation;
mod gameplay;
mod players;

pub type Utility = f32;
pub type Probability = f32;

#[tokio::main]
async fn main() {
    // Abstraction generation
    let ref river = Layer::river();
    let ref turn = Layer::upper(river);
    let ref flop = Layer::upper(turn);
    let ref pref = Layer::upper(flop);

    // Postgres connection semantics
    // I'm only ::clone() for visual parity tbh
    let ref url = std::env::var("DB_CONNECTION")
        .expect("missing enironment: DB_CONNECTION")
        .clone();
    let ref pool = sqlx::PgPool::connect(url)
        .await
        .expect("database connection");

    // Async persistence
    river.save(pool).await;
    turn.save(pool).await;
    flop.save(pool).await;
    pref.save(pool).await;

    // CFR training iterations
    Solver::new().solve(50_000);
}
