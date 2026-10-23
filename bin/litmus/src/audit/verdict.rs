#[derive(Clone, Debug)]
pub(crate) enum Verdict {
    OnGrid,
    OffGrid,
    OffTree(String),
}
