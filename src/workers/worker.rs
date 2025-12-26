use super::*;
use crate::cards::*;
use crate::database::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::transport::Density;
use crate::*;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio_postgres::Client;

/// Worker implements async MCCFR training with direct database access.
/// Each worker runs independently, reading/writing to PostgreSQL.
/// Multiple workers can train concurrently on the same database.
pub struct Worker {
    client: Arc<Client>,
    nodes: AtomicUsize,
    epoch: AtomicUsize,
    infos: AtomicUsize,
    start: Instant,
}

impl Worker {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            nodes: AtomicUsize::new(0),
            epoch: AtomicUsize::new(0),
            infos: AtomicUsize::new(0),
            start: Instant::now(),
        }
    }
    pub fn epoch(&self) -> usize {
        self.epoch.load(Ordering::Relaxed)
    }
    pub fn nodes(&self) -> usize {
        self.nodes.load(Ordering::Relaxed)
    }
    pub fn infos(&self) -> usize {
        self.infos.load(Ordering::Relaxed)
    }
    pub fn elapsed(&self) -> u64 {
        self.start.elapsed().as_secs()
    }
    fn inc_epoch(&self) {
        self.epoch.fetch_add(1, Ordering::Relaxed);
    }
    fn inc_nodes(&self) {
        self.nodes.fetch_add(1, Ordering::Relaxed);
    }
    fn inc_infos(&self) {
        self.infos.fetch_add(1, Ordering::Relaxed);
    }
    fn walker(&self) -> Turn {
        match self.epoch() % 2 {
            0 => Turn::Choice(0),
            _ => Turn::Choice(1),
        }
    }
}

// main training interface
impl Worker {
    pub async fn batch(&self) -> Vec<Record> {
        let mut updates = Vec::new();
        for infoset in self
            .tree()
            .await
            .partition()
            .into_values()
            .filter(|infoset| infoset.head().game().turn() == self.walker())
            .inspect(|_| self.inc_infos())
            .collect::<Vec<_>>()
        {
            updates.extend(self.updates(self.counterfactual(infoset).await).await);
        }
        updates
    }

    pub async fn step(&self) {
        self.client.submit(self.batch().await).await;
        self.client.advance().await;
        self.inc_epoch();
    }
}

// encoding operations
impl Worker {
    async fn encode(&self, game: &Game) -> Abstraction {
        self.client.encode(Isomorphism::from(game.sweat())).await
    }

    async fn seed(&self, game: &Game) -> Info {
        let present = self.encode(game).await;
        let history = Path::default();
        let choices = Info::futures(game, 0);
        Info::from((history, present, choices))
    }

    async fn info(&self, tree: &Tree<Turn, Edge, Game, Info>, leaf: Branch<Edge, Game>) -> Info {
        let (edge, ref game, head) = leaf;
        let history = std::iter::once(edge)
            .chain(tree.at(head).map(|(_, e)| e))
            .take(crate::MAX_DEPTH_SUBGAME)
            .collect::<Path>()
            .rev()
            .collect::<Path>();
        let choices = Info::futures(game, Info::depth(&history));
        let present = self.encode(game).await;
        Info::from((history, present, choices))
    }

    fn branches(&self, node: &Node<Turn, Edge, Game, Info>) -> Vec<Branch<Edge, Game>> {
        node.branches()
    }
}

// batch strategy calculations (single DB round trip per info)
impl Worker {
    /// Fetch all accumulated values for an info in one query.
    async fn memory(&self, info: &Info) -> Memory {
        self.client.memory(*info).await
    }
    /// Compute policy distribution for all edges (single DB round trip).
    async fn policy(&self, info: &Info) -> Policy<Edge> {
        let memory = self.memory(info).await;
        let denom = info
            .edges()
            .iter()
            .map(|e| memory.regret(e))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .map(|r| r.max(crate::POLICY_MIN))
            .sum::<Utility>();
        info.edges()
            .into_iter()
            .map(|e| (e, memory.regret(&e)))
            .map(|(e, r)| (e, r.max(crate::POLICY_MIN)))
            .map(|(e, r)| (e, r / denom))
            .collect()
    }
    /// Compute sampling distribution for all edges (single DB round trip).
    async fn sample(&self, info: &Info) -> Policy<Edge> {
        let memory = self.memory(info).await;
        let denom = info
            .edges()
            .iter()
            .map(|e| memory.policy(e))
            .inspect(|p| assert!(!p.is_nan()))
            .inspect(|p| assert!(!p.is_infinite()))
            .map(|p| p.max(crate::POLICY_MIN))
            .sum::<Probability>()
            + self.activation();
        info.edges()
            .into_iter()
            .map(|e| (e, memory.policy(&e)))
            .map(|(e, p)| (e, p.max(crate::POLICY_MIN)))
            .map(|(e, p)| (e, p * self.threshold()))
            .map(|(e, p)| (e, p + self.activation()))
            .map(|(e, p)| (e, p / denom))
            .map(|(e, p)| (e, p.max(self.exploration())))
            .collect()
    }
}

// exploration operations
impl Worker {
    async fn explore(
        &self,
        node: &Node<'_, Turn, Edge, Game, Info>,
        branches: Vec<Branch<Edge, Game>>,
    ) -> Vec<Branch<Edge, Game>> {
        match (branches.len(), node.game().turn()) {
            (0, _) => branches,
            (_, p) if p == self.walker() => branches,
            (_, p) if p == Turn::Chance => self.explore_any(node, branches),
            (_, p) if p != self.walker() => self.explore_one(node, branches).await,
            _ => panic!("at the disco"),
        }
    }

    fn explore_any(
        &self,
        node: &Node<'_, Turn, Edge, Game, Info>,
        branches: Vec<Branch<Edge, Game>>,
    ) -> Vec<Branch<Edge, Game>> {
        use rand::Rng;
        assert!(!branches.is_empty());
        let mut choices = branches;
        vec![choices.remove(self.rng(node.info()).random_range(0..choices.len()))]
    }

    async fn explore_one(
        &self,
        node: &Node<'_, Turn, Edge, Game, Info>,
        branches: Vec<Branch<Edge, Game>>,
    ) -> Vec<Branch<Edge, Game>> {
        use rand::distr::Distribution;
        use rand::distr::weighted::WeightedIndex;
        let mut choices = branches;
        let samples = self.sample(node.info()).await;
        vec![
            choices.remove(
                WeightedIndex::new(
                    choices
                        .iter()
                        .map(|(edge, _, _)| samples.density(edge))
                        .map(|p| p.max(POLICY_MIN))
                        .collect::<Vec<_>>(),
                )
                .expect("at least one policy > 0")
                .sample(&mut self.rng(node.info())),
            ),
        ]
    }

    fn rng(&self, info: &Info) -> rand::rngs::SmallRng {
        use rand::SeedableRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        let ref mut hasher = DefaultHasher::new();
        self.epoch().hash(hasher);
        info.hash(hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}

// reach probability calculations
// chance nodes are filtered because:
// 1. chance sampling is uniform (1/n) in both reach and sampling
// 2. these terms cancel in relative_value = reach / sampling
// 3. filtering avoids wasteful database round trips for Edge::Draw
impl Worker {
    async fn relative_reach(
        &self,
        root: &Node<'_, Turn, Edge, Game, Info>,
        leaf: &Node<'_, Turn, Edge, Game, Info>,
    ) -> Probability {
        let path = leaf
            .into_iter()
            .take_while(|(p, _)| p != root)
            .filter(|(p, _)| p.game().turn().is_choice())
            .collect::<Vec<_>>();
        path.iter()
            .zip(futures::future::join_all(path.iter().map(|(p, _)| self.policy(p.info()))).await)
            .map(|((_, e), p)| p.density(e))
            .product()
    }
    async fn expected_reach(&self, root: &Node<'_, Turn, Edge, Game, Info>) -> Probability {
        let path = root
            .into_iter()
            .filter(|(p, _)| p.game().turn().is_choice())
            .collect::<Vec<_>>();
        path.iter()
            .zip(futures::future::join_all(path.iter().map(|(p, _)| self.policy(p.info()))).await)
            .map(|((_, e), p)| p.density(e))
            .product()
    }
    async fn cfactual_reach(&self, root: &Node<'_, Turn, Edge, Game, Info>) -> Probability {
        let path = root
            .into_iter()
            .filter(|(p, _)| p.game().turn().is_choice())
            .filter(|(p, _)| p.game().turn() != self.walker())
            .collect::<Vec<_>>();
        path.iter()
            .zip(futures::future::join_all(path.iter().map(|(p, _)| self.policy(p.info()))).await)
            .map(|((_, e), p)| p.density(e))
            .product()
    }
    async fn sampling_reach(&self, leaf: &Node<'_, Turn, Edge, Game, Info>) -> Probability {
        let path = leaf
            .into_iter()
            .filter(|(p, _)| p.game().turn().is_choice())
            .filter(|(p, _)| p.game().turn() != self.walker())
            .collect::<Vec<_>>();
        path.iter()
            .zip(futures::future::join_all(path.iter().map(|(p, _)| self.sample(p.info()))).await)
            .map(|((_, e), s)| s.density(e))
            .product()
    }

    async fn ancestor_reach(
        &self,
        root: &Node<'_, Turn, Edge, Game, Info>,
        kids: &[Node<'_, Turn, Edge, Game, Info>],
    ) -> Probability {
        futures::future::join_all(kids.iter().map(|leaf| self.relative_value(root, leaf)))
            .await
            .into_iter()
            .sum::<Utility>()
    }
}

// utility calculations
impl Worker {
    async fn relative_value(
        &self,
        root: &Node<'_, Turn, Edge, Game, Info>,
        leaf: &Node<'_, Turn, Edge, Game, Info>,
    ) -> Utility {
        leaf.game().payoff(root.game().turn()) * self.relative_reach(root, leaf).await
            / self.sampling_reach(leaf).await
    }

    async fn expected_value(&self, root: &Node<'_, Turn, Edge, Game, Info>) -> Utility {
        assert!(self.walker() == root.game().turn());
        let ref descendants = root.descendants();
        self.ancestor_reach(root, descendants).await * self.expected_reach(root).await
    }

    async fn cfactual_value(
        &self,
        root: &Node<'_, Turn, Edge, Game, Info>,
        edge: &Edge,
    ) -> Utility {
        assert!(self.walker() == root.game().turn());
        let ref descendants = root
            .follow(edge)
            .expect("edge belongs to outgoing branches")
            .descendants();
        self.ancestor_reach(root, descendants).await * self.cfactual_reach(root).await
    }

    async fn node_gain(
        &self,
        root: &Node<'_, Turn, Edge, Game, Info>,
        edge: &Edge,
        expected: Utility,
    ) -> Utility {
        assert!(self.walker() == root.game().turn());
        self.cfactual_value(root, edge).await - expected
    }
}

// tree sampling
impl Worker {
    pub async fn tree(&self) -> Tree<Turn, Edge, Game, Info> {
        let mut todo = Vec::new();
        let ref root = Game::root();
        let mut tree = Tree::default();
        let node = tree.seed(self.seed(root).await, Game::root());
        todo.extend(self.explore(&node, self.branches(&node)).await);
        self.inc_nodes();
        while let Some(leaf) = todo.pop() {
            let node = tree.grow(self.info(&tree, leaf).await, leaf);
            todo.extend(self.explore(&node, self.branches(&node)).await);
            self.inc_nodes();
        }
        tree
    }
}

// CFR vector calculations
impl Worker {
    async fn counterfactual(
        &self,
        infoset: InfoSet<Turn, Edge, Game, Info>,
    ) -> Counterfactual<Edge, Info> {
        (
            infoset.info(),
            self.regret_vector(&infoset).await,
            self.policy_vector(&infoset).await,
        )
    }

    /// Compute regret gains for all edges. Pre-computes expected values
    /// for all roots to avoid redundant computation.
    async fn regret_vector(&self, infoset: &InfoSet<Turn, Edge, Game, Info>) -> Policy<Edge> {
        let ref span = infoset.span();
        let ref expected =
            futures::future::join_all(span.iter().map(|r| self.expected_value(r))).await;
        let edges = infoset.info().edges();
        let gains = futures::future::join_all(edges.iter().map(|edge| async move {
            futures::future::join_all(
                span.iter()
                    .zip(expected.iter())
                    .map(|(root, &ev)| self.node_gain(root, edge, ev)),
            )
            .await
            .into_iter()
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .sum::<Utility>()
        }))
        .await;
        edges.into_iter().zip(gains).collect()
    }

    /// Compute policy vector using single DB round trip (batch optimized).
    async fn policy_vector(&self, infoset: &InfoSet<Turn, Edge, Game, Info>) -> Policy<Edge> {
        let info = infoset.info();
        let memory = self.memory(&info).await;
        let regrets = info
            .edges()
            .into_iter()
            .map(|e| (e, memory.regret(&e)))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .map(|(e, r)| (e, r.max(crate::POLICY_MIN)))
            .collect::<Vec<_>>();
        let denom = regrets
            .iter()
            .map(|(_, r)| r)
            .inspect(|r| assert!(**r >= 0.))
            .sum::<Utility>();
        regrets
            .into_iter()
            .map(|(a, r)| (a, r / denom))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect()
    }
}

// update calculations
impl Worker {
    async fn updates(&self, cfr: Counterfactual<Edge, Info>) -> Vec<Record> {
        let ref info = cfr.0;
        let ref regret_vector = cfr.1;
        let ref policy_vector = cfr.2;
        let memory = self.memory(info).await;
        regret_vector
            .iter()
            .map(|(edge, regret_delta)| {
                let policy_delta = policy_vector.density(edge);
                let old_regret = memory.regret(edge);
                let old_policy = memory.policy(edge);
                let discount_r = self.discount(Some(old_regret));
                let discount_p = self.discount(None);
                let new_regret = (old_regret * discount_r + regret_delta).max(crate::REGRET_MIN);
                let new_policy = (old_policy * discount_p + policy_delta).max(crate::POLICY_MIN);
                Record {
                    info: *info,
                    edge: *edge,
                    policy: new_policy,
                    regret: new_regret,
                }
            })
            .collect()
    }
}

// discount parameters
impl Worker {
    fn discount(&self, regret: Option<Utility>) -> f32 {
        let t = self.epoch() as f32;
        let p = self.period() as f32;
        match regret {
            None => (t / (t + 1.)).powf(self.gamma()),
            Some(_) if t % p != 0. => 1.,
            Some(r) if r > 0. => (t / p).powf(self.alpha()) / ((t / p).powf(self.alpha()) + 1.),
            Some(r) if r < 0. => (t / p).powf(self.omega()) / ((t / p).powf(self.omega()) + 1.),
            Some(_) => (t / p) / ((t / p) + 1.),
        }
    }

    fn alpha(&self) -> f32 {
        1.5
    }

    fn omega(&self) -> f32 {
        0.5
    }

    fn gamma(&self) -> f32 {
        1.5
    }

    fn period(&self) -> usize {
        1
    }

    fn threshold(&self) -> crate::Entropy {
        crate::SAMPLING_THRESHOLD
    }

    fn activation(&self) -> crate::Energy {
        crate::SAMPLING_ACTIVATION
    }

    fn exploration(&self) -> Probability {
        crate::SAMPLING_EXPLORATION
    }
}
