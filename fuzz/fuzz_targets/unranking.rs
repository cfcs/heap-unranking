#![no_main]

use heap_unranking::precompute::{precompute, rank, unrank};
use heap_unranking::*;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;
use std::sync::OnceLock;

static PREFIXES: OnceLock<Vec<Box<[u8]>>> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let n: usize = data.len();
    if n == 0 {
        return;
    }
    if n > 128 {
        return;
    }

    let mut bits: u128 = 0;
    // ensure the [u8] contains indices:
    for &x in data.iter() {
        let idx = x as usize;
        if idx >= n {
            return; // too big
        }
        let bit = 1 << idx;
        if bits & bit != 0 {
            return; // seen before
        }
        bits |= bit;
    }

    let usize_data: Box<[usize]> = data.iter().map(|&e| e as usize).collect();
    let bigint_recovered_k = rank_bigint(&usize_data);
    let permutation = unrank_bigint(n, bigint_recovered_k.clone());
    assert_eq!(&usize_data[..], &permutation[..]);

    if n < 21 {
        // factorial(n) doesn't overflow for n < 21 for 64bit usize
        let recovered_k = rank_noprecomp(&data);
        let permutation = unrank_noprecomp(n, recovered_k);
        assert_eq!(data, &permutation[..]);
        let prefixes = PREFIXES.get_or_init(|| precompute(21));
        let ur = unrank(&prefixes, n, recovered_k);
        assert_eq!(permutation, ur);
        let r_k = rank(&prefixes, permutation);
        assert_eq!(recovered_k, r_k);
        assert_eq!(&BigUint::from(r_k), &bigint_recovered_k);

        /* this is something like 8 times slower than all of the above:
        let r_ur : Box<[u8]> = unrank_recursive(n, args.k).into();
        assert_eq!(r_ur, ur);
         */
    }
});
