criterion::criterion_main!(benches);
criterion::criterion_group! {
    name = benches;
    config = criterion::Criterion::default()
        .without_plots()
        .noise_threshold(3.0)
        .significance_level(0.01)
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(1));
    targets =
        sampling_river_evaluation,
        sampling_river_equity,
        sampling_river_observation,
        converting_turn_isomorphism,
        exhausting_flop_observations,
        exhausting_flop_isomorphisms,
        collecting_turn_histogram,
        computing_optimal_transport_variation,
        computing_optimal_transport_heuristic,
        computing_optimal_transport_sinkhorns,
        solving_cfr_rps,
        cfr_policy_vector_rps,
        cfr_sampling_probs_rps,
}

fn sampling_river_evaluation(c: &mut criterion::Criterion) {
    c.bench_function("evaluate a 7-card Hand", |b| {
        let hand = Hand::from(Observation::from(Street::Rive));
        b.iter(|| Strength::from(Evaluator::from(hand)))
    });
}

fn sampling_river_observation(c: &mut criterion::Criterion) {
    c.bench_function("collect a 7-card River Observation", |b| {
        b.iter(|| Observation::from(Street::Rive))
    });
}

fn sampling_river_equity(c: &mut criterion::Criterion) {
    let observation = Observation::from(Street::Rive);
    c.bench_function("calculate River equity", |b| {
        b.iter(|| observation.equity())
    });
}

fn exhausting_flop_observations(c: &mut criterion::Criterion) {
    c.bench_function("exhaust all Flop Observations", |b| {
        b.iter(|| ObservationIterator::from(Street::Flop).count())
    });
}

fn exhausting_flop_isomorphisms(c: &mut criterion::Criterion) {
    c.bench_function("exhaust all Flop Isomorphisms", |b| {
        b.iter(|| {
            ObservationIterator::from(Street::Flop)
                .filter(Isomorphism::is_canonical)
                .count()
        })
    });
}

fn converting_turn_isomorphism(c: &mut criterion::Criterion) {
    let observation = Observation::from(Street::Turn);
    c.bench_function("convert a Turn Observation to Isomorphism", |b| {
        b.iter(|| Isomorphism::from(observation))
    });
}

fn collecting_turn_histogram(c: &mut criterion::Criterion) {
    let observation = Observation::from(Street::Turn);
    c.bench_function("collect a Histogram from a Turn Observation", |b| {
        b.iter(|| Histogram::from(observation))
    });
}

fn computing_optimal_transport_variation(c: &mut criterion::Criterion) {
    let ref h1 = Histogram::from(Observation::from(Street::Turn));
    let ref h2 = Histogram::from(Observation::from(Street::Turn));
    c.bench_function("compute optimal transport (1-dimensional)", |b| {
        b.iter(|| Equity::variation(&h1, &h2))
    });
}

fn computing_optimal_transport_heuristic(c: &mut criterion::Criterion) {
    let (metric, h1, h2, _) = EMD::random().inner();
    c.bench_function("compute optimal transport (greedy)", |b| {
        b.iter(|| Heuristic::from((&h1, &h2, &metric)).minimize().cost())
    });
}

fn computing_optimal_transport_sinkhorns(c: &mut criterion::Criterion) {
    let (metric, h1, h2, _) = EMD::random().inner();
    c.bench_function("compute optimal transport (entropy regularized)", |b| {
        b.iter(|| Sinkhorn::from((&h1, &h2, &metric)).minimize().cost())
    });
    /*
    TEMPERATURE   ITERS  TOLERANCE  TIME
        0.125       16     0.001     200
        0.125       16     0.010     135
        0.125       16     0.100     67
        8.000       16     0.001     55
        8.000       16     0.010     55
        8.000       16     0.100     55
     */
}

fn solving_cfr_rps(c: &mut criterion::Criterion) {
    c.bench_function("cfr solve rock paper scissors (rps)", |b| {
        b.iter(|| RPS::default().solve());
    });
}

fn cfr_policy_vector_rps(c: &mut criterion::Criterion) {
    use robopoker::mccfr::rps::edge::Edge;
    use robopoker::mccfr::rps::game::Game;
    use robopoker::mccfr::rps::turn::Turn;
    use robopoker::mccfr::structs::infoset::InfoSet;
    use robopoker::mccfr::structs::tree::Tree;
    use robopoker::mccfr::traits::game::Game as GameTrait;
    use robopoker::mccfr::traits::profile::Profile;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Default)]
    struct BenchProfile {
        regrets: HashMap<Turn, HashMap<Edge, f32>>, // accumulated regrets
        epochs: usize,
    }
    impl Profile for BenchProfile {
        type T = Turn;
        type E = Edge;
        type G = Game;
        type I = Turn;
        fn increment(&mut self) { self.epochs += 1; }
        fn walker(&self) -> Self::T { Turn::P1 }
        fn epochs(&self) -> usize { self.epochs }
        fn sum_policy(&self, _info: &Self::I, _edge: &Self::E) -> f32 { 0.0 }
        fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> f32 {
            self.regrets.get(info).and_then(|m| m.get(edge)).copied().unwrap_or_default()
        }
    }

    // Build a minimal infoset for Turn::P1
    let mut tree: Tree<Turn, Edge, Game, Turn> = Tree::default();
    let head_index = { let node = tree.seed(Turn::P1, <Game as GameTrait>::root()); node.index() };
    let mut infoset = InfoSet::from(Arc::new(tree));
    infoset.push(head_index);

    let mut profile = BenchProfile::default();
    profile.regrets.entry(Turn::P1).or_default().extend([
        (Edge::R, 1.0), (Edge::P, 3.0), (Edge::S, 0.5)
    ]);

    c.bench_function("cfr policy_vector (RPS)", |b| {
        b.iter(|| robopoker::mccfr::traits::profile::Profile::policy_vector(&profile, &infoset))
    });
}

fn cfr_sampling_probs_rps(c: &mut criterion::Criterion) {
    use robopoker::mccfr::rps::edge::Edge;
    use robopoker::mccfr::rps::turn::Turn;
    use robopoker::mccfr::traits::profile::Profile;
    use std::collections::HashMap;

    #[derive(Default)]
    struct BenchProfile {
        policies: HashMap<Turn, HashMap<Edge, f32>>, // accumulated policy weights
        epochs: usize,
    }
    impl Profile for BenchProfile {
        type T = Turn;
        type E = Edge;
        type G = robopoker::mccfr::rps::game::Game;
        type I = Turn;
        fn increment(&mut self) { self.epochs += 1; }
        fn walker(&self) -> Self::T { Turn::P1 }
        fn epochs(&self) -> usize { self.epochs }
        fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> f32 {
            self.policies.get(info).and_then(|m| m.get(edge)).copied().unwrap_or_default()
        }
        fn sum_regret(&self, _info: &Self::I, _edge: &Self::E) -> f32 { 0.0 }
    }

    let mut profile = BenchProfile::default();
    profile.policies.entry(Turn::P1).or_default().extend([
        (Edge::R, 0.10), (Edge::P, 0.30), (Edge::S, 0.60)
    ]);

    c.bench_function("cfr sampling probability q(a) (RPS)", |b| {
        b.iter(|| {
            let _ = (
                Profile::sample(&profile, &Turn::P1, &Edge::R),
                Profile::sample(&profile, &Turn::P1, &Edge::P),
                Profile::sample(&profile, &Turn::P1, &Edge::S),
            );
        })
    });
}

use robopoker::cards::evaluator::Evaluator;
use robopoker::cards::hand::Hand;
use robopoker::cards::isomorphism::Isomorphism;
use robopoker::cards::observation::Observation;
use robopoker::cards::observations::ObservationIterator;
use robopoker::cards::street::Street;
use robopoker::cards::strength::Strength;
use robopoker::clustering::emd::EMD;
use robopoker::clustering::equity::Equity;
use robopoker::clustering::heuristic::Heuristic;
use robopoker::clustering::histogram::Histogram;
use robopoker::clustering::sinkhorn::Sinkhorn;
use robopoker::mccfr::rps::RPS;
use robopoker::mccfr::traits::Blueprint;
use robopoker::transport::coupling::Coupling;
use robopoker::Arbitrary;
