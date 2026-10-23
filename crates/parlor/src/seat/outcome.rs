pub(crate) enum Outcome {
    Decided(kicker::Action),
    Restart(kicker::Witness),
}
