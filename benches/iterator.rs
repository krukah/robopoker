use std::time::Duration;

use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use robopoker::cards::hand::Hand;
use robopoker::cards::hand::HandIterator;
use robopoker::cards::hand::SandIterator;

const MASK: u64 = 0b000000000000000001100001100110000u64;
const SIZE: usize = 2;

fn bench_jump(c: &mut Criterion) {
    let mut group = c.benchmark_group("Jump Iterator");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    group.bench_function("jump", |b| {
        b.iter(|| {
            let mut iter = HandIterator::from((SIZE, Hand::from(MASK)));
            for _ in 0..iter.combinations() {
                black_box(iter.next());
            }
        })
    });
    group.finish();
}

fn bench_skip(c: &mut Criterion) {
    let mut group = c.benchmark_group("Skip Iterator");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    group.bench_function("skip", |b| {
        b.iter(|| {
            let mut iter = SandIterator::from((SIZE, Hand::from(MASK)));
            for _ in 0..iter.combinations() {
                black_box(iter.next());
            }
        })
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(120))
        .warm_up_time(Duration::from_secs(1));
    targets = bench_jump, bench_skip
);
criterion_main!(benches);
