/// How a decision was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    Acted,
    Expired,
}
impl Prompt {
    pub fn expired(self) -> bool {
        self == Self::Expired
    }
}
