use super::player::Player;
use super::profile::Profile;
use super::tree::Tree;

pub struct Trainer(usize, Profile);

impl Trainer {
    pub fn train(epochs: usize) {
        let ref mut this = Self(0, Profile::empty());
        while this.0 < epochs {
            this.0 += 1;
            let ref mut profile = this.1;
            for ref infoset in Tree::sample(profile) {
                if infoset.node().player() == this.walker() {
                    this.1.update_regret(infoset, this.0);
                    this.1.update_policy(infoset, this.0);
                } else {
                    continue;
                }
            }
        }
    }
    fn walker(&self) -> &Player {
        match self.0 % 2 {
            0 => &Player::P1,
            _ => &Player::P2,
        }
    }
}
