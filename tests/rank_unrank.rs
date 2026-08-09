#[cfg(test)]
mod unittests {
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
            for _ in 0..=((1..=n).product::<usize>().min(n * n * 2_000)) {
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
    fn test_rank_noprecomp_matches_output_0_10() {
        // check that rank_noprecomp() matches the traditional Heap's algorithm's
        // outputs (from the permutohedron crate):
        for n in 2..=10usize {
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

    #[test]
    fn get_qs_test() {
        // test that the recursive implementation of `get_qs` matches
        // the imperative version used in unrank()
        for n in 1..10 {
            for k2 in 1..120 {
                let mut funct = get_qs(n, 1, k2, vec![]);
                funct.reverse();
                let mut k = k2;
                let mut qs = vec![0usize; n - 1].into_boxed_slice();
                for (q, i) in qs.iter_mut().zip(2usize..) {
                    *q = k % i;
                    k /= i;
                }
                assert_eq!(qs, funct.into());
            }
        }
    }
}
