/// A type's own slice of an [`Embedder`](crate::Embedder)'s row: `WIDTH` values, the same
/// meaning at the same offset in every row. A composite concatenates its parts' blocks.
pub trait Block {
    const WIDTH: usize;

    fn values(self) -> impl Iterator<Item = f32>;
}
