/// Which margin of the slot axis a label occupies: `Near` sits before the
/// tiles (top in landscape, left in portrait), `Far` sits after them.
/// Actor and street labels swap brinks under transposition — that rule
/// lives in one place, inside [`crate::Lay`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brink {
    Near,
    Far,
}
