use crate::*;

/// A tabular MCCFR solver for any game: its encoder, a [`Book`] profile, and the regret, weight,
/// and sampling schedules, with `B` trees per batch. A game supplies only its encoder.
pub struct Tabular<N, R, W, S, const B: usize>
where
    N: Encoding,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    profile: Book<N::F>,
    encoder: N,
    schedules: std::marker::PhantomData<fn() -> (R, W, S)>,
}

impl<N, R, W, S, const B: usize> Default for Tabular<N, R, W, S, B>
where
    N: Encoding + Default,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn default() -> Self {
        Self::new(Book::default(), N::default())
    }
}

impl<N, R, W, S, const B: usize> CfrSpec for Tabular<N, R, W, S, B>
where
    N: Encoding,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type F = N::F;
}

/// The solver stands in for its encoder, so subgame wrappers can take either.
impl<N, R, W, S, const B: usize> Encoding for Tabular<N, R, W, S, B>
where
    N: Encoding,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn seed(&self, game: &GameOf<Self>) -> InfoOf<Self> {
        self.encoder().seed(game)
    }

    fn info(&self, tree: &Tree<Self::F>, leaf: LeafOf<Self>) -> InfoOf<Self> {
        self.encoder().info(tree, leaf)
    }

    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        self.encoder().public(game, past)
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.encoder().secret(game)
    }
}

impl<N, R, W, S, const B: usize> Grower for Tabular<N, R, W, S, B>
where
    N::F: 'static,
    N: Encoding + Send + Sync,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type P = Book<N::F>;
    type N = N;
    type S = S;

    fn encoder(&self) -> &N {
        self.encoder()
    }

    fn profile(&self) -> &Book<N::F> {
        self.profile()
    }
}

impl<N, R, W, S, const B: usize> Solver for Tabular<N, R, W, S, B>
where
    N: Encoding + Send + Sync,
    N::F: 'static,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type R = R;
    type W = W;

    fn batch_size() -> usize {
        B
    }

    fn storage(&mut self) -> &mut Book<N::F> {
        self.profile_mut()
    }

    fn advance(&mut self) {
        self.profile_mut().increment();
    }

    fn apply(&mut self, updates: Batch<Self::F>) {
        self.profile_mut().absorb::<R, W>(updates);
    }
}

impl<N, R, W, S, const B: usize> std::fmt::Display for Tabular<N, R, W, S, B>
where
    N: Encoding + Send + Sync,
    N::F: 'static,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}  Exploitability: {:.4}", self.profile().epochs(), self.exploitability())?;
        writeln!(
            f,
            "{:<24} {:>12}  {:>+10}  {:>10}  {:>10}  {:>10}",
            "Info", "Edge", "Regret", "Weight", "Instant", "Average"
        )?;
        self.profile()
            .encounters()
            .iter()
            .flat_map(|(info, edges)| edges.keys().map(move |edge| (info, edge)))
            .try_for_each(|(info, edge)| {
                writeln!(
                    f,
                    "{:<24} {:>12}  {:>+10.2}  {:>10.2}  {:>10.2}  {:>10.2}",
                    format!("{info:?}"),
                    format!("{edge:?}"),
                    self.profile().cum_regret(info, edge),
                    self.profile().cum_weight(info, edge),
                    self.profile().instant_policy(info, edge),
                    self.profile().averaged_policy(info, edge),
                )
            })
    }
}

impl<N, R, W, S, const B: usize> Tabular<N, R, W, S, B>
where
    N: Encoding,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    pub fn new(profile: Book<N::F>, encoder: N) -> Self {
        Self {
            profile,
            encoder,
            schedules: std::marker::PhantomData,
        }
    }

    pub fn encoder(&self) -> &N {
        &self.encoder
    }

    pub fn profile(&self) -> &Book<N::F> {
        &self.profile
    }

    fn profile_mut(&mut self) -> &mut Book<N::F> {
        &mut self.profile
    }
}
