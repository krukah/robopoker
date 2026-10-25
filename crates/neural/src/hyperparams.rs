//! Hyperparameters for Deep CFR traversal, memory, and network fitting.

/// Every Deep CFR knob: traversals `K` per epoch, reservoir capacity, network shape, and how each
/// fit runs. The defaults suit Kuhn-sized games, not the paper's poker benchmarks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeepHyperParams {
    traversals: usize,
    capacity: usize,
    width: usize,
    depth: usize,
    steps: usize,
    batch: usize,
    rate: f64,
    warm: bool,
}

impl DeepHyperParams {
    /// Zero `traversals` is normalized to one so every epoch observes something.
    pub fn new(
        traversals: usize,
        capacity: usize,
        width: usize,
        depth: usize,
        steps: usize,
        batch: usize,
        rate: f64,
    ) -> Self {
        Self {
            traversals: traversals.max(1),
            capacity,
            width,
            depth,
            steps,
            batch,
            rate,
            warm: false,
        }
    }

    /// Each refit continues from the last net's weights instead of fresh ones, unlike the paper.
    pub fn with_warm(self, warm: bool) -> Self {
        Self { warm, ..self }
    }

    pub fn with_capacity(self, capacity: usize) -> Self {
        Self { capacity, ..self }
    }

    pub fn with_network(self, width: usize, depth: usize) -> Self {
        Self { width, depth, ..self }
    }

    pub fn with_fitting(self, steps: usize, batch: usize, rate: f64) -> Self {
        Self {
            steps,
            batch,
            rate,
            ..self
        }
    }

    /// External-sampling traversals `K` per epoch, all against the same frozen networks.
    pub fn traversals(&self) -> usize {
        self.traversals
    }

    /// Rows each reservoir keeps: every `M_V,p`, and `M_Π`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Units in each hidden layer.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Hidden layers between the features and the outputs.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Optimizer steps per fit; every fit starts from fresh parameters.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// Rows per minibatch.
    pub fn batch(&self) -> usize {
        self.batch
    }

    /// AdamW learning rate.
    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn warm(&self) -> bool {
        self.warm
    }
}

impl Default for DeepHyperParams {
    fn default() -> Self {
        Self::new(1 << 5, 1 << 16, 64, 2, 300, 512, 3e-3)
    }
}

pokerkit::hyperparams!(DeepHyperParams);
