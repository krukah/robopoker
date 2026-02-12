use rbp_core::Chips;
use rbp_gameplay::*;
use rbp_gameroom::*;
use dialoguer::Input;
use dialoguer::Select;

#[derive(Debug, Default)]
pub struct Human;

#[async_trait::async_trait]
impl Player for Human {
    async fn decide(&mut self, recall: &Partial) -> Action {
        let game = recall.head();
        let actions = game.legal();
        let labels = actions.iter().map(Action::label).collect::<Vec<_>>();
        let choice = Self::selection(&labels, &game);
        Self::resolve(&actions[choice], &game)
    }
    async fn notify(&mut self, event: &Event) {
        match event {
            Event::HandStart { hand, dealer, .. } => {
                println!("Hand #{} (dealer P{})", hand, dealer)
            }
            Event::HoleCards { hole, .. } => println!("Your cards: {}", hole),
            Event::Board { street, board, .. } => println!("{}: {}", street, board),
            Event::Action { seat, action, .. } => println!("P{}: {}", seat, action),
            Event::Decision { recall, .. } => println!("{}", recall),
            Event::Reveal {
                seat,
                hole: Some(h),
                ..
            } => println!("P{}: {}", seat, h),
            Event::Reveal {
                seat, hole: None, ..
            } => println!("P{}: mucks", seat),
            Event::HandEnd { winners, .. } => {
                for (p, c) in winners {
                    println!("P{} wins {}", p, c);
                }
            }
            Event::Disconnect(pos) => println!("P{}: disconnected", pos),
        }
    }
}

impl Human {
    fn selection(labels: &[&str], game: &Game) -> usize {
        Select::new()
            .with_prompt(format!("{}", game))
            .report(false)
            .items(labels)
            .default(0)
            .interact()
            .unwrap()
    }
    fn resolve(action: &Action, game: &Game) -> Action {
        match action {
            Action::Raise(_) => Self::sizing(game),
            action => *action,
        }
    }
    fn sizing(game: &Game) -> Action {
        let min = game.to_raise();
        let max = game.to_shove();
        let bet = Input::new()
            .with_prompt(format!("Raise [{}-{}]", min, max))
            .validate_with(|i: &String| -> Result<(), String> {
                let input = i
                    .parse::<Chips>()
                    .map_err(|_| String::from("Enter a positive integer"))?;
                if input < min {
                    return Err(format!("Minimum raise is {}", min));
                }
                if input > max {
                    return Err(format!("Maximum raise is {}", max));
                }
                Ok(())
            })
            .report(false)
            .interact()
            .unwrap()
            .parse::<Chips>()
            .unwrap();
        if bet == max {
            Action::Shove(max)
        } else {
            Action::Raise(bet)
        }
    }
}
