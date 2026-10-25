use deuce::*;
use kicker::*;

/// A complete runout: the five board cards a hand would see.
///
/// Everything the log dealt is kept and only the streets it never reached are
/// sampled, so a hand that went to showdown contributes its real board and a
/// hand that ended preflop contributes a sampled one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World(pub(crate) Vec<Card>);
impl World {
    /// The five cards, in deal order.
    pub fn board(&self) -> &[Card] {
        &self.0
    }

    /// The cards a deal onto `street` turns over.
    pub(crate) fn deal(&self, street: Street) -> Option<Action> {
        self.0
            .get(street.n_board() - street.n_revealed()..street.n_board())
            .map(|cards| Action::Draw(Hand::from(cards.to_vec())))
    }
}
