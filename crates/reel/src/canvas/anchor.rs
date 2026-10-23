/// Horizontal text anchoring — inherently an x-axis affair, which is why it
/// is the one label property the orientation must decide directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    pub(crate) fn svg(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Middle => "middle",
            Self::End => "end",
        }
    }
}
