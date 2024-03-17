pub struct Human;

impl Human {
    fn raise(&self, seat: &Seat, hand: &Hand) -> u32 {
        Input::new()
            .with_prompt("Amount ")
            .report(false)
            .validate_with(|i: &String| -> Result<(), &str> {
                match i.parse::<u32>() {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Enter a NUMBER"),
                }
            })
            .validate_with(|i: &String| -> Result<(), &str> {
                match i.parse::<u32>().unwrap() >= seat.min_raise(hand) {
                    true => Ok(()),
                    false => Err("Raise too small"),
                }
            })
            .validate_with(|i: &String| -> Result<(), &str> {
                match i.parse::<u32>().unwrap() <= seat.max_raise(hand) {
                    true => Ok(()),
                    false => Err("Raise too large"),
                }
            })
            .interact()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }
}
impl Player for Human {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        // get valid actions
        let choices = seat
            .valid_actions(hand)
            .iter()
            .filter(|a| match a {
                Action::Shove(_, _) => false,
                _ => true,
            })
            .map(|a| match a {
                Action::Fold(_) => "Fold",
                Action::Check(_) => "Check",
                Action::Call(_, _) => "Call",
                Action::Raise(_, _) => "Raise",
                _ => unreachable!(),
            })
            .collect::<Vec<&str>>();
        let selection = Select::new()
            .with_prompt(format!("\nYOU HOLD {}", seat.hole))
            .report(false)
            .items(choices.as_slice())
            .default(0)
            .interact()
            .unwrap();
        match choices[selection] {
            "Fold" => Action::Fold(seat.position),
            "Check" => Action::Check(seat.position),
            "Call" => Action::Call(seat.position, seat.to_call(hand)),
            "Shove" => Action::Shove(seat.position, seat.to_shove(hand)),
            "Raise" => {
                let raise = self.raise(seat, hand);
                let shove = seat.to_shove(hand);
                match raise == shove {
                    true => Action::Shove(seat.position, shove),
                    false => Action::Raise(seat.position, raise),
                }
            }
            _ => unreachable!(),
        }
    }
}
impl Debug for Human {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Human")
    }
}

use crate::gameplay::{action::Action, hand::Hand, player::Player, seat::Seat};
use dialoguer::{Input, Select};
use std::fmt::{Debug, Formatter};
use std::result::Result;
