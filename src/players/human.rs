#![allow(dead_code)]

use crate::play::{action::Action, game::Game, seat::Seat};
use dialoguer::{Input, Select};

#[derive(Debug)]
pub struct Human;

impl Human {
    fn raise(&self, seat: &Seat, hand: &Game) -> u32 {
        Input::new()
            .with_prompt(self.infoset(seat, hand))
            .validate_with(|i: &String| -> Result<(), &str> {
                let input = match i.parse::<u32>() {
                    Ok(value) => value,
                    Err(_) => return Err("Enter a positive integer"),
                };
                if input < seat.min_raise(hand) {
                    return Err("Raise too small");
                }
                if input > seat.max_raise(hand) {
                    return Err("Raise too large");
                }
                Ok(())
            })
            .report(false)
            .interact()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }

    fn infoset(&self, seat: &Seat, hand: &Game) -> String {
        format!(
            "\nBOARD      {}\nCARDS      {}\nPOT        {}\nSTACK      {}\nTO CALL    {}\nMIN RAISE  {}\n\nAction",
            hand.head.board,
            seat.peek(),
            hand.head.pot,
            seat.stack(),
            seat.to_call(hand),
            seat.min_raise(hand),
        )
    }

    fn act(&self, seat: &Seat, hand: &Game) -> Action {
        // get valid actions
        let choices = seat
            .valid_actions(hand)
            .iter()
            .map(|a| match a {
                Action::Fold(_) => "Fold",
                Action::Check(_) => "Check",
                Action::Call(_, _) => "Call",
                Action::Raise(_, _) => "Raise",
                Action::Shove(_, _) => "Shove",
                _ => unreachable!(),
            })
            .collect::<Vec<&str>>();
        let selection = Select::new()
            .with_prompt(self.infoset(seat, hand))
            .report(false)
            .items(choices.as_slice())
            .default(0)
            .interact()
            .unwrap();
        match choices[selection] {
            "Fold" => Action::Fold(seat.position()),
            "Check" => Action::Check(seat.position()),
            "Call" => Action::Call(seat.position(), seat.to_call(hand)),
            "Shove" => Action::Shove(seat.position(), seat.to_shove(hand)),
            "Raise" => {
                let raise = self.raise(seat, hand);
                let shove = seat.to_shove(hand);
                match raise == shove {
                    true => Action::Shove(seat.position(), shove),
                    false => Action::Raise(seat.position(), raise),
                }
            }
            _ => unreachable!(),
        }
    }
}
