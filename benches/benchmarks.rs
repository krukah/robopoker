criterion_group! {
    name = benches;
    config = Criterion::default()
        .without_plots()
        .noise_threshold(3.0)
        .significance_level(0.001)
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(1));
    targets =
        enumerating_flops,
        calculating_equity,
        evaluating_at_flop,
        evaluating_at_river,
        building_equity_histogram,
        calculating_histogram_emd,
}

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use robopoker::cards::evaluator::Evaluator;
use robopoker::cards::hand::Hand;
use robopoker::cards::observation::Observation;
use robopoker::cards::street::Street;
use robopoker::cards::strength::Strength;
use robopoker::clustering::histogram::Histogram;
use robopoker::clustering::metric::Metric;
use std::collections::BTreeMap;

fn enumerating_flops(c: &mut Criterion) {
    let mut group = c.benchmark_group("Exhaustive Flops");
    group.bench_function(BenchmarkId::new("flop enumeration", "flop"), |b| {
        b.iter(|| Observation::all(Street::Flop))
    });
    group.finish();
}

fn calculating_equity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Equity Calculation");
    group.bench_function(BenchmarkId::new("equity calculation", "showdown"), |b| {
        b.iter(|| Observation::from(Street::Rive).equity())
    });
    group.finish();
}

fn evaluating_at_river(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hand Evaluation");
    group.bench_function(BenchmarkId::new("hand evaluation", "7 cards"), |b| {
        b.iter(|| Strength::from(Evaluator::from(Hand::from(Observation::from(Street::Rive)))))
    });
    group.finish();
}

fn evaluating_at_flop(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hand Evaluation");
    group.bench_function(BenchmarkId::new("hand evaluation", "5 cards"), |b| {
        b.iter(|| Strength::from(Evaluator::from(Hand::from(Observation::from(Street::Flop)))))
    });
    group.finish();
}

fn building_equity_histogram(c: &mut Criterion) {
    let mut group = c.benchmark_group("Histogram from Observation");
    group.bench_function(BenchmarkId::new("histogram creation", "turn"), |b| {
        b.iter(|| Histogram::from(Observation::from(Street::Turn)))
    });
    group.finish();
}

fn calculating_histogram_emd(c: &mut Criterion) {
    let mut group = c.benchmark_group("Histogram EMD Calculation");
    group.bench_function(BenchmarkId::new("EMD calculation", "histogram pair"), |b| {
        b.iter(|| {
            let metric = BTreeMap::default();
            let ref h1 = Histogram::from(Observation::from(Street::Turn));
            let ref h2 = Histogram::from(Observation::from(Street::Turn));
            metric.emd(h1, h2)
        })
    });
    group.finish();
}

criterion_main!(benches);
