use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_nlhe::*;
use rbp_mccfr::*;
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
/// Implements [`AsyncProfile`] for database-backed strategy lookups.
/// The trait provides default implementations for reach probabilities,
/// expected values, and regret calculations — Worker only implements
/// the core data access methods (`policy`, `sample`, `advice`).
///
/// Uses Pluribus (Brown & Sandholm, Science 2019) configuration:
/// - [`LinearRegret`] — Linear CFR = DCFR(1, 1, 1), as used in Pluribus
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
}

/// AsyncProfile implementation for database-backed strategy lookups.
///
/// This implementation provides the core data access methods that fetch
/// accumulated regrets and weights from PostgreSQL. All the reach
/// probability, expected value, and regret calculations use the default
/// trait implementations, which call these methods via `join_all`.
#[async_trait::async_trait]
impl AsyncProfile for Worker {
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;

    fn traverser(&self) -> NlheTurn {
        NlheTurn::from(self.epoch() % NlheTurn::players())
    }

    fn iteration(&self) -> usize {
        self.epoch()
    }

    async fn policy(&self, info: &NlheInfo) -> Policy<NlheEdge> {
        let memory = self.client.memory(*info).await;
        let raw = info
            .public()
            .choices()
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

    async fn sample(&self, info: &NlheInfo) -> Policy<NlheEdge> {
        let memory = self.client.memory(*info).await;
        let raw = info
            .public()
            .choices()
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

    async fn advice(&self, info: &NlheInfo) -> Policy<NlheEdge> {
        let memory = self.client.memory(*info).await;
        let raw = info
            .public()
            .choices()
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
        let geometry = Geometry::from_game(game);
        NlheInfo::from((subgame, present, choices, geometry))
    }

    async fn info(
        &self,
        tree: &Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        leaf: Leaf<NlheEdge, NlheGame>,
    ) -> NlheInfo {
        let (edge, ref game, head) = leaf;
        let subgame = std::iter::once(edge)
            .chain(tree.at(head).map(|a| a.edge()))
            .take_while(|e| e.is_choice())
            .map(|e| Edge::from(e))
            .collect::<Path>()
            .rev()
            .collect::<Path>();
        let present = self.encode(game.as_ref()).await;
        let choices = game.as_ref().choices(subgame.aggression());
        let geometry = Geometry::from_game(game.as_ref());
        NlheInfo::from((subgame, present, choices, geometry))
    }

    fn branches(
        &self,
        node: &Node<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Vec<Leaf<NlheEdge, NlheGame>> {
        node.branches()
    }
}

// exploration operations
impl Worker {
    async fn explore(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Leaf<NlheEdge, NlheGame>>,
    ) -> Vec<Leaf<NlheEdge, NlheGame>> {
        match (branches.len(), node.game().turn()) {
            (0, _) => branches,
            (_, p) if p == self.traverser() => branches,
            (_, p) if Turn::from(p) == Turn::Chance => self.explore_any(node, branches),
            (_, p) if p != self.traverser() => self.explore_one(node, branches).await,
            _ => panic!("at the disco"),
        }
    }

    fn explore_any(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Leaf<NlheEdge, NlheGame>>,
    ) -> Vec<Leaf<NlheEdge, NlheGame>> {
        use rand::Rng;
        debug_assert!(!branches.is_empty());
        let mut choices = branches;
        vec![choices.remove(self.rng(node).random_range(0..choices.len()))]
    }

    async fn explore_one(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
        branches: Vec<Leaf<NlheEdge, NlheGame>>,
    ) -> Vec<Leaf<NlheEdge, NlheGame>> {
        use rand::distr::Distribution;
        use rand::distr::weighted::WeightedIndex;
        let mut choices = branches;
        let samples = AsyncProfile::sample(self, node.info()).await;
        vec![
            choices.remove(
                WeightedIndex::new(
                    choices
                        .iter()
                        .map(|(edge, _, _)| samples.density(edge))
                        .map(|p| p.max(EPSILON))
                        .collect::<Vec<_>>(),
                )
                .expect("at least one policy > 0")
                .sample(&mut self.rng(node)),
            ),
        ]
    }

    fn rng(
        &self,
        node: &Node<'_, NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> rand::rngs::SmallRng {
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
    pub async fn tree(&self) -> Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo> {
        let mut todo = Vec::new();
        let ref root = Game::root();
        // Workers create one tree per batch; id only has to differ from
        // concurrent trees at the same epoch. Since each worker runs
        // independently and the epoch is already in the RNG hash, 0 is fine.
        let mut tree = Tree::new(0);
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

// CFR vector calculations (uses AsyncProfile trait methods)
impl Worker {
    async fn update_vector(
        &self,
        infoset: InfoSet<NlheTurn, NlheEdge, NlheGame, NlheInfo>,
    ) -> Decisions<NlheEdge, NlheInfo> {
        let policy = AsyncProfile::policy_vector(self, &infoset).await;
        let (regret, payoff) = AsyncProfile::dfs(self, &infoset).await;
        Decisions {
            info: infoset.info(),
            regret,
            policy,
            payoff,
        }
    }
}

// update calculations
impl Worker {
    async fn updates(&self, cfr: Decisions<NlheEdge, NlheInfo>) -> Vec<Record> {
        let ref info = cfr.info;
        let ref regret_vector = cfr.regret;
        let ref policy_vector = cfr.policy;
        let infoset_payoff = cfr.payoff;
        let epoch = self.epoch();
        let memory = self.client.memory(*info).await;
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
                Record {
                    info: *info,
                    edge: Edge::from(*edge),
                    weight: new_weight,
                    regret: new_regret,
                    payoff: new_payoff,
                    visits: old_visits + 1,
                }
            })
            .collect()
    }
}
