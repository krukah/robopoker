//! Prints the game-tree fingerprint for each regime. Two trainers agreeing on
//! this can share a blueprint table; two that disagree must not.
//!
//! `cargo run -p kicker --example fingerprint`
use kicker::Pluribus;
use kicker::Regimen;
use kicker::Slumbot;

fn main() {
    for regimen in [&Pluribus as &dyn Regimen, &Slumbot] {
        println!("{}\n{}\n", regimen.digest(), regimen.config());
    }
}
