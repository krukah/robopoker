pub mod traits;

/// ---
trait Edge: Copy + Clone + PartialEq + Eq {}

/// ---
trait Turn: Clone + Copy + PartialEq + Eq {
    fn chance() -> Self;
}

/// ---
trait Decision: Clone + Copy + PartialEq + Eq {
    type E: Edge;
    fn choices(&self) -> impl Iterator<Item = Self::E>;
}

/// ---
trait Info: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    type E: Edge;
    type T: Turn;
    fn decision(&self) -> impl Decision<E = Self::E>;
}

/// ---
trait Game: Clone + Copy {
    type E: Edge;
    type T: Turn;
    fn root() -> Self;
    fn player(&self) -> Self::T;
    fn payoff(&self, player: Self::T) -> crate::Utility;
}

/// ---
#[derive(Debug, Default)]
struct InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    root: Vec<petgraph::graph::NodeIndex>,
    tree: std::sync::Arc<Tree<T, E, G, I>>,
}
impl<T, E, G, I> InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<T = T, E = E>,
    I: Info<E = E, T = T>,
{
    fn from(tree: std::sync::Arc<Tree<T, E, G, I>>) -> Self {
        Self {
            root: Vec::new(),
            tree,
        }
    }
    fn next(&mut self) -> Option<Node<T, E, G, I>> {
        self.root.pop().map(|i| self.tree.at(i))
    }
    fn add(&mut self, index: petgraph::graph::NodeIndex) {
        self.root.push(index);
    }
    fn roots(&self) -> Vec<Node<T, E, G, I>> {
        self.root.iter().copied().map(|i| self.tree.at(i)).collect()
    }
    fn head(&self) -> Node<T, E, G, I> {
        self.tree
            .at(self.root.first().copied().expect("nodes in info"))
    }
    fn info(&self) -> I {
        self.head().info().clone()
    }
    fn choices(&self) -> impl Iterator<Item = E> {
        self.info()
            .decision()
            .choices()
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<T, E, G, I> From<std::sync::Arc<Tree<T, E, G, I>>> for InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn from(tree: std::sync::Arc<Tree<T, E, G, I>>) -> Self {
        Self::from(tree)
    }
}

/// ---
#[derive(Copy, Clone)]
struct Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    index: petgraph::graph::NodeIndex,
    graph: &'tree petgraph::graph::DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}
impl<'tree, T, E, G, I> Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn info(&self) -> &I {
        todo!("make impl generic over T, i.e. hoist type T to the struct level")
    }

    fn index(&self) -> petgraph::graph::NodeIndex {
        self.index
    }
    fn graph(&self) -> &'tree petgraph::graph::DiGraph<(G, I), E> {
        self.graph
    }
    fn inner(&self) -> &G {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid node index")
    }
    fn at(&self, index: petgraph::graph::NodeIndex) -> Node<'tree, T, E, G, I> {
        Self {
            index,
            graph: self.graph(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }

    fn next(&self) -> Option<(Node<'tree, T, E, G, I>, &'tree E)> {
        match (self.parent(), self.incoming()) {
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("live by the ship die by the ship"),
            (_, Some(_)) => unreachable!("live by the ship die by the ship"),
            (None, None) => None,
        }
    }
    fn parent(&self) -> Option<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|index| self.at(index))
    }
    fn incoming(&self) -> Option<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    fn follow(&self, edge: &E) -> Option<Node<'tree, T, E, G, I>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
    }
    fn outgoing(&self) -> Vec<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    fn children(&self) -> Vec<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    fn descendants(&self) -> Vec<Node<'tree, T, E, G, I>> {
        if self.children().is_empty() {
            vec![self.clone()]
        } else {
            self.children()
                .into_iter()
                .map(|child| child.descendants())
                .flatten()
                .collect()
        }
    }
}

/// ---
#[derive(Debug, Default)]
struct Tree<T, E, G, I>(
    petgraph::graph::DiGraph<(G, I), E>,
    std::marker::PhantomData<(T, I)>,
)
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>;

impl<T, E, G, I> Tree<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn all(&self) -> impl Iterator<Item = Node<T, E, G, I>> {
        self.0.node_indices().map(|n| self.at(n))
    }

    fn at(&self, index: petgraph::graph::NodeIndex) -> Node<T, E, G, I> {
        Node {
            index,
            graph: &self.0,
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }

    fn seed(&mut self, info: I, seed: G) -> Node<T, E, G, I> {
        let seed = self.0.add_node(seed);
        self.at(seed)
    }

    fn partition(self) -> std::collections::HashMap<I, InfoSet<T, E, G, I>> {
        let tree = std::sync::Arc::new(self);
        tree.all().filter(|n| n.children().len() > 0).fold(
            std::collections::HashMap::new(),
            |mut info, node| {
                info.entry(node.info().clone())
                    .or_insert_with(|| InfoSet::from(tree.clone()))
                    .add(node.index());
                info
            },
        )
    }
}
