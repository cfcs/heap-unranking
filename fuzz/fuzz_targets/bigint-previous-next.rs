//!
//! Tests next()/previous() for large n and large ranks
//!

#![no_main]

use heap_unranking::*;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;

#[derive(Debug, Arbitrary)]
pub struct NK {
    pub data: Box<[u8]>,
    pub n: u8,
    pub steps: u16,
    // pub x: BigUint, // feature=quickcheck
}

fuzz_target!(|args: NK| {
    if args.data.len() == 0 {
        return;
    }
    if args.n == 0 {
        return;
    }

    if args.steps > 722 {
        return; // we don't want *too* many steps since that is slow
    }

    let n = args.n as usize;

    let mut fact_i = BigUint::ONE;
    for i in 1..n {
        // can we compute this faster? https://mathoverflow.net/a/484115
        fact_i *= BigUint::from(i + 1);
    }

    let input_rank = BigUint::from_bytes_le(&args.data[..]);
    let fwd_rank = BigUint::from(args.steps as usize + 1_usize) + &input_rank;

    if fwd_rank >= fact_i {
        return; // reject if input_rank isn't in range
    }
    let mut heaps = HeapsAlgorithm::at_k(0..n, input_rank.clone());
    let fst = heaps.next();
    for _ in 0..args.steps {
        heaps.step(|_| true);
    }
    let fwd = heaps.next().unwrap();

    let recovered_rank = rank_bigint(&fwd[..]);

    assert_eq!(fwd_rank, recovered_rank);

    for _ in 0..args.steps {
        heaps.previous();
    }

    let back = heaps.previous();
    assert_eq!(back, fst.as_ref());
});
