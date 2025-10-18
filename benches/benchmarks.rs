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
        clustering_kmeans_elkan,
        clustering_kmeans_naive,
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

fn clustering_kmeans_elkan(c: &mut criterion::Criterion) {
    c.bench_function("equity k-means clustering (Elkan optimization)", |b| {
        b.iter(|| {
            let mut km = TurnLayer::new();
            for _ in 0..km.t() {
                km.step_elkan();
            }
        })
    });
}

fn clustering_kmeans_naive(c: &mut criterion::Criterion) {
    c.bench_function("equity k-means clustering (naive implementation)", |b| {
        b.iter(|| {
            let mut km = TurnLayer::new();
            for _ in 0..km.t() {
                km.step_naive();
            }
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
use robopoker::clustering::elkan::Elkan;
use robopoker::clustering::emd::EMD;
use robopoker::clustering::equity::Equity;
use robopoker::clustering::heuristic::Heuristic;
use robopoker::clustering::histogram::Histogram;
use robopoker::clustering::sinkhorn::Sinkhorn;
use robopoker::clustering::turns::TurnLayer;
use robopoker::mccfr::rps::RPS;
use robopoker::mccfr::traits::Blueprint;
use robopoker::transport::coupling::Coupling;
use robopoker::Arbitrary;
