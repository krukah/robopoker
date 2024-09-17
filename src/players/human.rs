#![allow(dead_code)]
use crate::play::action::Action;
use crate::play::spot::Spot;
use crate::play::Chips;
use dialoguer::Input;
use dialoguer::Select;

#[derive(Debug)]
pub struct Human;

impl Human {
    fn raise(&self, spot: &Spot) -> Chips {
        Input::new()
            .with_prompt(self.infoset(spot))
            .validate_with(|i: &String| -> Result<(), &str> {
                let input = match i.parse::<Chips>() {
                    Ok(value) => value,
                    Err(_) => return Err("Enter a positive integer"),
                };
                if input < spot.to_raise() {
                    return Err("Raise too small");
                }
                if input > spot.to_shove() {
                    return Err("Raise too large");
                }
                Ok(())
            })
            .report(false)
            .interact()
            .unwrap()
            .parse::<Chips>()
            .unwrap()
    }

    fn infoset(&self, spot: &Spot) -> String {
        format!(
            "\nBOARD      {}\nCARDS      {}\nPOT        {}\nSTACK      {}\nTO CALL    {}\nMIN RAISE  {}\n\nAction",
            spot.board(),
            spot.actor().cards(),
            spot.pot(),
            spot.actor().stack(),
            spot.to_call(),
            spot.to_raise(),
        )
    }
    fn act(&self, spot: &Spot) -> Action {
        let ref choices = self.available(spot);
        let selection = self.selection(choices, spot);
        self.bind(choices, selection, spot)
    }

    fn available(&self, spot: &Spot) -> Vec<&str> {
        spot.options()
            .iter()
            .map(|a| match a {
                Action::Fold => "Fold",
                Action::Check => "Check",
                Action::Call(_) => "Call",
                Action::Raise(_) => "Raise",
                Action::Shove(_) => "Shove",
                _ => unreachable!(),
            })
            .collect::<Vec<&str>>()
    }

    fn selection(&self, choices: &[&str], spot: &Spot) -> usize {
        Select::new()
            .with_prompt(self.infoset(spot))
            .report(false)
            .items(choices)
            .default(0)
            .interact()
            .unwrap()
    }

    fn bind(&self, choices: &[&str], selection: usize, spot: &Spot) -> Action {
        match choices[selection] {
            "Fold" => Action::Fold,
            "Check" => Action::Check,
            "Call" => Action::Call(spot.to_call()),
            "Shove" => Action::Shove(spot.to_shove()),
            "Raise" => {
                let raise = self.raise(spot);
                let shove = spot.to_shove();
                if raise == shove {
                    Action::Shove(shove)
                } else {
                    Action::Raise(raise)
                }
            }
            _ => unreachable!(),
        }
    }
}
