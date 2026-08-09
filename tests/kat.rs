//
// Known-answer tests for unrank(), rank()
//

#[cfg(test)]
mod kat_tests {
    use heap_unranking::*;

    #[test]
    fn precomp_digit_0_1_2() {
        // [0]
        assert_eq!(precomp_digit(1, 0), 0);
        // [1,0]
        assert_eq!(precomp_digit(2, 0), 1);
        assert_eq!(precomp_digit(2, 1), 0);
        // [2,1,0]
        assert_eq!(precomp_digit(3, 0), 2);
        assert_eq!(precomp_digit(3, 1), 1);
        assert_eq!(precomp_digit(3, 2), 0);
    }

    #[test]
    fn precomp_digit_even() {
        // this is the loop var n starting at array length-1
        for n in [4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32] {
            let origin: Vec<_> = (0..n + 1).collect();
            let mut permutation1 = origin.clone();
            let mut permutation2 = origin.clone();

            // how we currently do it per n-loop:
            {
                let mut scratch = Vec::with_capacity(n);
                let mut precomped_digits = Vec::with_capacity(n);
                precomped_digits.extend((0..n).map(|d| precomp_digit(n, d)));

                scratch.extend(precomped_digits.iter().map(|&d| permutation1[d as usize]));
                permutation1[0..n].copy_from_slice(&scratch);
                permutation1.swap(0, n);
            }
            {
                // for even n >= 4, the pattern we are looking for is something like:
                // [5, 6,   1, 2, 3, 4,   7,   0]  //  n == 8

                // These will get overwritten:
                let fst = permutation2[n - 3]; // [5] will get overwritten
                                               // 5,4,3,2 (when n == 8):
                for x in (2..n - 2).rev() {
                    permutation2[x] = permutation2[x - 1];
                }
                permutation2[1] = permutation2[n - 2]; // [1] < [6]
                permutation2[n - 2] = permutation2[n - 1]; // [6] <- [7]
                permutation2[n - 1] = permutation2[0]; // [7] <- [0]
                permutation2[0] = permutation2[n];
                permutation2[n] = fst; // [8] <- [5]
            }
            assert_eq!(permutation1, permutation2, "these two methods should match");
        }
    }

    #[test]
    fn precomp_digit_odd() {
        // this is the loop var n starting at array length-1
        for n in [5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31] {
            let origin: Vec<_> = (0..n + 1).collect();
            for i in 0..n {
                let mut permutation1 = origin.clone();
                let mut permutation2 = origin.clone();

                // how we currently do it per n-loop:
                {
                    let mut scratch = Vec::with_capacity(n);
                    let mut precomped_digits = Vec::with_capacity(n);
                    precomped_digits.extend((0..n).map(|d| precomp_digit(n, d)));

                    scratch.extend(precomped_digits.iter().map(|&d| permutation1[d as usize]));
                    permutation1[0..n].copy_from_slice(&scratch);
                    permutation1.swap(i, n);
                }
                {
                    // for even n >= 7, the pattern we are looking for is something like:
                    // [6,   1, 2, 3, 4, 5,   0]  //  n == 7
                    //
                    // Note that the middle section doesn't actually move anything, so all we need is:
                    //
                    permutation2.swap(0, n - 1);
                    permutation2.swap(i, n);
                }
                assert_eq!(
                    permutation1, permutation2,
                    "i={:?} these two methods should match",
                    i
                );
            }
        }
    }

    #[test]
    fn precompute_kats() {
        // Known-answer tests for precompute()

        //
        // Note that these look suspiciously regular; do we really need to precompute them?
        // - Depending on whether n is odd or even they have:
        //   - Even: two first elements [n-2],[n-3], second-to-last is n[-1]
        //   - Odd : first element is [n-1]
        // But I'm not convinced this is true for all $n$, can we prove that property?
        //
        // Something like:
        fn f(n: usize, i: usize) -> u8 {
            assert!(i <= n);
            // There's probably a tighter formulation of this with fewer cases?
            if i == n - 1 {
                return 0;
            } // last element is always zero
            if i == n - 2 && n & 1 == 0 {
                return (n - 1) as u8;
            }
            if n == 2 && i == 0 {
                return (1 - i) as u8;
            } // special case for n=2 where we want [1,0]
            match i {
                0 => (n - 3 + (n & 1) * 2) as u8, // for n==2 this underflows and produces -1+0 == -1 but we want 1
                1 if n & 1 == 0 => (n - 2) as u8,
                i => (i - 1 + (n & 1)) as u8,
            }
        }
        let vectors: Vec<Box<[u8]>> = vec![
            Box::new([0]),                                                            // 1
            Box::new([1, 0]),                                                         // 2
            Box::new([2, 1, 0]),                                                      // 3
            Box::new([1, 2, 3, 0]),                                                   // 4
            Box::new([4, 1, 2, 3, 0]),                                                // 5
            Box::new([3, 4, 1, 2, 5, 0]),                                             // 6
            Box::new([6, 1, 2, 3, 4, 5, 0]),                                          // 7
            Box::new([5, 6, 1, 2, 3, 4, 7, 0]),                                       // 8
            Box::new([8, 1, 2, 3, 4, 5, 6, 7, 0]),                                    // 9
            Box::new([7, 8, 1, 2, 3, 4, 5, 6, 9, 0]),                                 // 10
            Box::new([10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0]),                             // 11
            Box::new([9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 11, 0]),                         // 12
            Box::new([12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0]),                     // 13
            Box::new([11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 0]),                 // 14
            Box::new([14, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0]),             // 15
            Box::new([13, 14, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15, 0]),         // 16
            Box::new([16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]),     // 17
            Box::new([15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17, 0]), // 18
            Box::new([
                18, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0,
            ]), // 19
            Box::new([
                17, 18, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 19, 0,
            ]), // 20
            Box::new([
                20, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 0,
            ]), // 21
            Box::new([
                19, 20, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 21, 0,
            ]), // 22
        ];
        for i in 1..=vectors.len() {
            let prefixes = &vectors[0..i];
            assert_eq!(
                precompute(i),
                prefixes,
                "element {:?} doesn't match, expected: {:?}",
                i - 1,
                precompute(i)[i - 1]
            );
            prefixes[prefixes.len() - 1]
                .iter()
                .enumerate()
                .for_each(|(x, &pdigit)| {
                    let p2digit = f(prefixes.len(), x);
                    assert_eq!(pdigit, p2digit, "{i}/{x}");
                    let p3digit = precomp_digit(prefixes.len(), x);
                    assert_eq!(pdigit, p3digit);
                });
        }
    }

    #[test]
    fn rank_kats() {
        let s = precompute(20);
        assert_eq!(0, rank(&s, [0, 1, 2].into()));
        assert_eq!(5, rank(&s, [2, 1, 0].into()));
        assert_eq!(5, rank_noprecomp(&[2, 1, 0]));
        assert_eq!(20, rank(&s, [1, 3, 2, 0].into()));
        assert_eq!(20, rank_noprecomp(&[1, 3, 2, 0]));
        assert_eq!(9, rank(&s, [3, 0, 1, 2, 4].into()));
        assert_eq!(9, rank_noprecomp(&[3, 0, 1, 2, 4]));
        assert_eq!(11, rank_noprecomp(&[0, 1, 3, 2, 4]));
        assert_eq!(16, rank_noprecomp(&[2, 3, 0, 1, 4, 5]));
        assert_eq!(
            39916798,
            rank(&s, [1, 10, 2, 3, 4, 5, 6, 7, 8, 9, 0].into())
        );
        assert_eq!(
            154423787521,
            rank(
                &s,
                [7, 2, 10, 14, 12, 13, 5, 0, 6, 3, 1, 4, 9, 8, 11].into()
            )
        );
        assert_eq!(
            1502989870400,
            rank_noprecomp(&[7, 8, 0, 13, 6, 11, 3, 1, 9, 15, 5, 2, 4, 12, 10, 14])
        );
    }

    #[test]
    fn unrank_kats() {
        let s = precompute(17);
        assert_eq!(unrank(&s, 1, 0), [0].into());
        assert_eq!(unrank(&s, 2, 0), [0, 1].into());
        assert_eq!(unrank(&s, 2, 1), [1, 0].into());
        assert_eq!(unrank(&s, 3, 0), [0, 1, 2].into());
        assert_eq!(unrank(&s, 3, 1), [1, 0, 2].into());
        assert_eq!(unrank(&s, 3, 2), [2, 0, 1].into());
        assert_eq!(unrank(&s, 3, 3), [0, 2, 1].into());
        assert_eq!(unrank(&s, 3, 4), [1, 2, 0].into());
        assert_eq!(unrank(&s, 3, 5), [2, 1, 0].into());
        assert_eq!(unrank(&s, 4, 0), [0, 1, 2, 3].into());
        assert_eq!(unrank(&s, 4, 1), [1, 0, 2, 3].into());
        assert_eq!(unrank(&s, 4, 2), [2, 0, 1, 3].into());
        assert_eq!(unrank(&s, 4, 3), [0, 2, 1, 3].into());
        assert_eq!(unrank(&s, 4, 4), [1, 2, 0, 3].into());
        assert_eq!(unrank(&s, 4, 5), [2, 1, 0, 3].into());
        assert_eq!(unrank(&s, 4, 6), [3, 1, 0, 2].into());
        assert_eq!(unrank(&s, 4, 7), [1, 3, 0, 2].into());
        assert_eq!(unrank(&s, 4, 8), [0, 3, 1, 2].into());
        assert_eq!(unrank(&s, 4, 9), [3, 0, 1, 2].into());
        assert_eq!(unrank(&s, 4, 10), [1, 0, 3, 2].into());
        assert_eq!(unrank(&s, 4, 11), [0, 1, 3, 2].into());
        assert_eq!(unrank(&s, 4, 12), [0, 2, 3, 1].into());
        assert_eq!(unrank(&s, 4, 13), [2, 0, 3, 1].into());
        assert_eq!(unrank(&s, 4, 14), [3, 0, 2, 1].into());
        assert_eq!(unrank(&s, 4, 15), [0, 3, 2, 1].into());
        assert_eq!(unrank(&s, 4, 16), [2, 3, 0, 1].into());
        assert_eq!(unrank(&s, 4, 17), [3, 2, 0, 1].into());
        assert_eq!(unrank(&s, 4, 18), [3, 2, 1, 0].into());
        assert_eq!(unrank(&s, 4, 19), [2, 3, 1, 0].into());
        assert_eq!(unrank(&s, 4, 20), [1, 3, 2, 0].into());
        assert_eq!(unrank(&s, 4, 21), [3, 1, 2, 0].into());
        assert_eq!(unrank(&s, 4, 22), [2, 1, 3, 0].into());
        assert_eq!(unrank(&s, 4, 23), [1, 2, 3, 0].into());
        assert_eq!(unrank(&s, 5, 0), [0, 1, 2, 3, 4].into());
        assert_eq!(unrank(&s, 5, 1), [1, 0, 2, 3, 4].into());
        assert_eq!(unrank(&s, 5, 2), [2, 0, 1, 3, 4].into());
        assert_eq!(unrank(&s, 5, 3), [0, 2, 1, 3, 4].into());
        assert_eq!(unrank(&s, 5, 4), [1, 2, 0, 3, 4].into());
        assert_eq!(unrank(&s, 5, 5), [2, 1, 0, 3, 4].into());
        assert_eq!(unrank(&s, 5, 6), [3, 1, 0, 2, 4].into());
        assert_eq!(unrank(&s, 5, 7), [1, 3, 0, 2, 4].into());
        assert_eq!(unrank(&s, 5, 8), [0, 3, 1, 2, 4].into());
        assert_eq!(unrank(&s, 5, 9), [3, 0, 1, 2, 4].into());
        assert_eq!(unrank(&s, 5, 10), [1, 0, 3, 2, 4].into());
        assert_eq!(unrank(&s, 5, 11), [0, 1, 3, 2, 4].into());
        assert_eq!(unrank(&s, 5, 12), [0, 2, 3, 1, 4].into());
        assert_eq!(unrank(&s, 5, 13), [2, 0, 3, 1, 4].into());
        assert_eq!(unrank(&s, 5, 14), [3, 0, 2, 1, 4].into());
        assert_eq!(unrank(&s, 5, 15), [0, 3, 2, 1, 4].into());
        assert_eq!(unrank(&s, 5, 16), [2, 3, 0, 1, 4].into());
        assert_eq!(unrank(&s, 5, 17), [3, 2, 0, 1, 4].into());
        assert_eq!(unrank(&s, 5, 18), [3, 2, 1, 0, 4].into());
        assert_eq!(unrank(&s, 5, 19), [2, 3, 1, 0, 4].into());
        assert_eq!(unrank(&s, 5, 20), [1, 3, 2, 0, 4].into());
        assert_eq!(unrank(&s, 5, 21), [3, 1, 2, 0, 4].into());
        assert_eq!(unrank(&s, 5, 22), [2, 1, 3, 0, 4].into());
        assert_eq!(unrank(&s, 5, 23), [1, 2, 3, 0, 4].into());
        assert_eq!(unrank(&s, 5, 24), [4, 2, 3, 0, 1].into());
        assert_eq!(unrank(&s, 5, 25), [2, 4, 3, 0, 1].into());
        assert_eq!(unrank(&s, 5, 26), [3, 4, 2, 0, 1].into());
        assert_eq!(unrank(&s, 5, 27), [4, 3, 2, 0, 1].into());
        assert_eq!(unrank(&s, 5, 28), [2, 3, 4, 0, 1].into());
        assert_eq!(unrank(&s, 5, 29), [3, 2, 4, 0, 1].into());
        assert_eq!(unrank(&s, 6, 0), [0, 1, 2, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 1), [1, 0, 2, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 2), [2, 0, 1, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 3), [0, 2, 1, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 4), [1, 2, 0, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 5), [2, 1, 0, 3, 4, 5].into());
        assert_eq!(unrank(&s, 6, 6), [3, 1, 0, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 7), [1, 3, 0, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 8), [0, 3, 1, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 9), [3, 0, 1, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 10), [1, 0, 3, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 11), [0, 1, 3, 2, 4, 5].into());
        assert_eq!(unrank(&s, 6, 12), [0, 2, 3, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 13), [2, 0, 3, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 14), [3, 0, 2, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 15), [0, 3, 2, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 16), [2, 3, 0, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 17), [3, 2, 0, 1, 4, 5].into());
        assert_eq!(unrank(&s, 6, 18), [3, 2, 1, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 19), [2, 3, 1, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 20), [1, 3, 2, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 21), [3, 1, 2, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 22), [2, 1, 3, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 23), [1, 2, 3, 0, 4, 5].into());
        assert_eq!(unrank(&s, 6, 24), [4, 2, 3, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 25), [2, 4, 3, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 26), [3, 4, 2, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 27), [4, 3, 2, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 28), [2, 3, 4, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 29), [3, 2, 4, 0, 1, 5].into());
        assert_eq!(unrank(&s, 6, 239), [0, 1, 2, 3, 5, 4].into());
        assert_eq!(unrank(&s, 6, 240), [0, 4, 2, 3, 5, 1].into());
        assert_eq!(unrank(&s, 7, 0), [0, 1, 2, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 1), [1, 0, 2, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 2), [2, 0, 1, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 3), [0, 2, 1, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 4), [1, 2, 0, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 5), [2, 1, 0, 3, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 6), [3, 1, 0, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 7), [1, 3, 0, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 8), [0, 3, 1, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 9), [3, 0, 1, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 10), [1, 0, 3, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 11), [0, 1, 3, 2, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 12), [0, 2, 3, 1, 4, 5, 6].into());
        assert_eq!(unrank(&s, 7, 5038), [1, 6, 2, 3, 4, 5, 0].into());
        assert_eq!(unrank(&s, 7, 5039), [6, 1, 2, 3, 4, 5, 0].into());
        assert_eq!(unrank(&s, 8, 40318), [6, 5, 1, 2, 3, 4, 7, 0].into());
        assert_eq!(unrank(&s, 8, 40319), [5, 6, 1, 2, 3, 4, 7, 0].into());
        assert_eq!(unrank(&s, 9, 1), [1, 0, 2, 3, 4, 5, 6, 7, 8].into());
        assert_eq!(unrank(&s, 9, 362878), [1, 8, 2, 3, 4, 5, 6, 7, 0].into());
        assert_eq!(unrank(&s, 9, 362879), [8, 1, 2, 3, 4, 5, 6, 7, 0].into());
        // 10
        assert_eq!(unrank(&s, 10, 1), [1, 0, 2, 3, 4, 5, 6, 7, 8, 9].into());
        assert_eq!(
            unrank(&s, 10, 3628798),
            [8, 7, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 10, 1048577),
            [4, 3, 7, 5, 9, 2, 8, 6, 0, 1].into()
        );
        assert_eq!(
            unrank(&s, 10, 2097153),
            [6, 9, 2, 3, 0, 5, 1, 8, 7, 4].into()
        );
        assert_eq!(
            unrank(&s, 10, 3145729),
            [4, 0, 2, 6, 3, 8, 9, 1, 5, 7].into()
        );
        assert_eq!(
            unrank(&s, 10, 3628799),
            [7, 8, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 10, 3628799),
            [7, 8, 1, 2, 3, 4, 5, 6, 9, 0].into()
        );
        // 11
        assert_eq!(
            unrank(&s, 11, 39916798),
            [1, 10, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 11, 39916799),
            [10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        assert_eq!(
            unrank(&s, 11, 39916799),
            [10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0].into()
        );
        // 12
        assert_eq!(
            unrank(&s, 12, 479001598),
            [10, 9, 1, 2, 3, 4, 5, 6, 7, 8, 11, 0].into()
        );
        assert_eq!(
            unrank(&s, 12, 479001599),
            [9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 11, 0].into()
        );
        assert_eq!(
            unrank(&s, 12, 100663297),
            [4, 9, 5, 2, 7, 6, 0, 8, 11, 10, 3, 1].into()
        );
        assert_eq!(
            unrank(&s, 12, 52428801),
            [1, 9, 6, 4, 0, 3, 2, 7, 11, 8, 5, 10].into()
        );
        assert_eq!(
            unrank(&s, 12, 146800641),
            [8, 11, 0, 9, 3, 1, 4, 6, 5, 7, 10, 2].into()
        );
        assert_eq!(
            unrank(&s, 12, 477102081),
            [7, 2, 1, 8, 4, 5, 9, 10, 6, 3, 11, 0].into()
        );
        // 13
        assert_eq!(
            unrank(&s, 13, 319815681),
            [2, 1, 6, 5, 10, 0, 9, 4, 3, 8, 11, 7, 12].into()
        );
        assert_eq!(
            unrank(&s, 13, 634388481),
            [10, 7, 2, 6, 11, 12, 3, 4, 5, 0, 8, 1, 9].into()
        );
        assert_eq!(
            unrank(&s, 13, 1347420161),
            [11, 12, 10, 3, 4, 0, 1, 7, 9, 2, 5, 6, 8].into()
        );
        assert_eq!(
            unrank(&s, 13, 2107637761),
            [1, 9, 8, 0, 2, 10, 7, 3, 12, 5, 4, 11, 6].into()
        );
        assert_eq!(
            unrank(&s, 13, 2144337921),
            [8, 11, 2, 5, 3, 1, 0, 4, 7, 12, 9, 10, 6].into()
        );
        assert_eq!(
            unrank(&s, 13, 3523215361),
            [1, 8, 12, 5, 2, 7, 6, 4, 0, 10, 11, 9, 3].into()
        );
        assert_eq!(
            unrank(&s, 13, 5022679041),
            [9, 8, 6, 3, 2, 0, 1, 5, 12, 4, 11, 7, 10].into()
        );
        assert_eq!(
            unrank(&s, 13, 6223298561),
            [6, 10, 5, 12, 3, 2, 4, 1, 7, 8, 9, 11, 0].into()
        );
        // 14
        assert_eq!(
            unrank(&s, 14, 3344957441),
            [8, 10, 3, 11, 12, 7, 9, 0, 2, 6, 1, 5, 4, 13].into()
        );
        assert_eq!(
            unrank(&s, 14, 4902092801),
            [9, 12, 8, 3, 6, 2, 7, 5, 1, 11, 0, 4, 10, 13].into()
        );
        assert_eq!(
            unrank(&s, 14, 7932477441),
            [11, 5, 1, 8, 3, 4, 9, 10, 6, 13, 0, 2, 7, 12].into()
        );
        assert_eq!(
            unrank(&s, 14, 16420700161),
            [8, 3, 5, 10, 0, 12, 6, 4, 13, 11, 9, 7, 2, 1].into()
        );
        assert_eq!(
            unrank(&s, 14, 16583229441),
            [13, 10, 12, 5, 11, 6, 3, 4, 9, 7, 8, 0, 2, 1].into()
        );
        // 15
        assert_eq!(
            unrank(&s, 15, 2233466881),
            [5, 9, 12, 0, 3, 4, 10, 1, 7, 11, 8, 2, 6, 13, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 11141120001),
            [6, 9, 13, 3, 8, 1, 7, 4, 2, 11, 0, 5, 10, 12, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 14905507841),
            [13, 12, 6, 10, 4, 0, 7, 9, 2, 3, 11, 8, 5, 1, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 24363663361),
            [4, 10, 0, 9, 12, 8, 1, 5, 3, 7, 6, 13, 11, 2, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 31111249921),
            [13, 6, 2, 7, 10, 5, 12, 1, 8, 9, 4, 11, 0, 3, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 44281364481),
            [11, 4, 8, 1, 5, 10, 13, 7, 0, 12, 3, 2, 9, 6, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 55343841281),
            [5, 12, 4, 13, 0, 10, 1, 8, 2, 9, 3, 6, 11, 7, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 66091745281),
            [5, 8, 6, 11, 13, 0, 4, 12, 1, 7, 10, 3, 2, 9, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 80625008641),
            [9, 5, 10, 1, 6, 7, 4, 3, 8, 12, 13, 2, 0, 11, 14].into()
        );
        assert_eq!(
            unrank(&s, 15, 89474990081),
            [9, 12, 8, 6, 7, 2, 1, 4, 13, 10, 14, 3, 5, 0, 11].into()
        );
        assert_eq!(
            unrank(&s, 15, 140005867521),
            [1, 12, 5, 10, 9, 3, 4, 7, 13, 0, 14, 8, 2, 6, 11].into()
        );
        assert_eq!(
            unrank(&s, 15, 154423787521),
            [7, 2, 10, 14, 12, 13, 5, 0, 6, 3, 1, 4, 9, 8, 11].into()
        );
        assert_eq!(
            unrank(&s, 16, 60317166),
            [4, 6, 1, 11, 0, 2, 7, 5, 9, 8, 3, 10, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 63040392),
            [11, 5, 4, 0, 9, 1, 6, 3, 8, 7, 2, 10, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 101976303),
            [10, 4, 11, 0, 9, 6, 7, 8, 3, 5, 2, 1, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 449569912),
            [3, 9, 8, 10, 11, 1, 7, 2, 6, 4, 5, 0, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 453709146),
            [4, 2, 8, 1, 10, 6, 9, 11, 7, 5, 3, 0, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 462443280),
            [3, 7, 11, 5, 10, 8, 4, 2, 6, 9, 1, 0, 12, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 791652516),
            [0, 2, 12, 11, 6, 7, 3, 4, 1, 10, 8, 5, 9, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1536412922),
            [6, 3, 4, 11, 12, 5, 1, 8, 2, 9, 10, 0, 7, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1719411164),
            [1, 6, 5, 12, 2, 0, 10, 9, 11, 4, 8, 3, 7, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1947450820),
            [12, 2, 0, 3, 5, 1, 10, 7, 11, 9, 4, 8, 6, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 2110962693),
            [3, 7, 0, 12, 4, 2, 10, 9, 1, 8, 5, 11, 6, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 2168025930),
            [3, 12, 8, 11, 5, 0, 9, 2, 7, 4, 10, 1, 6, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 2556786030),
            [6, 10, 8, 3, 12, 1, 4, 9, 2, 11, 7, 0, 5, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 2871869308),
            [12, 1, 10, 9, 4, 11, 8, 3, 2, 0, 7, 6, 5, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 3437360017),
            [4, 6, 5, 11, 2, 0, 12, 9, 8, 1, 10, 7, 3, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 5167046400),
            [9, 2, 3, 12, 1, 8, 11, 4, 7, 5, 6, 0, 10, 13, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 7577732176),
            [1, 0, 11, 2, 13, 5, 9, 3, 4, 10, 7, 6, 8, 12, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 10166721673),
            [13, 0, 11, 4, 9, 7, 8, 10, 1, 3, 5, 6, 2, 12, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 15479064468),
            [7, 3, 13, 10, 11, 0, 8, 6, 12, 5, 2, 9, 4, 1, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 18675635160),
            [10, 5, 13, 2, 12, 7, 4, 3, 8, 6, 9, 11, 0, 1, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 21078023364),
            [9, 12, 13, 4, 3, 10, 6, 1, 0, 11, 8, 7, 5, 2, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 30180649400),
            [4, 9, 0, 7, 6, 5, 2, 13, 8, 10, 1, 12, 11, 3, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 40324897360),
            [1, 13, 0, 2, 12, 9, 11, 6, 4, 7, 10, 8, 3, 5, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 50153476428),
            [10, 9, 0, 4, 8, 1, 12, 5, 11, 3, 2, 6, 13, 7, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 60088529700),
            [13, 11, 10, 3, 4, 2, 12, 5, 7, 6, 0, 9, 1, 8, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 70761817190),
            [1, 8, 4, 12, 6, 7, 9, 13, 11, 0, 3, 2, 5, 10, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 82600898362),
            [2, 7, 13, 4, 8, 10, 9, 5, 3, 11, 1, 12, 6, 0, 14, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 91789074340),
            [1, 3, 6, 14, 9, 8, 4, 2, 7, 10, 5, 13, 12, 0, 11, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 101839688049),
            [4, 13, 2, 0, 8, 6, 3, 14, 10, 7, 9, 1, 5, 12, 11, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 113912598800),
            [1, 5, 13, 8, 14, 0, 12, 7, 10, 9, 3, 4, 6, 2, 11, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 124470168804),
            [6, 10, 2, 13, 4, 9, 12, 8, 7, 1, 5, 14, 0, 3, 11, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 211654560000),
            [5, 0, 7, 8, 4, 12, 13, 3, 6, 9, 1, 11, 14, 2, 10, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 253031122911),
            [1, 6, 4, 2, 0, 11, 8, 5, 7, 14, 12, 3, 13, 9, 10, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 294098519628),
            [0, 12, 8, 13, 3, 11, 7, 14, 10, 4, 2, 6, 5, 1, 9, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 320356550602),
            [2, 4, 3, 14, 10, 7, 11, 8, 0, 12, 6, 13, 1, 5, 9, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 503852278032),
            [2, 13, 3, 0, 9, 5, 1, 10, 11, 12, 14, 8, 6, 4, 7, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 641911891467),
            [1, 3, 6, 7, 11, 12, 9, 8, 13, 10, 0, 4, 2, 14, 5, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 749464253344),
            [11, 12, 5, 0, 6, 10, 7, 9, 3, 2, 1, 8, 14, 13, 4, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 871325984320),
            [14, 0, 11, 13, 2, 6, 9, 1, 7, 10, 8, 12, 5, 4, 3, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 959236375660),
            [7, 11, 13, 10, 12, 14, 5, 6, 8, 2, 0, 9, 4, 3, 1, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1052749187750),
            [14, 5, 0, 13, 10, 6, 7, 9, 8, 2, 4, 11, 1, 3, 12, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1501474071468),
            [9, 15, 1, 13, 11, 8, 2, 5, 6, 0, 4, 3, 7, 12, 10, 14].into()
        );

        assert_eq!(
            unrank(&s, 16, 1214115477200),
            [3, 2, 6, 14, 4, 11, 10, 1, 7, 5, 8, 9, 12, 0, 13, 15].into()
        );
        assert_eq!(
            unrank(&s, 16, 1307708436426),
            [3, 5, 8, 1, 2, 6, 7, 10, 15, 4, 9, 11, 12, 13, 0, 14].into()
        );
        assert_eq!(
            unrank(&s, 16, 1347314059074),
            [1, 2, 9, 12, 15, 4, 11, 8, 13, 10, 7, 3, 6, 5, 0, 14].into()
        );
        assert_eq!(
            unrank(&s, 16, 1403307163908),
            [6, 15, 8, 2, 10, 7, 12, 0, 9, 4, 3, 1, 5, 13, 11, 14].into()
        );
        assert_eq!(
            unrank(&s, 16, 1459635106812),
            [5, 9, 12, 13, 1, 6, 15, 10, 2, 0, 4, 7, 3, 8, 11, 14].into()
        );
        assert_eq!(
            unrank(&s, 16, 1502989870400),
            [7, 8, 0, 13, 6, 11, 3, 1, 9, 15, 5, 2, 4, 12, 10, 14].into()
        );
        assert_eq!(
            unrank_noprecomp(16, 1502989870400),
            [7, 8, 0, 13, 6, 11, 3, 1, 9, 15, 5, 2, 4, 12, 10, 14].into()
        );
    }

    #[test]
    fn test_rank_noprecompute_5() {
        assert_eq!(0, rank_noprecomp(&[0, 1, 2, 3, 4]));
        assert_eq!(1, rank_noprecomp(&[1, 0, 2, 3, 4]));
        assert_eq!(2, rank_noprecomp(&[2, 0, 1, 3, 4]));
        assert_eq!(3, rank_noprecomp(&[0, 2, 1, 3, 4]));
        assert_eq!(4, rank_noprecomp(&[1, 2, 0, 3, 4]));
        assert_eq!(5, rank_noprecomp(&[2, 1, 0, 3, 4]));
        assert_eq!(6, rank_noprecomp(&[3, 1, 0, 2, 4]));
        assert_eq!(7, rank_noprecomp(&[1, 3, 0, 2, 4]));
        assert_eq!(8, rank_noprecomp(&[0, 3, 1, 2, 4]));
        assert_eq!(9, rank_noprecomp(&[3, 0, 1, 2, 4]));
        assert_eq!(10, rank_noprecomp(&[1, 0, 3, 2, 4]));
        assert_eq!(11, rank_noprecomp(&[0, 1, 3, 2, 4]));
        assert_eq!(12, rank_noprecomp(&[0, 2, 3, 1, 4]));
        assert_eq!(13, rank_noprecomp(&[2, 0, 3, 1, 4]));
        assert_eq!(14, rank_noprecomp(&[3, 0, 2, 1, 4]));
        assert_eq!(15, rank_noprecomp(&[0, 3, 2, 1, 4]));
        assert_eq!(16, rank_noprecomp(&[2, 3, 0, 1, 4]));
        assert_eq!(17, rank_noprecomp(&[3, 2, 0, 1, 4]));
        assert_eq!(18, rank_noprecomp(&[3, 2, 1, 0, 4]));
        assert_eq!(19, rank_noprecomp(&[2, 3, 1, 0, 4]));
        assert_eq!(20, rank_noprecomp(&[1, 3, 2, 0, 4]));
        assert_eq!(21, rank_noprecomp(&[3, 1, 2, 0, 4]));
        assert_eq!(22, rank_noprecomp(&[2, 1, 3, 0, 4]));
        assert_eq!(23, rank_noprecomp(&[1, 2, 3, 0, 4]));
        assert_eq!(24, rank_noprecomp(&[4, 2, 3, 0, 1]));
        assert_eq!(25, rank_noprecomp(&[2, 4, 3, 0, 1]));
        assert_eq!(26, rank_noprecomp(&[3, 4, 2, 0, 1]));
        assert_eq!(27, rank_noprecomp(&[4, 3, 2, 0, 1]));
        assert_eq!(28, rank_noprecomp(&[2, 3, 4, 0, 1]));
        assert_eq!(29, rank_noprecomp(&[3, 2, 4, 0, 1]));
        assert_eq!(30, rank_noprecomp(&[0, 2, 4, 3, 1]));
        assert_eq!(31, rank_noprecomp(&[2, 0, 4, 3, 1]));
        assert_eq!(32, rank_noprecomp(&[4, 0, 2, 3, 1]));
        assert_eq!(33, rank_noprecomp(&[0, 4, 2, 3, 1]));
        assert_eq!(34, rank_noprecomp(&[2, 4, 0, 3, 1]));
        assert_eq!(35, rank_noprecomp(&[4, 2, 0, 3, 1]));
        assert_eq!(36, rank_noprecomp(&[4, 3, 0, 2, 1]));
        assert_eq!(37, rank_noprecomp(&[3, 4, 0, 2, 1]));
        assert_eq!(38, rank_noprecomp(&[0, 4, 3, 2, 1]));
        assert_eq!(39, rank_noprecomp(&[4, 0, 3, 2, 1]));
        assert_eq!(40, rank_noprecomp(&[3, 0, 4, 2, 1]));
        assert_eq!(41, rank_noprecomp(&[0, 3, 4, 2, 1]));
        assert_eq!(42, rank_noprecomp(&[0, 3, 2, 4, 1]));
        assert_eq!(43, rank_noprecomp(&[3, 0, 2, 4, 1]));
        assert_eq!(44, rank_noprecomp(&[2, 0, 3, 4, 1]));
        assert_eq!(45, rank_noprecomp(&[0, 2, 3, 4, 1]));
        assert_eq!(46, rank_noprecomp(&[3, 2, 0, 4, 1]));
        assert_eq!(47, rank_noprecomp(&[2, 3, 0, 4, 1]));
        assert_eq!(48, rank_noprecomp(&[1, 3, 0, 4, 2]));
        assert_eq!(49, rank_noprecomp(&[3, 1, 0, 4, 2]));
        assert_eq!(50, rank_noprecomp(&[0, 1, 3, 4, 2]));
        assert_eq!(51, rank_noprecomp(&[1, 0, 3, 4, 2]));
        assert_eq!(52, rank_noprecomp(&[3, 0, 1, 4, 2]));
        assert_eq!(53, rank_noprecomp(&[0, 3, 1, 4, 2]));
        assert_eq!(54, rank_noprecomp(&[4, 3, 1, 0, 2]));
        assert_eq!(55, rank_noprecomp(&[3, 4, 1, 0, 2]));
        assert_eq!(56, rank_noprecomp(&[1, 4, 3, 0, 2]));
        assert_eq!(57, rank_noprecomp(&[4, 1, 3, 0, 2]));
        assert_eq!(58, rank_noprecomp(&[3, 1, 4, 0, 2]));
        assert_eq!(59, rank_noprecomp(&[1, 3, 4, 0, 2]));
        assert_eq!(60, rank_noprecomp(&[1, 0, 4, 3, 2]));
        assert_eq!(61, rank_noprecomp(&[0, 1, 4, 3, 2]));
        assert_eq!(62, rank_noprecomp(&[4, 1, 0, 3, 2]));
        assert_eq!(63, rank_noprecomp(&[1, 4, 0, 3, 2]));
        assert_eq!(64, rank_noprecomp(&[0, 4, 1, 3, 2]));
        assert_eq!(65, rank_noprecomp(&[4, 0, 1, 3, 2]));
        assert_eq!(66, rank_noprecomp(&[4, 0, 3, 1, 2]));
        assert_eq!(67, rank_noprecomp(&[0, 4, 3, 1, 2]));
        assert_eq!(68, rank_noprecomp(&[3, 4, 0, 1, 2]));
        assert_eq!(69, rank_noprecomp(&[4, 3, 0, 1, 2]));
        assert_eq!(70, rank_noprecomp(&[0, 3, 4, 1, 2]));
        assert_eq!(71, rank_noprecomp(&[3, 0, 4, 1, 2]));
        assert_eq!(72, rank_noprecomp(&[2, 0, 4, 1, 3]));
        assert_eq!(73, rank_noprecomp(&[0, 2, 4, 1, 3]));
        assert_eq!(74, rank_noprecomp(&[4, 2, 0, 1, 3]));
        assert_eq!(75, rank_noprecomp(&[2, 4, 0, 1, 3]));
        assert_eq!(76, rank_noprecomp(&[0, 4, 2, 1, 3]));
        assert_eq!(77, rank_noprecomp(&[4, 0, 2, 1, 3]));
        assert_eq!(78, rank_noprecomp(&[1, 0, 2, 4, 3]));
        assert_eq!(79, rank_noprecomp(&[0, 1, 2, 4, 3]));
        assert_eq!(80, rank_noprecomp(&[2, 1, 0, 4, 3]));
        assert_eq!(81, rank_noprecomp(&[1, 2, 0, 4, 3]));
        assert_eq!(82, rank_noprecomp(&[0, 2, 1, 4, 3]));
        assert_eq!(83, rank_noprecomp(&[2, 0, 1, 4, 3]));
        assert_eq!(84, rank_noprecomp(&[2, 4, 1, 0, 3]));
        assert_eq!(85, rank_noprecomp(&[4, 2, 1, 0, 3]));
        assert_eq!(86, rank_noprecomp(&[1, 2, 4, 0, 3]));
        assert_eq!(87, rank_noprecomp(&[2, 1, 4, 0, 3]));
        assert_eq!(88, rank_noprecomp(&[4, 1, 2, 0, 3]));
        assert_eq!(89, rank_noprecomp(&[1, 4, 2, 0, 3]));
        assert_eq!(90, rank_noprecomp(&[1, 4, 0, 2, 3]));
        assert_eq!(91, rank_noprecomp(&[4, 1, 0, 2, 3]));
        assert_eq!(92, rank_noprecomp(&[0, 1, 4, 2, 3]));
        assert_eq!(93, rank_noprecomp(&[1, 0, 4, 2, 3]));
        assert_eq!(94, rank_noprecomp(&[4, 0, 1, 2, 3]));
        assert_eq!(95, rank_noprecomp(&[0, 4, 1, 2, 3]));
        assert_eq!(96, rank_noprecomp(&[3, 4, 1, 2, 0]));
        assert_eq!(97, rank_noprecomp(&[4, 3, 1, 2, 0]));
        assert_eq!(98, rank_noprecomp(&[1, 3, 4, 2, 0]));
        assert_eq!(99, rank_noprecomp(&[3, 1, 4, 2, 0]));
        assert_eq!(100, rank_noprecomp(&[4, 1, 3, 2, 0]));
        assert_eq!(101, rank_noprecomp(&[1, 4, 3, 2, 0]));
        assert_eq!(102, rank_noprecomp(&[2, 4, 3, 1, 0]));
        assert_eq!(103, rank_noprecomp(&[4, 2, 3, 1, 0]));
        assert_eq!(104, rank_noprecomp(&[3, 2, 4, 1, 0]));
        assert_eq!(105, rank_noprecomp(&[2, 3, 4, 1, 0]));
        assert_eq!(106, rank_noprecomp(&[4, 3, 2, 1, 0]));
        assert_eq!(107, rank_noprecomp(&[3, 4, 2, 1, 0]));
        assert_eq!(108, rank_noprecomp(&[3, 1, 2, 4, 0]));
        assert_eq!(109, rank_noprecomp(&[1, 3, 2, 4, 0]));
        assert_eq!(110, rank_noprecomp(&[2, 3, 1, 4, 0]));
        assert_eq!(111, rank_noprecomp(&[3, 2, 1, 4, 0]));
        assert_eq!(112, rank_noprecomp(&[1, 2, 3, 4, 0]));
        assert_eq!(113, rank_noprecomp(&[2, 1, 3, 4, 0]));
        assert_eq!(114, rank_noprecomp(&[2, 1, 4, 3, 0]));
        assert_eq!(115, rank_noprecomp(&[1, 2, 4, 3, 0]));
        assert_eq!(116, rank_noprecomp(&[4, 2, 1, 3, 0]));
        assert_eq!(117, rank_noprecomp(&[2, 4, 1, 3, 0]));
        assert_eq!(118, rank_noprecomp(&[1, 4, 2, 3, 0]));
        assert_eq!(119, rank_noprecomp(&[4, 1, 2, 3, 0]));
    }
}
