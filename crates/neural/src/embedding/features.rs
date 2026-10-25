/// An infoset as a network's input row: one value per [`Embedder::inputs`](crate::Embedder::inputs).
#[derive(Debug, Clone, PartialEq)]
pub struct Features(Vec<f32>);

impl FromIterator<f32> for Features {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = f32>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for Features {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        let Self(values) = self;
        values.into_iter()
    }
}
