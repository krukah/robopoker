use super::{
    action::Action, info::Info, player::Player, policy::Policy, profile::Profile, Probability,
    Utility,
};

pub(crate) trait Optimizer {
    fn update_regret(&mut self, info: &Self::OInfo);
    fn update_policy(&mut self, info: &Self::OInfo);

    fn current_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn instant_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn updated_regret(&self, info: &Self::OInfo, action: &Self::OAction) -> Utility;
    fn updated_policy(&self, info: &Self::OInfo) -> Self::OPolicy;

    fn regret_vector(&self, info: &Self::OInfo) -> Vec<Utility>;
    fn policy_vector(&self, info: &Self::OInfo) -> Vec<Probability>;

    type OProfile: Profile;
    type OPlayer: Player;
    type OAction: Action;
    type OInfo: Info;
    type OPolicy: Policy;
}
