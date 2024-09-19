#![allow(dead_code)]
use crate::play::action::Action;
use crate::play::game::Game;
use crate::play::Chips;
use dialoguer::Input;
use dialoguer::Select;

#[derive(Debug)]
pub struct Human;

impl Human {
    pub fn act(spot: &Game) -> Action {
        return Self::random(spot);
        let ref choices = Self::available(spot);
        let choice = Self::selection(choices, spot);
        Self::choose(choices, choice, spot)
    }

    fn random(game: &Game) -> Action {
        use rand::seq::SliceRandom;
        let ref mut rng = rand::thread_rng();
        game.options()
            .choose(rng)
            .copied()
            .expect("decision node has options")
    }

    fn raise(spot: &Game) -> Chips {
        Input::new()
            .with_prompt(Self::infoset(spot))
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

    fn infoset(spot: &Game) -> String {
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

    fn available(spot: &Game) -> Vec<&str> {
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

    fn selection(choices: &[&str], spot: &Game) -> usize {
        Select::new()
            .with_prompt(Self::infoset(spot))
            .report(false)
            .items(choices)
            .default(0)
            .interact()
            .unwrap()
    }

    fn choose(choices: &[&str], selection: usize, spot: &Game) -> Action {
        match choices[selection] {
            "Fold" => Action::Fold,
            "Check" => Action::Check,
            "Call" => Action::Call(spot.to_call()),
            "Shove" => Action::Shove(spot.to_shove()),
            "Raise" => {
                let raise = Self::raise(spot);
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
