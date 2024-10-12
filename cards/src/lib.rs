#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub mod board;
pub mod card;
pub mod deck;
pub mod evaluator;
pub mod hand;
pub mod hands;
pub mod hole;
pub mod isomorphism;
pub mod kicks;
pub mod observation;
pub mod permutation;
pub mod rank;
pub mod ranking;
pub mod street;
pub mod strength;
pub mod suit;
