use tree_sitter::{Node, Point};

/// Where a node sits in its source, zero-based.
pub(crate) trait Place {
    fn top(&self) -> usize;

    fn bottom(&self) -> usize;

    fn column(&self) -> usize;
}

impl Place for Node<'_> {
    fn top(&self) -> usize {
        let Point { row, .. } = self.start_position();
        row
    }

    fn bottom(&self) -> usize {
        let Point { row, .. } = self.end_position();
        row
    }

    fn column(&self) -> usize {
        let Point { column, .. } = self.start_position();
        column
    }
}
