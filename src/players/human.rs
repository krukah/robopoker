#![allow(dead_code)]
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::Chips;
use dialoguer::Input;
use dialoguer::Select;

#[derive(Debug)]
pub struct Human;

impl Human {
    pub fn decide(game: &Game) -> Action {
        Self::random(game)
        // let ref choices = Self::available(game);
        // let choice = Self::selection(choices, game);
        // Self::choose(choices, choice, game)
    }

    fn random(game: &Game) -> Action {
        use rand::seq::SliceRandom;
        let ref mut rng = rand::thread_rng();
        game.legal()
            .choose(rng)
            .copied()
            .expect("decision node has options")
    }

    fn raise(game: &Game) -> Chips {
        Input::new()
            .with_prompt(Self::infoset(game))
            .validate_with(|i: &String| -> Result<(), &str> {
                let input = match i.parse::<Chips>() {
                    Ok(value) => value,
                    Err(_) => return Err("Enter a positive integer"),
                };
                if input < game.to_raise() {
                    return Err("Raise too small");
                }
                if input > game.to_shove() {
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

    fn infoset(game: &Game) -> String {
        format!(
            "\nBOARD      {}\nCARDS      {}\nPOT        {}\nSTACK      {}\nTO CALL    {}\nMIN RAISE  {}\n\nAction",
            game.board(),
            game.actor().cards(),
            game.pot(),
            game.actor().stack(),
            game.to_call(),
            game.to_raise(),
        )
    }

    fn available(game: &Game) -> Vec<&str> {
        game.legal()
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

    fn selection(choices: &[&str], game: &Game) -> usize {
        Select::new()
            .with_prompt(Self::infoset(game))
            .report(false)
            .items(choices)
            .default(0)
            .interact()
            .unwrap()
    }

    fn choose(choices: &[&str], selection: usize, game: &Game) -> Action {
        match choices[selection] {
            "Fold" => Action::Fold,
            "Check" => Action::Check,
            "Call" => Action::Call(game.to_call()),
            "Shove" => Action::Shove(game.to_shove()),
            "Raise" => {
                let raise = Self::raise(game);
                let shove = game.to_shove();
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
