use criterion::{Criterion, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;

use heap_unranking::treapheaps::{forward_by_qs, reference_forward_by_qs};

struct Case {
    qs: Vec<usize>,
    identity: Vec<usize>,
}

fn make_cases(length: usize, num_cases: usize) -> Vec<Case> {
    let mut rng = StdRng::seed_from_u64(0x1);
    let mut cases = Vec::new();

    for _case in 0..num_cases {
        let identity: Vec<usize> = (0..length + 1).collect();
        let mut qs = vec![0; length];

        for (offset, q) in qs.iter_mut().enumerate() {
            *q = rng.random_range(0..=offset);
        }

        cases.push(Case { qs, identity });
    }

    cases
}

fn bench_forward_by_qs(c: &mut Criterion) {
    let mut group = c.benchmark_group("forward_by_qs");
    group.sample_size(10); // Minimum allowed
    group.measurement_time(std::time::Duration::from_millis(3_000));
    group.warm_up_time(std::time::Duration::from_millis(1_000));

    for length in [1000, 10000, 22500, 32768, 65536, 2 * 65536].iter() {
        let cases = make_cases(*length, 1);

        group.bench_with_input(format!("just_the_clone_n{}", length), length, |b, _| {
            b.iter(|| {
                let mut even_tmp: Vec<usize> = Vec::with_capacity(length + 1);
                let mut actuals: Vec<Vec<usize>> =
                    cases.iter().map(|case| case.identity.clone()).collect();

                for (i, case) in cases.iter().enumerate() {
                    black_box(&case.qs);
                    black_box(&mut actuals[i]);
                }
                black_box(&mut even_tmp);
            });
        });

        group.bench_with_input(format!("reference_n{}", length), length, |b, _| {
            b.iter(|| {
                let mut even_tmp = Vec::with_capacity(length + 1);
                let mut actuals: Vec<Vec<usize>> =
                    cases.iter().map(|case| case.identity.clone()).collect();

                for (i, case) in cases.iter().enumerate() {
                    reference_forward_by_qs(black_box(&case.qs), &mut even_tmp, &mut actuals[i]);
                    black_box(&mut actuals[i]);
                }
            });
        });

        group.bench_with_input(format!("treap_n{}", length), length, |b, _| {
            b.iter(|| {
                let mut even_tmp = Vec::with_capacity(length + 1);
                let mut actuals: Vec<Vec<usize>> =
                    cases.iter().map(|case| case.identity.clone()).collect();

                for (i, case) in cases.iter().enumerate() {
                    forward_by_qs(black_box(&case.qs), &mut even_tmp, &mut actuals[i]);
                    black_box(&mut actuals[i]);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_forward_by_qs);
criterion_main!(benches);
