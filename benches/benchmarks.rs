criterion::criterion_main!(benches);
criterion::criterion_group! {
    name = benches;
    config = criterion::Criterion::default()
        .without_plots()
        .noise_threshold(3.0)
        .significance_level(0.01)
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(1));
    targets =
        evaluating_7,
        enumerating_flops,
        calculating_equity,
        sampling_observation,
        computing_isomorphism,
        enumerating_flops_isomorphic,
        constructing_equity_histogram,
        differencing_equity_histograms,
}

fn sampling_observation(c: &mut criterion::Criterion) {
    c.bench_function("sample Observation from River", |b| {
        b.iter(|| Observation::from(Street::Rive))
    });
}

fn enumerating_flops(c: &mut criterion::Criterion) {
    c.bench_function("enumerate all Flops", |b| {
        b.iter(|| Observation::exhaust(Street::Flop))
    });
}

fn enumerating_flops_isomorphic(c: &mut criterion::Criterion) {
    c.bench_function("enumerate all Flops up to isomorphism", |b| {
        b.iter(|| Isomorphism::exhaust(Street::Flop))
    });
}

fn calculating_equity(c: &mut criterion::Criterion) {
    c.bench_function("calculate River equity", |b| {
        let observation = Observation::from(Street::Rive);
        b.iter(|| observation.equity())
    });
}

fn evaluating_7(c: &mut criterion::Criterion) {
    c.bench_function("evaluate a 7-card Hand", |b| {
        let hand = Hand::from(Observation::from(Street::Rive));
        b.iter(|| Strength::from(Evaluator::from(hand)))
    });
}

fn computing_isomorphism(c: &mut criterion::Criterion) {
    c.bench_function("compute Isomorphism from a Turn Observation", |b| {
        let observation = Observation::from(Street::Turn);
        b.iter(|| Isomorphism::from(observation))
    });
}

fn constructing_equity_histogram(c: &mut criterion::Criterion) {
    c.bench_function("collect a Histogram from a Turn Observation", |b| {
        let observation = Observation::from(Street::Turn);
        b.iter(|| Histogram::from(observation))
    });
}

fn differencing_equity_histograms(c: &mut criterion::Criterion) {
    c.bench_function("calculate EMD between two Turn Histograms", |b| {
        let metric = Metric::default();
        let ref h1 = Histogram::from(Observation::from(Street::Turn));
        let ref h2 = Histogram::from(Observation::from(Street::Turn));
        b.iter(|| metric.emd(h1, h2))
    });
}

use robopoker::cards::evaluator::Evaluator;
use robopoker::cards::hand::Hand;
use robopoker::cards::isomorphism::Isomorphism;
use robopoker::cards::observation::Observation;
use robopoker::cards::street::Street;
use robopoker::cards::strength::Strength;
use robopoker::clustering::histogram::Histogram;
use robopoker::clustering::metric::Metric;
