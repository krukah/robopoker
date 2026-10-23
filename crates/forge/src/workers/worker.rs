use deuce::*;
use kicker::*;
use mccfr::*;
use monge::Density;
use nlhe::*;
use pokerkit::*;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio_postgres::Client;

/// Async MCCFR training with direct database access. Workers run
/// independently and may train concurrently against the same database.
///
/// Uses the Pluribus (Brown & Sandholm, Science 2019) configuration:
/// [`LinearRegret`] (Linear CFR = DCFR(1, 1, 1)) + [`LinearWeight`].
pub struct Worker {
    client: Arc<Client>,
    vertices: AtomicUsize,
    epochs: AtomicUsize,
    infosets: AtomicUsize,
    start: Instant,
}

impl Worker {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            vertices: AtomicUsize::new(0),
            epochs: AtomicUsize::new(0),
            infosets: AtomicUsize::new(0),
            start: Instant::now(),
        }
    }

    pub fn epoch(&self) -> usize {
        self.epochs().load(Ordering::Relaxed)
    }

    pub fn nodes(&self) -> usize {
        self.vertices().load(Ordering::Relaxed)
    }

    pub fn infos(&self) -> usize {
        self.infosets().load(Ordering::Relaxed)
    }

    pub fn elapsed(&self) -> u64 {
        self.start().elapsed().as_secs()
    }

    fn client(&self) -> &Arc<Client> {
        &self.client
    }

    fn vertices(&self) -> &AtomicUsize {
        &self.vertices
    }

    fn epochs(&self) -> &AtomicUsize {
        &self.epochs
    }

    fn infosets(&self) -> &AtomicUsize {
        &self.infosets
    }

    fn start(&self) -> Instant {
        self.start
    }

    fn inc_epoch(&self) {
        self.epochs().fetch_add(1, Ordering::Relaxed);
    }

    fn inc_nodes(&self) {
        self.vertices().fetch_add(1, Ordering::Relaxed);
    }

    fn inc_infos(&self) {
        self.infosets().fetch_add(1, Ordering::Relaxed);
    }
}

impl CfrSpec for Worker {
    type F = NlheInfo;
}

/// Database-backed strategy lookups. Only the data-access methods are
/// implemented here; reach/EV/regret use the trait defaults, which fan these
/// out via `join_all`.
#[async_trait::async_trait]
impl AsyncProfile for Worker {
    fn traverser(&self) -> NlheTurn {
        NlheTurn::from(self.epoch() % NlheTurn::players())
    }

    fn iteration(&self) -> usize {
        self.epoch()
    }

    async fn policy(&self, info: &NlheInfo) -> Over<NlheEdge, Policy> {
        let memory = self.client().memory(*info).await;
        let raw = info
            .public()
            .menu()
            .map(|e| {
                let r = memory.regret(e.as_ref());
                debug_assert!(!r.is_nan());
                debug_assert!(!r.is_infinite());
                (e, r.max(EPSILON))
            })
            .collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, r)| r).sum::<Utility>();
        raw.into_iter().map(|(e, r)| (e, r / denom)).collect()
    }

    async fn sample(&self, info: &NlheInfo) -> Over<NlheEdge, Policy> {
        let memory = self.client().memory(*info).await;
        let raw = info
            .public()
            .menu()
            .map(|e| {
                let w = memory.weight(e.as_ref());
                debug_assert!(!w.is_nan());
                debug_assert!(!w.is_infinite());
                (e, w.max(EPSILON))
            })
            .collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, w)| *w).sum::<Probability>() + self.smoothing();
        raw.into_iter()
            .map(|(e, w)| (e, w / self.temperature()))
            .map(|(e, w)| (e, w + self.smoothing()))
            .map(|(e, w)| (e, w / denom))
            .map(|(e, w)| (e, w.max(self.curiosity())))
            .collect()
    }

    async fn advice(&self, info: &NlheInfo) -> Over<NlheEdge, Policy> {
        let memory = self.client().memory(*info).await;
        let raw = info
            .public()
            .menu()
            .map(|e| {
                let w = memory.weight(e.as_ref());
                debug_assert!(!w.is_nan());
                debug_assert!(!w.is_infinite());
                (e, w.max(EPSILON))
            })
            .collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, w)| *w).sum::<Probability>();
        raw.into_iter().map(|(e, w)| (e, w / denom)).collect()
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
            .filter(|infoset| infoset.head().game().turn() == self.traverser())
            .inspect(|_| self.inc_infos())
            .collect::<Vec<_>>()
        {
            updates.extend(self.updates(self.update_vector(infoset).await).await);
        }
        updates
    }

    pub async fn step(&self) {
        self.client().submit(self.batch().await).await;
        self.client().advance().await;
        self.inc_epoch();
    }
}

// encoding operations
impl Worker {
    async fn encode(&self, game: &Game) -> Abstraction {
        self.client().encode(Isomorphism::from(game.sweat())).await
    }

    async fn seed(&self, game: &Game) -> NlheInfo {
        let secret = NlheSecret::from(self.encode(game).await);
        NlheInfo::join(NlhePublic::at(&NlheGame::from(*game), std::iter::empty()), secret)
    }

    async fn info(&self, tree: &Tree<NlheInfo>, leaf: Leaf<NlheGame>) -> NlheInfo {
        let (edge, ref game, head) = (leaf.edge(), leaf.game(), leaf.parent());
        let secret = NlheSecret::from(self.encode(game.as_ref()).await);
        NlheInfo::join(NlhePublic::at(game, std::iter::once(edge).chain(tree.at(head).map(mccfr::Jump::edge))), secret)
    }

    fn branches(&self, node: &Node<NlheInfo>) -> Branches<NlheGame> {
        node.branches()
    }
}

// exploration operations
impl Worker {
    async fn explore(&self, node: &Node<'_, NlheInfo>, branches: Branches<NlheGame>) -> Branches<NlheGame> {
        match (branches.len(), node.game().turn()) {
            (0, _) => branches,
            (_, p) if p == self.traverser() => branches,
            (_, p) if Turn::from(p) == Turn::Chance => self.explore_any(node, branches),
            (_, p) if p != self.traverser() => self.explore_one(node, branches).await,
            _ => panic!("at the disco"),
        }
    }

    fn explore_any(&self, node: &Node<'_, NlheInfo>, branches: Branches<NlheGame>) -> Branches<NlheGame> {
        use rand::Rng;
        debug_assert!(!branches.is_empty());
        let n = branches.len();
        branches.only(self.rng(node).random_range(0..n))
    }

    async fn explore_one(&self, node: &Node<'_, NlheInfo>, branches: Branches<NlheGame>) -> Branches<NlheGame> {
        use rand::distr::Distribution;
        use rand::distr::weighted::WeightedIndex;
        let samples = AsyncProfile::sample(self, node.info()).await;
        let index = WeightedIndex::new(
            branches
                .edges()
                .iter()
                .map(|edge| samples.density(edge))
                .map(|p| p.max(EPSILON)),
        )
        .expect("at least one policy > 0")
        .sample(&mut self.rng(node));
        branches.only(index)
    }

    fn rng(&self, node: &Node<'_, NlheInfo>) -> rand::rngs::SmallRng {
        use rand::SeedableRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        let ref mut hasher = DefaultHasher::new();
        AsyncProfile::iteration(self).hash(hasher);
        node.info().hash(hasher);
        node.seed().hash(hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}

// tree sampling
impl Worker {
    pub async fn tree(&self) -> Tree<NlheInfo> {
        let mut todo = Vec::new();
        let ref root = Game::root();
        // Workers create one tree per batch; id only has to differ from concurrent trees at the
        // same epoch.
        let mut tree = Tree::new(0);
        let node = tree.seed(self.seed(root).await, NlheGame::from(Game::root()));
        todo.extend(self.explore(&node, self.branches(&node)).await.leaves());
        self.inc_nodes();
        while let Some(leaf) = todo.pop() {
            let node = tree.grow(self.info(&tree, leaf).await, leaf);
            todo.extend(self.explore(&node, self.branches(&node)).await.leaves());
            self.inc_nodes();
        }
        tree
    }
}

// CFR vector calculations (uses AsyncProfile trait methods)
impl Worker {
    async fn update_vector(&self, infoset: InfoSet<NlheInfo>) -> Decisions<NlheInfo> {
        let policy = AsyncProfile::policy_vector(self, &infoset).await;
        let (regret, payoff) = AsyncProfile::dfs(self, &infoset).await;
        Decisions::new(infoset.info(), regret, policy, payoff)
    }
}

// update calculations
impl Worker {
    async fn updates(&self, cfr: Decisions<NlheInfo>) -> Vec<Record> {
        let info = cfr.info();
        let regret_vector = cfr.regret();
        let policy_vector = cfr.policy();
        let infoset_payoff = cfr.payoff();
        let epoch = self.epoch();
        let memory = self.client().memory(*info).await;
        regret_vector
            .iter()
            .map(|(edge, regret_delta)| {
                let policy_delta = policy_vector.density(edge);
                let action_payoff = infoset_payoff + regret_delta;
                let old_regret = memory.regret(edge.as_ref());
                let old_weight = memory.weight(edge.as_ref());
                let old_payoff = memory.payoff(edge.as_ref());
                let old_visits = memory.visits(edge.as_ref());
                let new_regret = LinearRegret::gain(old_regret, *regret_delta, epoch);
                let new_weight = LinearWeight::learn(old_weight, policy_delta, epoch);
                let new_payoff = LinearWeight::learn(old_payoff, action_payoff, epoch);
                Record::new(*info, Edge::from(*edge), new_weight, new_regret, new_payoff, old_visits + 1)
            })
            .collect()
    }
}
