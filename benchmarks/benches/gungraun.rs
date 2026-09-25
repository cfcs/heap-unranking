use gungraun::prelude::*;
use gungraun::{Cachegrind, ValgrindTool, client_requests};
use std::hint::black_box;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

struct Case {
    qs: Vec<usize>,
    identity: Vec<u32>,
}

fn make_cases(length: usize, num_cases: usize) -> Vec<Case> {
    let mut rng = StdRng::seed_from_u64(0x1);
    let mut cases: Vec<Case> = Vec::new();

    for _case in 0..num_cases {
        let identity = (0..(length + 1).try_into().unwrap()).collect();
        let mut qs = vec![0; length];

        for (offset, q) in qs.iter_mut().enumerate() {
            *q = rng.random_range(0..=offset);
        }

        cases.push(Case { qs, identity });
    }

    cases
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .default_tool(ValgrindTool::Cachegrind)
        .tool(Cachegrind::with_args(["--instr-at-start=no"])),
    setup = make_cases
)]
#[bench::with_setup_5k(args = [5_000, 1])]
#[bench::with_setup_25k(args = [25_000, 1])]
#[bench::with_setup_100k(args = [100_000, 1])]
fn bench_forward_by_qs(cases: Vec<Case>) {
    let mut even_tmp = Vec::with_capacity(cases[0].identity.len() + 1);
    for case in cases {
        let permutation = case.identity.clone();
        client_requests::cachegrind::start_instrumentation();
        black_box(heap_unranking::treapheaps::forward_by_qs(
            black_box(&case.qs),
            &mut even_tmp,
            &mut black_box(permutation)[..],
        ));
        client_requests::cachegrind::stop_instrumentation();
    }
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default()
        .default_tool(ValgrindTool::Cachegrind)
        .tool(Cachegrind::with_args(["--instr-at-start=no"])),
    setup = make_cases
)]
#[bench::with_setup_5k(args = [5_000, 1])]
#[bench::with_setup_25k(args = [25_000, 1])]
#[bench::with_setup_100k(args = [100_000, 1])]
fn bench_reference_forward_by_qs(cases: Vec<Case>) {
    let mut even_tmp = Vec::with_capacity(cases[0].identity.len() + 1);
    for case in cases {
        let permutation = case.identity.clone();
        client_requests::cachegrind::start_instrumentation();
        black_box(heap_unranking::treapheaps::reference_forward_by_qs(
            black_box(&case.qs),
            &mut even_tmp,
            &mut black_box(permutation)[..],
        ));
        client_requests::cachegrind::stop_instrumentation();
    }
}

library_benchmark_group!(
    name = forward_by_qs,
    compare_by_id = true,
    benchmarks = [bench_forward_by_qs, bench_reference_forward_by_qs]
);

main!(library_benchmark_groups = forward_by_qs);
