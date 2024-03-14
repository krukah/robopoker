pub struct Human;

impl Human {}
impl Player for Human {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        let choices = &seat.valid_actions(hand);
        let selection = Select::new()
            .with_prompt(format!("YOUR TURN\n{}", seat))
            .report(false)
            .items(&choices[..])
            .default(0)
            .interact()
            .unwrap();
        choices[selection].clone()
    }
}
impl Debug for Human {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Human")
    }
}

use crate::gameplay::{action::Action, hand::Hand, player::Player, seat::Seat};
use dialoguer::Select;
use std::fmt::{Debug, Formatter, Result};
