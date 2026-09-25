//!
//! Benchmark of next() and prev()
//!
//! NB: For some reason the permutohedron.next() is almost twice as slow as next()
//! on my computer.
//! I have not dug into the "why," but what you see here is is ALMOST CERTAINLY NOT a fair benchmark.
//!

use criterion::{Criterion, criterion_group, criterion_main};
use heap_unranking::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("heaps-next");
    group.sample_size(2000);
    let make_data = |n| (0..n).collect::<Vec<_>>().into_boxed_slice();

    for n in [
        12, /* Min that doesn't cause .prev() to underflow */
        15, 16, /* Max supported by permutohedron */
    ] {
        group.bench_with_input(format!("ours.prev(n={n})"), &n, |b, &n| {
            // Seems to be just a tiny bit faster than next(), interestingly.
            let mut ours_prev = HeapsAlgorithm::at_k::<Box<[u8]>, usize>(
                make_data(n),
                (1..=n as usize).product::<usize>() - 1, // start at the last
            );
            b.iter(|| std::hint::black_box(ours_prev.previous().unwrap().clone()))
        });
        group.bench_with_input(format!("ours.next(n={n})"), &n, |b, &n| {
            let x = &make_data(n)[..];
            let mut ours = HeapsAlgorithm::new::<&[u8]>(x);
            b.iter(|| std::hint::black_box(ours.next().unwrap()));
        });

        group.bench_with_input(format!("permutohedron.next(n={n})"), &n, |b, &n| {
            let mut data = make_data(n);
            let mut other = permutohedron::Heap::new(&mut data);
            b.iter(|| std::hint::black_box(other.next().unwrap()))
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
