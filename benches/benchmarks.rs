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
        sampling_turn_isomorphism,
        exhausting_flop_observations,
        exhausting_flop_isomorphisms,
        sampling_turn_histogram,
        sampling_turn_histogram_emd,
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
    c.bench_function("calculate River equity", |b| {
        let observation = Observation::from(Street::Rive);
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
                .filter(|o| Equivalence::is_canonical(o))
                .count()
        })
    });
}

fn sampling_turn_isomorphism(c: &mut criterion::Criterion) {
    c.bench_function("compute Isomorphism from a Turn Observation", |b| {
        let observation = Observation::from(Street::Turn);
        b.iter(|| Equivalence::from(&observation))
    });
}

fn sampling_turn_histogram(c: &mut criterion::Criterion) {
    c.bench_function("collect a Histogram from a Turn Observation", |b| {
        let observation = Observation::from(Street::Turn);
        b.iter(|| Histogram::from(observation))
    });
}

fn sampling_turn_histogram_emd(c: &mut criterion::Criterion) {
    c.bench_function("calculate EMD between two Turn Histograms", |b| {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        b.iter(|| metric.emd(h1, h2))
    });
}

use robopoker::cards::evaluator::Evaluator;
use robopoker::cards::hand::Hand;
use robopoker::cards::isomorphism::Equivalence;
use robopoker::cards::observation::Observation;
use robopoker::cards::observations::ObservationIterator;
use robopoker::cards::street::Street;
use robopoker::cards::strength::Strength;
use robopoker::clustering::histogram::Histogram;
use robopoker::clustering::metric::Metric;
