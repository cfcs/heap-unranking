// Throughput benchmark for rank()/unrank(), run with:
//   cargo run --release --example bench
//
// The interesting workload is job splitting: many unrank() calls at scattered k,
// so the per-call cost at a fixed n is what matters, not the one-off precompute().

use heap_unranking::*;
use std::hint::black_box;
use std::time::Instant;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
}

// Warm up, then report the best of several reps. A single unwarmed run overstated
// these by up to 2x at small n, which is enough to draw the wrong conclusion.
fn bench<T>(label: &str, iters: usize, mut f: impl FnMut() -> T) {
    for _ in 0..iters / 3 {
        black_box(f());
    }
    let mut best = f64::MAX;
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..iters {
            black_box(f());
        }
        best = best.min(start.elapsed().as_nanos() as f64 / iters as f64);
    }
    println!("{label:28} {best:>9.1} ns/op   {:>8.2} Mops/s", 1e3 / best);
}

fn main() {
    let s = precompute(20);

    bench("precompute(20)", 100_000, || precompute(20));

    for n in [8usize, 12, 16, 20] {
        let mut rng = Lcg(0x243f6a8885a308d3);
        bench(&format!("unrank(n={n})"), 1_000_000, || {
            unrank(&s, n, rng.next() as usize)
        });
    }

    for n in [8usize, 12, 16, 20] {
        let mut rng = Lcg(0x243f6a8885a308d3);
        let mut out = vec![0u8; n];
        bench(&format!("unrank_into(n={n})"), 1_000_000, || {
            unrank_into(&s, rng.next() as usize, &mut out)
        });
    }

    for n in [8usize, 12, 16, 20] {
        let mut rng = Lcg(0x243f6a8885a308d3);
        let perms: Vec<Box<[u8]>> = (0..1024)
            .map(|_| unrank(&s, n, rng.next() as usize))
            .collect();
        let mut i = 0;
        bench(&format!("rank(n={n})"), 1_000_000, || {
            i = (i + 1) % perms.len();
            rank(&s, &perms[i])
        });
    }
}
