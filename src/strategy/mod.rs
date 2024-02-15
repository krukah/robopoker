pub struct Infoset {
    board: Vec<Card>,
    history: History,
    hole: Hole,
}

pub struct Range<T> {
    range: [T; 52 * 51 / 2],
}

pub struct Tree {
    root: Box<Mutex<TreeNode>>,
    history: History,
    paths_considered: Vec<Vec<Action>>,
    paths_eliminated: Vec<Vec<Action>>,
}
