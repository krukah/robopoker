use crate::cards::Deck;

pub enum Action {
    Check,
    Fold,
    Call(u32),
    Open(u32),
    Shove(u32),
    Raise(u32),
    Draw(Card),
}

pub struct Round {
    pot: u32,
    deck: Deck,
    board: Board,
    rotation: Rotation,
}
impl Round {
    fn new() -> Round {
        todo!()
    }

    fn apply(&mut self, action: Action) {
        let mut player = self.rotation.next();
        match action {
            Action::Check => (),
            Action::Fold => self.fold(player),
            Action::Draw(card) => self.board.accept(card),
            Action::Call(bet) | Action::Open(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                self.bet(&mut player, bet)
            }
        }
        if self.rotation.is_all_folded() {
            self.distribute();
        }
        if self.rotation.is_all_in() {
            self.deal();
        }
        if self.rotation.is_all_called() {
            self.deal();
        }
    }

    fn bet(&mut self, player: &mut RefCell<Player>, bet: u32) {
        self.pot += bet;
        player.borrow_mut().wager += bet;
        player.borrow_mut().stack -= bet;
    }

    fn fold(&mut self, player: &mut RefCell<Player>) {
        self.pot += player.borrow().wager;
        player.borrow_mut().wager = 0;
        player.borrow_mut().status = PlayerStatus::Folded;
    }

    fn deal(&mut self) {
        match self.board.street {
            Street::Pre => self.deal_holes(),
            Street::Flop => self.deal_three(),
            Street::Turn | Street::River => self.deal_one(),
        }
    }

    fn deal_holes(&mut self) {
        for player in self.rotation {
            let mut player = player.borrow_mut();
            player.cards.cards.push(self.deck.deal().unwrap());
            player.cards.cards.push(self.deck.deal().unwrap());
        }
    }

    fn deal_three(&mut self) {
        self.board.accept(self.deck.deal().unwrap());
        self.board.accept(self.deck.deal().unwrap());
        self.board.accept(self.deck.deal().unwrap());
    }

    fn deal_one(&mut self) {
        self.board.accept(self.deck.deal().unwrap());
    }
}
