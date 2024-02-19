pub struct Infoset {
    board: Vec<Card>,
    history: History,
    hole: Hole,
}

pub struct Range<T> {
    range: [T; 52 * 51 / 2],
}

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

pub struct Tree {
    root: Box<Mutex<TreeNode>>,
    history: History,
    paths_considered: Vec<Vec<Action>>,
    paths_eliminated: Vec<Vec<Action>>,
}
