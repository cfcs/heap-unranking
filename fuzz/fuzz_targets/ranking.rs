#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

use heap_unranking::precompute::{precompute, rank, unrank};
use heap_unranking::*;

#[derive(Debug, Arbitrary)]
pub struct NK {
    pub n: u8,
    pub k: usize,
}

static PREFIXES: OnceLock<Vec<Box<[u8]>>> = OnceLock::new();

fuzz_target!(|args: NK| {
    let n: usize = args.n as usize;
    if n == 0 {
        return;
    }

    // ensure k < n!
    match (1..=args.n as usize).try_fold(1usize, |acc, i| acc.checked_mul(i)) {
        Some(factor) if factor > args.k => {}
        _ => {
            return;
        }
    }
    let permutation = unrank_noprecomp(n, args.k);
    let recovered_k = rank_noprecomp(&permutation);
    assert_eq!(args.k, recovered_k);
    let prefixes = PREFIXES.get_or_init(|| precompute(20));
    let r_k = rank(&prefixes, permutation.clone());
    assert_eq!(args.k, r_k);
    let ur = unrank(&prefixes, n, args.k);
    assert_eq!(permutation, ur);
    /* this is something like 8 times slower than all of the above:
    let r_ur : Box<[u8]> = unrank_recursive(n, args.k).into();
    assert_eq!(r_ur, ur);
    */
});
