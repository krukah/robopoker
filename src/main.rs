use cfr::{clustering::layer::Layer, training::solver::Solver};

mod cards;
mod cfr;
mod evaluation;
mod play;
mod players;

pub type Utility = f32;
pub type Probability = f32;

#[tokio::main]
async fn main() {
    let ref rivr = Layer::river();
    // let ref turn = Layer::upper(rivr);
    // let ref flop = Layer::upper(turn);
    // let ref pref = Layer::upper(flop);

    // Postgres connection semantics
    let ref url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let ref pool = sqlx::PgPool::connect(url)
        .await
        .expect("database connection");

    // Abstraction generation

    // Async persistence
    rivr.save(pool).await;
    // rivr.save(pool).await;
    // turn.save(pool).await;
    // flop.save(pool).await;
    // pref.save(pool).await;

    // CFR training iterations
    Solver::new().solve(50_000);
}
