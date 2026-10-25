/// Lifecycle phase of a live hand. Derived from `LiveGame` state — never
/// stored, never set explicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Waiting,
    Playing,
    Showdown,
    Settled,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use deuce::*;
    use pokerkit::*;

    impl LiveGame {
        fn fresh(epoch: u64) -> Self {
            let mut live = Self::default();
            live.start(epoch, Game::from_start(0, [STACK; N]));
            live
        }
    }

    #[test]
    fn start_resets_state() {
        let live = LiveGame::fresh(1);
        assert_eq!(live.epoch(), 1);
        assert_eq!(live.phase(), Phase::Waiting);
        assert!(live.holes().iter().all(Option::is_none));
        assert!(live.dealt().is_empty());
        assert!(live.actions().is_empty());
        assert!(live.settlements().is_empty());
    }

    #[test]
    fn deal_hole_transitions_to_playing() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        let h0 = deck.hole(&mut rand::rng());
        live.deal_hole(0, h0);
        assert_eq!(live.hole(0), Some(h0));
        assert_eq!(live.hole(1), None);
        assert_eq!(live.phase(), Phase::Playing);
    }

    #[test]
    fn deal_appends_draw_action() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole(&mut rand::rng()));
        live.deal_hole(1, deck.hole(&mut rand::rng()));
        live.act(Action::Call(1));
        live.act(Action::Check);
        live.deal(deck.deal(Street::Pref, &mut rand::rng()));
        assert_eq!(live.dealt().len(), 3);
        assert_eq!(live.game().street(), Street::Flop);
    }

    #[test]
    fn show_writes_to_shown_not_holes() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        let h1 = deck.hole(&mut rand::rng());
        live.show(1, h1);
        assert_eq!(live.hole(1), None);
        assert_eq!(live.shown(1), Some(h1));
    }

    #[test]
    fn settle_transitions_to_settled() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        let strength = Strength::from(Hand::from(deck.hole(&mut rand::rng())));
        live.settle(vec![Settlement::from((0, State::Folding, strength))]);
        assert_eq!(live.phase(), Phase::Settled);
    }

    #[test]
    fn consecutive_hands_reset_state() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        let h1 = deck.hole(&mut rand::rng());
        live.show(1, h1);
        assert_eq!(live.shown(1), Some(h1));
        live.start(2, Game::from_start(0, [STACK; N]));
        assert_eq!(live.epoch(), 2);
        assert_eq!(live.phase(), Phase::Waiting);
        assert!(live.holes().iter().all(Option::is_none));
        assert_eq!(live.shown(1), None);
        assert!(live.actions().is_empty());
    }

    #[test]
    fn visible_pov_logic() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        let h0 = deck.hole(&mut rand::rng());
        let h1 = deck.hole(&mut rand::rng());
        live.deal_hole(0, h0);
        live.deal_hole(1, h1);
        assert_eq!(live.visible(0, 0), Some(h0));
        assert_eq!(live.visible(1, 0), None);
        live.show(1, h1);
        assert_eq!(live.visible(1, 0), Some(h1));
        assert_eq!(live.visible(1, 1), Some(h1));
        assert_eq!(live.hole(1), Some(h1));
    }

    #[test]
    fn recall_builds_partial() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole(&mut rand::rng()));
        live.deal_hole(1, deck.hole(&mut rand::rng()));
        live.act(Action::Call(1));
        let witness = live.recall(0).expect("has hole cards");
        assert_eq!(witness.turn(), Turn::Choice(0));
        assert_eq!(witness.actions().len(), 1);
    }

    #[test]
    fn recall_none_without_hole_cards() {
        let live = LiveGame::fresh(1);
        assert!(live.recall(0).is_none());
    }

    #[test]
    fn act_tracks_choice_only() {
        let mut live = LiveGame::fresh(1);
        let mut deck = Deck::new();
        live.deal_hole(0, deck.hole(&mut rand::rng()));
        live.deal_hole(1, deck.hole(&mut rand::rng()));
        live.act(Action::Call(1));
        live.act(Action::Check);
        assert_eq!(live.actions().len(), 2);
        live.deal(deck.deal(Street::Pref, &mut rand::rng()));
        assert_eq!(live.actions().len(), 3);
        assert!(live.actions()[2].is_chance());
    }
}
