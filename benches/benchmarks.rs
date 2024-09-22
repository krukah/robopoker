use criterion::measurement::WallTime;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use robopoker::cards::deck::Deck;
use robopoker::cards::evaluator::Evaluator;
use robopoker::cards::hand::Hand;
use robopoker::cards::observation::NodeObservation;
use robopoker::cards::street::Street;
use robopoker::cards::strength::Strength;

fn custom_criterion() -> Criterion<WallTime> {
    Criterion::default()
        .without_plots()
        .noise_threshold(0.5)
        .significance_level(0.01)
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(1))
}

fn benchmark_exhaustive_flops(c: &mut Criterion) {
    let mut group = c.benchmark_group("Exhaustive Flops");
    group.throughput(Throughput::Elements(1)); // If you're enumerating one flop at a time
    group.bench_function(BenchmarkId::new("flop enumeration", "flop"), |b| {
        b.iter(|| NodeObservation::all(Street::Flop))
    });
    group.finish();
}

fn benchmark_exhaustive_equity_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Equity Calculation");
    group.throughput(Throughput::Elements(1)); // One equity calculation per iteration
    group.bench_function(BenchmarkId::new("equity calculation", "showdown"), |b| {
        b.iter(|| {
            let mut deck = Deck::new();
            let secret = Hand::from((0..2).map(|_| deck.draw()).collect::<Vec<_>>());
            let public = Hand::from((0..5).map(|_| deck.draw()).collect::<Vec<_>>());
            let observation = NodeObservation::from((secret, public));
            observation.equity()
        })
    });
    group.finish();
}

fn benchmark_evaluator_7_card(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hand Evaluation");
    group.throughput(Throughput::Elements(1)); // One hand evaluation per iteration
    group.bench_function(BenchmarkId::new("hand evaluation", "7 cards"), |b| {
        b.iter(|| {
            let mut deck = Deck::new();
            let hand = Hand::from((0..7).map(|_| deck.draw()).collect::<Vec<_>>());
            let evaluator = Evaluator::from(hand);
            Strength::from(evaluator)
        })
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = benchmark_exhaustive_equity_calculation, benchmark_exhaustive_flops, benchmark_evaluator_7_card
}
criterion_main!(benches);
