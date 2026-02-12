use rbp_cards::*;
use rbp_core::*;
use rbp_database::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::Density;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio_postgres::Client;

/// Worker implements async MCCFR training with direct database access.
/// Each worker runs independently, reading/writing to PostgreSQL.
/// Multiple workers can train concurrently on the same database.
///
/// Uses Pluribus configuration:
/// - [`PluribusRegret`] — hybrid regret (no discount positive, t/(t+1) negative)
/// - [`LinearWeight`] — linear weighting for average strategy
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
    fn walker(&self) -> NlheTurn {
        NlheTurn::from(self.epoch() % 2)
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

    async fn seed(&self, game: &Game) -> NlheInfo {
        let present = self.encode(game).await;
        let subgame = Path::default();
        let choices = game.choices(0);
        NlheInfo::from((subgame, present, choices))
    }

    async fn info(
        &self,
        tree: &Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        leaf: Branch<NlheEdge, NlheGame>,
    ) -> NlheInfo {
        let (edge, ref game, head) = leaf;
        let subgame = std::iter::once(edge)
            .chain(tree.at(head).map(|(_, e)| e))
            .take_while(|e| e.is_choice())
            .map(|e| Edge::from(e))
            .collect::<Path>()
            .rev()
            .collect::<Path>();
        let present = self.encode(game.as_ref()).await;
        let choices = game.as_ref().choices(subgame.aggression());
        NlheInfo::from((subgame, present, choices))
    }

    fn branches(
        &self,
        node: &Node<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Vec<Branch<NlheEdge, NlheGame>> {
        node.branches()
    }
}

// batch strategy calculations (single DB round trip per info)
impl Worker {
    /// Fetch all accumulated values for an info in one query.
    async fn memory(&self, info: &NlheInfo) -> Memory {
        self.client.memory(*info).await
    }
    /// Compute policy distribution for all edges (single DB round trip).
    async fn policy(&self, info: &NlheInfo) -> Policy<NlheEdge> {
        let memory = self.memory(info).await;
        let denom = info
            .public()
            .choices()
            .iter()
            .map(|e| memory.regret(e.as_ref()))
            .inspect(|r| debug_assert!(!r.is_nan()))
            .inspect(|r| debug_assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Utility>();
        info.public()
            .choices()
            .into_iter()
            .map(|e| (e, memory.regret(e.as_ref())))
            .map(|(e, r)| (e, r.max(POLICY_MIN)))
            .map(|(e, r)| (e, r / denom))
            .collect()
    }
    /// Compute sampling distribution for all edges (single DB round trip).
    async fn sample(&self, info: &NlheInfo) -> Policy<NlheEdge> {
        let memory = self.memory(info).await;
        let denom = info
            .public()
            .choices()
            .iter()
            .map(|e| memory.weight(e.as_ref()))
            .inspect(|p| debug_assert!(!p.is_nan()))
            .inspect(|p| debug_assert!(!p.is_infinite()))
            .map(|p| p.max(POLICY_MIN))
            .sum::<Probability>()
            + self.smoothing();
        info.public()
            .choices()
            .into_iter()
            .map(|e| (e, memory.weight(e.as_ref())))
            .map(|(e, w)| (e, w.max(POLICY_MIN)))
            .map(|(e, w)| (e, w / self.temperature()))
            .map(|(e, w)| (e, w + self.smoothing()))
            .map(|(e, w)| (e, w / denom))
            .map(|(e, w)| (e, w.max(self.curiosity())))
            .collect()
    }
}

// exploration operations
impl Worker {
    async fn explore(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Branch<NlheEdge, NlheGame>>,
    ) -> Vec<Branch<NlheEdge, NlheGame>> {
        match (branches.len(), node.game().turn()) {
            (0, _) => branches,
            (_, p) if p == self.walker() => branches,
            (_, p) if Turn::from(p) == Turn::Chance => self.explore_any(node, branches),
            (_, p) if p != self.walker() => self.explore_one(node, branches).await,
            _ => panic!("at the disco"),
        }
    }

    fn explore_any(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Branch<NlheEdge, NlheGame>>,
    ) -> Vec<Branch<NlheEdge, NlheGame>> {
        use rand::Rng;
        debug_assert!(!branches.is_empty());
        let mut choices = branches;
        vec![choices.remove(self.rng(node.info()).random_range(0..choices.len()))]
    }

    async fn explore_one(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Branch<NlheEdge, NlheGame>>,
    ) -> Vec<Branch<NlheEdge, NlheGame>> {
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

    fn rng(&self, info: &NlheInfo) -> rand::rngs::SmallRng {
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
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        leaf: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
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
    async fn cfactual_reach(
        &self,
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Probability {
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
    async fn sampling_reach(
        &self,
        leaf: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Probability {
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

    async fn ancestor_value(
        &self,
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        kids: &[Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>],
    ) -> Utility {
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
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        leaf: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Utility {
        debug_assert!(
            leaf.game().turn() == NlheTurn::terminal(),
            "worker builds full trees; leaves must be terminal"
        );
        CfrGame::payoff(leaf.game(), root.game().turn())
            * self.relative_reach(root, leaf).await
            / self.sampling_reach(leaf).await
    }

    /// Policy-weighted expected utility at this node (Bellman equation).
    ///
    /// V(I) = Σ_a π(a) × Q(I,a)
    async fn expected_value(
        &self,
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        let ref edges = root.outgoing();
        let policy = self.policy(root.info()).await;
        let values =
            futures::future::join_all(edges.iter().map(|e| self.cfactual_value(root, e))).await;
        edges
            .iter()
            .zip(values)
            .map(|(e, v)| policy.density(e) * v)
            .sum()
    }

    async fn cfactual_value(
        &self,
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        edge: &NlheEdge,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        let ref descendants = root
            .follow(edge)
            .expect("edge belongs to outgoing branches")
            .descendants();
        self.ancestor_value(root, descendants).await * self.cfactual_reach(root).await
    }

    async fn node_gain(
        &self,
        root: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        edge: &NlheEdge,
        expected: Utility,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        self.cfactual_value(root, edge).await - expected
    }
}

// tree sampling
impl Worker {
    pub async fn tree(&self) -> Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo> {
        let mut todo = Vec::new();
        let ref root = Game::root();
        let mut tree = Tree::default();
        let node = tree.seed(self.seed(root).await, NlheGame::from(Game::root()));
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
        infoset: InfoSet<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Counterfactual<NlheEdge, NlheInfo> {
        Counterfactual {
            info: infoset.info(),
            regret: self.regret_vector(&infoset).await,
            policy: self.policy_vector(&infoset).await,
            evalue: self.infoset_value(&infoset).await,
        }
    }
    /// Compute the expected value of an information set under current strategy.
    async fn infoset_value(
        &self,
        infoset: &InfoSet<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Utility {
        futures::future::join_all(infoset.span().iter().map(|r| self.expected_value(r)))
            .await
            .into_iter()
            .sum()
    }

    /// Compute regret gains for all edges. Pre-computes expected values
    /// for all roots to avoid redundant computation.
    ///
    /// Iterates per-node over each node's actual outgoing edges, since
    /// sampling may have expanded different edges at different nodes.
    async fn regret_vector(
        &self,
        infoset: &InfoSet<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Policy<NlheEdge> {
        let ref span = infoset.span();
        let ref expected =
            futures::future::join_all(span.iter().map(|r| self.expected_value(r))).await;
        let pairs = span
            .iter()
            .zip(expected.iter())
            .flat_map(|(root, &evalue)| {
                root.outgoing()
                    .into_iter()
                    .cloned()
                    .map(move |edge| (root.clone(), edge, evalue))
            })
            .collect::<Vec<_>>();
        let gains = futures::future::join_all(
            pairs
                .iter()
                .map(|(root, edge, evalue)| self.node_gain(root, edge, *evalue)),
        )
        .await;
        pairs
            .into_iter()
            .map(|(_, edge, _)| edge)
            .zip(gains)
            .inspect(|(_, r)| debug_assert!(!r.is_nan()))
            .inspect(|(_, r)| debug_assert!(!r.is_infinite()))
            .fold(
                std::collections::HashMap::<NlheEdge, Utility>::new(),
                |mut acc, (edge, gain)| {
                    *acc.entry(edge).or_default() += gain;
                    acc
                },
            )
            .into_iter()
            .collect()
    }

    /// Compute policy vector using single DB round trip (batch optimized).
    async fn policy_vector(
        &self,
        infoset: &InfoSet<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Policy<NlheEdge> {
        let info = infoset.info();
        let memory = self.memory(&info).await;
        let regrets = info
            .public()
            .choices()
            .into_iter()
            .map(|e| (e, memory.regret(e.as_ref())))
            .inspect(|(_, r)| debug_assert!(!r.is_nan()))
            .inspect(|(_, r)| debug_assert!(!r.is_infinite()))
            .map(|(e, r)| (e, r.max(POLICY_MIN)))
            .collect::<Vec<_>>();
        let denom = regrets
            .iter()
            .map(|(_, r)| r)
            .inspect(|r| debug_assert!(**r >= 0.))
            .sum::<Utility>();
        regrets
            .into_iter()
            .map(|(a, r)| (a, r / denom))
            .inspect(|(_, p)| debug_assert!(*p >= 0.))
            .inspect(|(_, p)| debug_assert!(*p <= 1.))
            .collect()
    }
}

// update calculations
impl Worker {
    async fn updates(&self, cfr: Counterfactual<NlheEdge, NlheInfo>) -> Vec<Record> {
        let ref info = cfr.info;
        let ref regret_vector = cfr.regret;
        let ref policy_vector = cfr.policy;
        let infoset_evalue = cfr.evalue;
        let epoch = self.epoch();
        let memory = self.memory(info).await;
        regret_vector
            .iter()
            .map(|(edge, regret_delta)| {
                let policy_delta = policy_vector.density(edge);
                let action_evalue = infoset_evalue + regret_delta;
                let old_regret = memory.regret(edge.as_ref());
                let old_weight = memory.weight(edge.as_ref());
                let old_evalue = memory.evalue(edge.as_ref());
                let old_counts = memory.counts(edge.as_ref());
                let new_regret = PluribusRegret::gain(old_regret, *regret_delta, epoch);
                let new_weight = LinearWeight::learn(old_weight, policy_delta, epoch);
                let new_evalue = LinearWeight::learn(old_evalue, action_evalue, epoch);
                Record {
                    info: *info,
                    edge: Edge::from(*edge),
                    weight: new_weight,
                    regret: new_regret,
                    evalue: new_evalue,
                    counts: old_counts + 1,
                }
            })
            .collect()
    }
}

// sampling parameters
impl Worker {
    fn temperature(&self) -> Entropy {
        SAMPLING_TEMPERATURE
    }
    fn smoothing(&self) -> Energy {
        SAMPLING_SMOOTHING
    }
    fn curiosity(&self) -> Probability {
        SAMPLING_CURIOSITY
    }
}
