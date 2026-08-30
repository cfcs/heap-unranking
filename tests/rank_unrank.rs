#[cfg(test)]
mod unittests {
    use heap_unranking::precompute::*;
    use heap_unranking::*;

    // prng for "random" testing:
    use std::time::{SystemTime, UNIX_EPOCH};
    struct SimpleRng {
        state: usize,
    }
    impl SimpleRng {
        fn new() -> Self {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos() as usize;
            Self { state: seed }
        }

        fn next(&mut self) -> usize {
            let a: usize = 6364136223846793005;
            let c: usize = 1442695040888963407;
            self.state = self.state.wrapping_mul(a).wrapping_add(c);
            self.state
        }

        fn gen_range(&mut self, min: usize, max: usize) -> usize {
            let range = max - min;
            min + (self.next() % range)
        }

        fn fisher_yates_shuffle_u8(&mut self, v: &mut Box<[u8]>) {
            let mut i = v.len();
            while i > 1 {
                let j = self.gen_range(0, i);
                i -= 1;
                v.swap(i, j);
            }
        }
    }

    #[test]
    fn test_rankings() {
        // check that the ranking functions are internally consistent
        let mut rng = SimpleRng::new();
        let precomp = precompute(21);
        for n in 1..21 {
            for _ in 0..=((1..=n).product::<usize>().min(n * n * 1_000)) {
                let mut arr: Box<[u8]> = (0..n as u8).collect();
                rng.fisher_yates_shuffle_u8(&mut arr);
                let k = rank_noprecomp(&arr);
                let sanity_check = unrank_noprecomp(n, k);
                assert_eq!(arr, sanity_check);
                let k2 = rank(&precomp, arr);
                assert_eq!(k, k2);
            }
        }
    }

    #[test]
    fn unrank_rank_random_k() {
        let mut rng = SimpleRng::new();
        let precomp = precompute(21);
        for _ in 0..100000 {
            let n = rng.gen_range(2, 22);
            let mut factorial_n: usize = 1;
            for i in 2..=n {
                match factorial_n.checked_mul(i) {
                    Some(val) => factorial_n = val.saturating_sub(1),
                    None => factorial_n = usize::MAX,
                }
            }
            let k = rng.gen_range(0, factorial_n);
            let perm = unrank(&precomp, n, k);
            let perm2 = unrank_noprecomp(n, k);
            assert_eq!(perm, perm2);
            let recovered2 = rank_noprecomp(&perm);
            assert_eq!(k, recovered2);
            let recovered = rank(&precomp, perm);
            assert_eq!(k, recovered);
        }
    }

    #[test]
    fn test_extremes() {
        // Check we can recover the usize::MAX for n=21 (where 21! overflows it)
        let s = precompute(20);
        let perm = unrank(&s, 21, usize::MAX);
        let k2 = rank_noprecomp(&perm);
        assert_eq!(usize::MAX, k2);
        let k = rank(&s, perm);
        assert_eq!(usize::MAX, k);
    }

    #[test]
    fn test_precompute23() {
        let s = precompute(2);
        let perm = unrank(&s, 3, 5);
        let k2 = rank_noprecomp(&perm);
        assert_eq!(5, k2);
        let k = rank(&s, perm);
        assert_eq!(5, k);
    }

    #[test]
    fn test_precompute34() {
        let s = precompute(3);
        let perm = unrank(&s, 4, 6);
        let k = rank(&s, perm);
        assert_eq!(6, k);
    }

    #[test]
    #[should_panic(expected = "right: 0")]
    fn test_precompute24() {
        // This only precomputes 4-2, but we need at least 4-1 precomputed
        // prefixes:
        let s = precompute(2);
        let perm = unrank(&s, 4, 6);
        let k = rank(&s, perm.clone());
        assert_eq!(6, k);
        let k2 = rank_noprecomp(&perm);
        assert_eq!(6, k2);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn usize_overflow_n_21_overflows() {
        usize_overflow_n_21();
    }

    #[cfg(not(debug_assertions))]
    #[test]
    ///
    /// Silently returns bad result at the moment
    ///
    fn usize_overflow_n_21_wrong() {
        usize_overflow_n_21();
    }

    fn usize_overflow_n_21() {
        let data = [
            6, 10, 12, 9, 14, 17, 0, 7, 16, 4, 19, 3, 5, 2, 8, 1, 13, 20, 11, 15, 18,
        ];
        assert_eq!(21, data.len());
        let k = rank_noprecomp(&data);
        assert_eq!(
            8526381368646914543, k,
            "have not checked if this is the correct k"
        );
        let rec = unrank_noprecomp(data.len(), k);
        assert_ne!(data, &rec[..]);
        let prefixes = precompute(20);
        let k2 = rank(&prefixes, data.into());
        assert_eq!(k, k2, "at least the two implementations agree");
    }

    #[test]
    fn unrank_matches_output_0_10() {
        // check that unrank() matches the traditional Heap's algorithm's outputs
        // (from the permutohedron crate):
        //let mut rng = SimpleRng::new();
        //let mut next_print = rng.gen_range(6_000_000, (u32::MAX as usize) / 50);
        for n in 1..=10usize {
            let mut data: Vec<u8> = (0..(n as u8)).collect();
            let heap = permutohedron::Heap::new(&mut data);

            let s = precompute(if n > 1 { n - 1 } else { 1usize });

            for (k, p) in heap.enumerate() {
                let ur = unrank(&s, p.len(), k);
                /* KAT generation:
                        if k % next_print == 0 {
                            print!(
                                "assert_eq!(unrank(&s, {:?}, {k}), {:?}.into());\n",
                                p.len(),
                                p
                            );
                            next_print = rng.gen_range(6_000_000, (u32::MAX as usize) / 50);
                        }
                */
                assert_eq!(ur, p.into());
            }
        }
    }

    #[test]
    fn test_rank_noprecomp_matches_output_1_11() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 1..=11usize {
            let mut data: Vec<u8> = (0..(n as u8)).collect();
            let heap = permutohedron::Heap::new(&mut data);
            for (k, p) in heap.enumerate() {
                let k2 = rank_noprecomp(&p);
                assert_eq!(k, k2, "rank_noprecomp({:?}) == {k}", p);
                //println!("PASSED: {k2} for n={n}");
            }
        }
    }

    #[test]
    fn test_heaps_algo_4_string() {
        // check that we get the right amount of yielded elements:
        let n = 4;
        let mut last_k = 0;
        for (k, _p) in HeapsAlgorithm::new::<Vec<&str>>(vec!["a", "b", "c", "d"]).enumerate() {
            // println!("{k}: {:?}", p);y
            last_k = k;
        }
        assert_eq!(
            (1..=n).product::<usize>(),
            last_k + 1,
            "they should yield factorial(4) elements"
        );
    }

    #[test]
    fn test_heaps_algo_matches_output_1_11() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 1..=11usize {
            let mut data: Vec<u8> = (0..(n as u8)).collect();
            let heap = permutohedron::Heap::new(&mut data);
            let heap2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
            let mut last_k = 0;
            for ((k, p), p2) in heap.enumerate().zip(heap2) {
                assert_eq!(
                    p.iter().map(|x| *x as usize).collect::<Vec<usize>>(),
                    p2[..],
                    "n={n} k={k}"
                );
                last_k = k;
            }
            assert_eq!(
                (1..=n).product::<usize>() - 1,
                last_k,
                "they should yield factorial(n) elements"
            );
        }
    }

    /*
            /*
                if i == 2 && arr[1] == permutation_i {
                 *qq = 1; // special case for len 3 not covered by the two rules above for [0], [i]
                arr.swap(1, i);
                arr.swap(0, 1);
                continue;
            }
             */

                // It is worth noting that below the only operation we perform that involves
            // the elements of even_tmp / permutation_i is comparing whether or not a given element
            // is equal to permutation_i or not, so I used a bitmap below to amortize the O(n^2)
            // looping over forward_by_q() for the even `i`s, arriving at this
            // [amortized] O(n) solution. First we use the bitmap version of forward_by_q, tracking
            // only the permutation[i], and we use that to find `q`.
            // O(0.5n * 0.5 n * ceil(n/wordsize))
            for it in 1..i {
                // for q in 1..i
                if bmap & (1 << i) != 0 {
                    break;
                } // if permutation_i == tmp[i]
                if bmap & (1 << (i - 1)) != 0
                /* tmp[i - 1] == permutation_i*/
                {
                    /* Essentially, if tmp[i-1] == permutation_i:
                            tmp[i - 2] = tmp[i - 1];
                            tmp[i - 1] = u8::MAX; // unset ; we don't want this to remain permutation_i
                    */
                    bmap |= 1 << (i - 2);
                    bmap &= !(1 << (i - 1)); // if bmap&1 then this step is redundant since we set it below
                    // Since assert_eq!(bmap & (1<<i), 0) doesn't change, we simulate continue; and
                    // proceed:
                    q += 1;
            continue;
                } else {
                    q += 1;
                }
                bmap |= (bmap & 1) << (i - 1); // set [i-1] if bmap[0] is set, can continue if rhs!=0

                // emap is what we call even_tmp in the forward_by_q()
                let mut emap = bmap & 1; // [0]:=tmp[0]

                emap |= (bmap >> (i - 2)) & 2; // [1]:=tmp[i-1]
                emap |= (bmap >> (i - 4)) & 4; // [2]:=tmp[i-2]

                emap |= (bmap & ((1 << (i - 2)) - 1)) << 2; // [3..] = tmp[1..i-2]

                bmap |= (emap & (1 << (i - 1))) << 1; // set [i] if even_tmp[i-1] is set.
                // COULD continue; if rhs!=0

                bmap |= (emap & (1 << i)) >> i; // set bmap[0] if even_tmp[i] == permutation_i
                // COULD continue; if rhs != 0

                let esuffix = (emap >> 2) & ((1 << (i - 3)) - 1);
                bmap |= (((!bmap) >> 1) & esuffix) << 1; // secondcopy_from_slice(even_tmp, ..n)
            }
    */

    #[test]
    fn test_heaps_algo_at_k_output_1_10() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 1..=10usize {
            let mut data: Vec<u8> = (0..(n as u8)).collect();
            let heap = permutohedron::Heap::new(&mut data);
            let mut last_k = 0;
            for (k, p) in (0..).zip(heap) {
                let ur = unrank_noprecomp(n, k);
                let mut heap2 = HeapsAlgorithm::at_k(0..(n as u8), k);
                let p2 = heap2.next().unwrap();
                assert_eq!(p, p2[..], "n={n} k={k}");
                assert_eq!(ur[..], p2[..], "n={n} k={k} (sanity check)");
                last_k = k;
            }
            assert_eq!(
                (1..=n).product::<usize>(),
                last_k + 1,
                "they should yield factorial(n) elements"
            );
        }
    }

    #[test]
    fn test_heaps_algo_previous_4() {
        let n = 3;
        let mut heap2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
        let h0 = heap2.next().unwrap();
        let h1 = heap2.next().unwrap();
        let h2 = heap2.next().unwrap();
        let h3 = heap2.next().unwrap();
        let h4 = heap2.next().unwrap();
        let h5 = heap2.next().unwrap();
        assert_eq!([2, 1, 0], h5[..], "rank 5");
        assert_eq!([1, 2, 0], h4[..], "rank 4");
        assert_eq!([0, 2, 1], h3[..], "rank 3");
        assert_eq!([2, 0, 1], h2[..], "rank 2");
        assert_eq!([1, 0, 2], h1[..], "rank 1");
        assert_eq!([0, 1, 2], h0[..], "rank 0");

        let p4 = heap2.previous().unwrap();
        assert_eq!(h4[..], p4[..], "previous() rank 4");
        let p3 = heap2.previous().unwrap();
        assert_eq!(h3[..], p3[..], "previous() rank 3");
        let p2 = heap2.previous().unwrap();
        assert_eq!(h2[..], p2[..], "previous() rank 2");
        let p1 = heap2.previous().unwrap();
        assert_eq!(h1[..], p1[..], "previous() rank 1");
        let p0 = heap2.previous().unwrap();
        assert_eq!(h0[..], p0[..], "previous() rank 0");
    }

    #[test]
    fn test_heaps_algo_previous_underflow() {
        let n = 4_usize;

        let mut heap1: HeapsAlgorithm<usize> = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
        assert_eq!(None, heap1.previous());
        assert_eq!([0, 1, 2, 3], heap1.next().unwrap()[..]);
        assert_eq!([1, 0, 2, 3], heap1.next().unwrap()[..]);
        assert_eq!([2, 0, 1, 3], heap1.next().unwrap()[..]);
        assert_eq!([1, 0, 2, 3], heap1.previous().unwrap()[..]);

        let mut heap2: HeapsAlgorithm<usize> = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
        assert_eq!([0, 1, 2, 3], heap2.next().unwrap()[..]);
        assert_eq!(None, heap2.previous());
        assert_eq!([0, 1, 2, 3], heap2.next().unwrap()[..]);
        assert_eq!(None, heap2.previous()); // underflow
        // Check that we didn't touch the state:
        assert_eq!([0, 1, 2, 3], heap2.next().unwrap()[..]);
        assert_eq!([1, 0, 2, 3], heap2.next().unwrap()[..]);
        assert_eq!([2, 0, 1, 3], heap2.next().unwrap()[..]);
        assert_eq!([1, 0, 2, 3], heap2.previous().unwrap()[..]);
        assert_eq!([0, 1, 2, 3], heap2.previous().unwrap()[..]);
        assert_eq!(None, heap2.previous());
        assert_eq!(None, heap2.previous());
        assert_eq!([0, 1, 2, 3], heap2.next().unwrap()[..]);
    }

    #[test]
    fn test_heaps_algo_previous_5() {
        let n = 5;
        let mut heap2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
        let h0 = heap2.next().unwrap();
        let h1 = heap2.next().unwrap();
        let h2 = heap2.next().unwrap();
        let h3 = heap2.next().unwrap();
        let h4 = heap2.next().unwrap();
        let h5 = heap2.next().unwrap();
        let h6 = heap2.next().unwrap();
        let h7 = heap2.next().unwrap();
        assert_eq!([1, 3, 0, 2, 4], h7[..], "rank 7");
        assert_eq!([3, 1, 0, 2, 4], h6[..], "rank 6");
        assert_eq!([2, 1, 0, 3, 4], h5[..], "rank 5");
        assert_eq!([1, 2, 0, 3, 4], h4[..], "rank 4");
        assert_eq!([0, 2, 1, 3, 4], h3[..], "rank 3");
        assert_eq!([2, 0, 1, 3, 4], h2[..], "rank 2");
        assert_eq!([1, 0, 2, 3, 4], h1[..], "rank 1");
        assert_eq!([0, 1, 2, 3, 4], h0[..], "rank 0");
        let p6 = heap2.previous().unwrap();
        assert_eq!(h6[..], p6[..], "previous() rank 6");
        let p5 = heap2.previous().unwrap();
        assert_eq!(h5[..], p5[..], "previous() rank 5");
        let p4 = heap2.previous().unwrap();
        assert_eq!(h4[..], p4[..], "previous() rank 4");
        let p3 = heap2.previous().unwrap();
        assert_eq!(h3[..], p3[..], "previous() rank 3");
        let p2 = heap2.previous().unwrap();
        assert_eq!(h2[..], p2[..], "previous() rank 2");
        let p1 = heap2.previous().unwrap();
        assert_eq!(h1[..], p1[..], "previous() rank 1");
        let p0 = heap2.previous().unwrap();
        assert_eq!(h0[..], p0[..], "previous() rank 0");
    }

    #[test]
    fn test_heaps_algo_previous_1_10() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 1..=10usize {
            let mut data: Vec<u8> = (0..(n as u8)).collect();
            let heap = permutohedron::Heap::new(&mut data);
            let mut last_k = 0;
            for (k, p) in (0..).zip(heap).skip(1) {
                let ur = unrank_noprecomp(n, k - 1);

                let mut heap2 = HeapsAlgorithm::at_k(0..(n as u8), k);
                let _kx = heap2.next().unwrap();

                let p2 = heap2.previous().unwrap();
                assert_eq!(ur[..], p2[..], "n={n} k={k} previous() == unrank(k-1)");
                let p3 = heap2.next().unwrap();
                assert_eq!(p, p3[..], "n={n} k={k} next() still works");
                last_k = k;
            }
            assert_eq!(
                (1..=n).product::<usize>(),
                last_k + 1,
                "they should yield factorial(n) elements"
            );
        }
    }

    /// Note: currently disabled because `nth` isn't implemented properly yet:
    #[test]
    fn test_heaps_algo_nth() {
        for n in 1..=10 {
            let mut heap1 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
            let mut last_k = 0;
            let fact = (1..=n).product();
            for k in 0..fact {
                let p1 = heap1.next();
                let p2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>()).nth(k);
                assert_eq!(p1, p2, "n={n} k={k}: from k=0 to nth(k)");
                if k > 1 {
                    //println!("-----nth(k-1) for k={k}");
                    let mut h3 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
                    h3.nth(k - 1);
                    //println!("||  h3.next() after nth(k-1) leaves {:?}", p1);
                    let p3 = h3.next();
                    assert_eq!(p1, p3, "from 0 to nth(k-1);next() is equivalent to nth(k)");
                    if n > 3 {
                        if k > 5 && k < 10 {
                            // test that nth() skips correctly relative to current position
                            //println!("========================= multi-nth:");
                            let x = k - 4;
                            let y = k - x;
                            let mut h2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
                            let _ = h2.nth(x); // x + 1
                            //println!("x=={x} + y=={y} == {:?} k=={k}", x + y);
                            let x2 = h2.nth(y); // y + 1

                            /*println!(
                                "k={k} x1.nth({x}):{:?} x2.nth({y}):{:?} == p1:{:?}",
                                x1, x2, p1
                            );*/
                            //let p4 = h2.next();
                            assert_eq!(
                                p1,
                                x2,
                                //"p4: {:?}", p4
                            );
                        }
                    }
                }
                last_k = k;
            }
            assert_eq!(
                (1..=n).product::<usize>(),
                last_k + 1,
                "they should yield factorial({n}) elements"
            );
        }
    }

    use num_bigint::BigUint;
    #[test]
    fn test_heaps_algo_and_unranking_1_10() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 1..=10usize {
            let heap2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
            let mut last_k = 0;
            for (k, p) in heap2.enumerate() {
                let p2 = unrank_bigint(n, BigUint::from(k));
                assert_eq!(p, p2, "k={k}");
                let k2 = rank_bigint(&p2);
                assert_eq!(k, k2.try_into().expect("ranks for n=10 fit in usize?"));
                last_k = k;
            }
            assert_eq!(
                (1..=n).product::<usize>() - 1,
                last_k,
                "they should yield factorial({n}) elements"
            );
        }
    }
    #[test]
    fn test_heaps_algo_and_unranking_11_40() {
        // check that unrank_bigint matches our own permutations
        for n in 11..=40usize {
            let heap2 = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
            let mut last_k = 0;
            for (k, p) in heap2.enumerate() {
                let p2 = unrank_bigint(n, BigUint::from(k));
                assert_eq!(p, p2, "k={k}");
                let k2 = rank_bigint(&p2);
                assert_eq!(k, k2.try_into().expect("should be <= 50000"));
                last_k = k;
                if k == 50000 {
                    break; // we can't test exhaustively
                }
            }
            assert!(
                last_k == 50000 || (1..=n).product::<usize>() - 1 == last_k,
                "they should yield factorial({n}) elements but got {last_k}"
            );
        }
    }

    #[test]
    fn functional_0_unrank_test() {
        // check that unrank and unrank_recursive agree
        let s = precompute(1);
        (1..20).for_each(|n| {
            let f = unrank_recursive(n, 0);
            let o = unrank(&s, n, 0);
            assert_eq!(o, f.into());
        })
    }
    #[test]
    fn functional_1_unrank_test() {
        // check that unrank and unrank_recursive agree.
        // Since we are only looking at k=1, we don't need the prefix table
        let prefixes = precompute(1);
        (1..20).for_each(|n| {
            let f = unrank_recursive(n, 1);
            let o = unrank(&prefixes, n, 1);
            assert_eq!(o, f.into());
        })
    }

    #[test]
    fn noprecompute_n_unrank_test() {
        // check that unrank and unrank_recursive agree
        let mut _fact: usize = 1;
        for n in 1..11 {
            let s = precompute(n);
            _fact *= n;
            for k in 0.._fact - 1 {
                let f = unrank_noprecomp(n, k);
                let o = unrank(&s, n, k);
                assert_eq!(o, f.into(), "n={n} k={n}");
            }
        }
    }

    #[test]
    fn functional_n_unrank_test() {
        // check that unrank and unrank_recursive agree
        let s = precompute(10);
        let mut _fact: usize = 1;
        for n in 2..9 {
            _fact *= n;
            for k in 0.._fact - 1 {
                let f = unrank_recursive(n, k);
                let o = unrank(&s, n, k);
                assert_eq!(o, f.into());
            }
        }
    }
}
