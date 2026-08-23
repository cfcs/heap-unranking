#![no_main]

use heap_unranking::*;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    if data[0] == 0 {
        return;
    }

    let n = data[0] as usize;

    let mut fact_i = BigUint::ONE;
    for i in 1..n {
        // https://mathoverflow.net/a/484115
        fact_i *= BigUint::from(i + 1);
    }

    if fact_i.bits() * 8 + 1 < data.len() as u64 {
        return;
    }

    let input_rank = BigUint::from_bytes_le(&data[1..]);

    if input_rank >= fact_i {
        return; // reject if input_rank isn't in range
    }

    let permutation = unrank_bigint(n, input_rank.clone());
    let recovered_rank = rank_bigint(&permutation[..]);
    assert_eq!(input_rank, recovered_rank);

    let permutation2 = unrank_bigint(n, recovered_rank);

    assert_eq!(permutation, permutation2);
});
