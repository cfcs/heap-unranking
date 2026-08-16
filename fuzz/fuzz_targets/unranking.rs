#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

use heap_unranking::*;

#[derive(Debug, Arbitrary)]
pub struct NK {
    pub n: u8,
    pub k: usize,
}

static PREFIXES: OnceLock<Vec<Box<[u8]>>> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let n: usize = data.len();
    if n == 0 {
        return;
    }

    if n >= 21 {
        return; // reject the n at which factorial(n) overflows
    }

    let mut bits: u32 = 0;
    // ensure the [u8] contains indices:
    for &x in data.iter() {
        let idx = x as usize;
        if idx >= n {
            return; // too big
        }
        let bit = 1u32 << idx;
        if bits & bit != 0 {
            return; // seen before
        }
        bits |= bit;
    }

    let recovered_k = rank_noprecomp(&data);
    let permutation = unrank_noprecomp(n, recovered_k);
    assert_eq!(data, &permutation[..]);
    let prefixes = PREFIXES.get_or_init(|| precompute(21));
    let r_k = rank(&prefixes, permutation);
    assert_eq!(recovered_k, r_k);

    /*
    assert_eq!(permutation, permutation2);
    let ur = unrank(&prefixes, n, args.k);
    assert_eq!(permutation2, ur);
    */
    /* this is something like 8 times slower than all of the above:
    let r_ur : Box<[u8]> = unrank_recursive(n, args.k).into();
    assert_eq!(r_ur, ur);
    */
});
