pub struct Human;
impl Human {}
impl Player for Human {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        let choices = &seat.valid_actions(hand);
        let selection = Select::new()
            .with_prompt(format!("       {}", seat.cards()))
            .items(&choices[..])
            .default(0)
            .interact()
            .unwrap();
        choices[selection].clone()
    }
}
impl Debug for Human {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Human")
    }
}
use crate::gameplay::{action::Action, hand::Hand, player::Player, seat::Seat};
use dialoguer::Select;
use std::fmt::Debug;
