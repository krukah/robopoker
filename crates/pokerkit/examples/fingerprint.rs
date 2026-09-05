//! Prints the game-tree fingerprint. Two trainers agreeing on this can share a
//! blueprint table; two that disagree must not.
//!
//! `cargo run -p pokerkit --example fingerprint`

fn main() {
    println!("{}", pokerkit::fingerprint(pokerkit::Regime::Pluribus));
    println!("{}", pokerkit::config_string(pokerkit::Regime::Pluribus));
}
