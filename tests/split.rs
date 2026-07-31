#[cfg(test)]
mod split_tests {
    use heap_unranking::*;

    // A u128 holds 34!, well past the 20! a usize rank can address, so the boundaries of
    // the large n that motivate splitting can still be checked against exact arithmetic.
    // The reference scales n! by a job index first, which costs the top few n.
    const MAX_EXACT_N: usize = 30;

    fn factorial(n: usize) -> u128 {
        (1..=n as u128).product()
    }

    fn to_u128(f: &Factoradic) -> u128 {
        f.digits()
            .iter()
            .enumerate()
            .rev()
            .fold(0u128, |k, (i, &d)| k * (i as u128 + 1) + d as u128)
    }

    const PARTS: [usize; 7] = [1, 2, 3, 7, 64, 1000, 65537];

    #[test]
    fn boundaries_are_exact_divisions() {
        for n in 1..=MAX_EXACT_N {
            let fact = factorial(n);
            for parts in PARTS {
                let boundaries = split_factoradic(n, parts);
                assert_eq!(boundaries.len(), parts + 1);
                for (j, b) in boundaries.iter().enumerate() {
                    assert_eq!(to_u128(b), j as u128 * fact / parts as u128, "n={n} j={j}");
                }
            }
        }
    }

    #[test]
    fn boundaries_are_valid_factoradics() {
        for n in 1..=MAX_EXACT_N {
            for b in split_factoradic(n, 97) {
                for (i, &d) in b.digits().iter().enumerate() {
                    assert!(d as usize <= i, "digit {d} at place {i} is out of range");
                }
            }
        }
    }

    #[test]
    fn spans_tile_the_range() {
        for n in 1..=MAX_EXACT_N {
            for parts in PARTS {
                let boundaries = split_factoradic(n, parts);
                assert_eq!(to_u128(&boundaries[0]), 0);
                assert_eq!(to_u128(boundaries.last().unwrap()), factorial(n));

                // Even means even to within one permutation, and nobody gets an empty
                // span unless there are more jobs than permutations.
                let lengths: Vec<u128> = boundaries
                    .windows(2)
                    .map(|w| to_u128(&w[1]) - to_u128(&w[0]))
                    .collect();
                let min = *lengths.iter().min().unwrap();
                let max = *lengths.iter().max().unwrap();
                assert!(max - min <= 1, "n={n} parts={parts}: {min}..{max}");
                assert_eq!(lengths.iter().sum::<u128>(), factorial(n));
            }
        }
    }

    #[test]
    fn split_boundary_matches_the_vector() {
        for n in 1..=MAX_EXACT_N {
            for parts in [1usize, 2, 3, 7, 64, 1000] {
                let boundaries = split_factoradic(n, parts);
                for (index, b) in boundaries.iter().enumerate() {
                    assert_eq!(*b, split_boundary(n, parts, index), "n={n} index={index}");
                }
            }
        }
    }

    #[test]
    fn split_ranks_matches_split_factoradic() {
        for n in 1..=20 {
            for parts in PARTS {
                let ranks = split_ranks(n, parts);
                for (r, b) in ranks.iter().zip(split_factoradic(n, parts)) {
                    assert_eq!(*r as u128, to_u128(&b));
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "exceeds a usize")]
    fn split_ranks_rejects_ranks_beyond_usize() {
        // 21! is the first factorial a usize cannot hold, and it is the last boundary.
        split_ranks(21, 4);
    }

    #[test]
    #[should_panic(expected = "divisor must be positive")]
    fn split_rejects_zero_parts() {
        split_factoradic(8, 0);
    }

    #[test]
    fn factorial_div_rem_agrees_with_exact_arithmetic() {
        for n in 1..=MAX_EXACT_N {
            for divisor in [1usize, 2, 3, 7, 64, 1000, 65537] {
                let (quotient, rem) = factorial_div_rem(n, divisor);
                assert_eq!(to_u128(&quotient), factorial(n) / divisor as u128);
                assert_eq!(rem as u128, factorial(n) % divisor as u128);
            }
        }
    }

    #[test]
    fn factoradic_round_trips_through_usize() {
        for n in 1..=20 {
            for k in [0usize, 1, 2, 41, 6187, usize::MAX / 3] {
                let k = k % (1..=n).product::<usize>().max(1);
                assert_eq!(Factoradic::from_rank(k, n).to_rank(), Some(k));
            }
        }
    }

    #[test]
    fn factoradic_arithmetic_matches_usize_arithmetic() {
        for n in 6..=20 {
            let fact: usize = (1..=n).product();
            for (a, b) in [(0usize, 0usize), (1, 1), (719, 5039), (fact - 1, 1)] {
                let (a, b) = (a % fact, b % fact);
                let mut sum = Factoradic::from_rank(a, n);
                sum.add_assign(&Factoradic::from_rank(b, n));
                assert_eq!(sum.to_rank(), Some(a + b));

                let mut difference = sum;
                difference.sub_assign(&Factoradic::from_rank(b, n));
                assert_eq!(difference.to_rank(), Some(a));

                let (a, b) = (a % 97, b % 97); // room for the product below n!
                let mut scaled = Factoradic::from_rank(a, n);
                scaled.scale(37);
                scaled.add_rank(b);
                assert_eq!(scaled.to_rank(), Some(a * 37 + b));
            }
        }
    }

    #[test]
    fn unrank_factoradic_matches_unrank() {
        let s = precompute(20);
        for n in 1..=20 {
            let fact: usize = (1..=n).product();
            let ks = [0, 1, 2, fact / 3, fact / 2, fact - 1];
            for k in ks.into_iter().filter(|&k| k < fact) {
                let expected = unrank(&s, n, k);
                let got = unrank_factoradic(&s, n, &Factoradic::from_rank(k, n));
                assert_eq!(got, expected, "n={n} k={k}");
                assert_eq!(rank(&s, &got), k);
            }
        }
    }

    #[test]
    #[should_panic(expected = "is not below 8!")]
    fn unrank_factoradic_rejects_the_exclusive_end() {
        let s = precompute(20);
        let boundaries = split_factoradic(8, 5);
        unrank_factoradic(&s, 8, boundaries.last().unwrap());
    }

    #[test]
    fn spans_reassemble_heaps_enumeration() {
        // The point of the whole exercise: each job unranks its own start and walks
        // forward, and the jobs concatenated are Heap's algorithm unchanged.
        let n = 8;
        let s = precompute(n - 1);
        let mut data: Vec<u8> = (0..n as u8).collect();
        let expected: Vec<Vec<u8>> = permutohedron::Heap::new(&mut data).collect();

        let mut got = Vec::with_capacity(expected.len());
        for span in split_ranks(n, 13).windows(2) {
            for k in span[0]..span[1] {
                got.push(unrank(&s, n, k).into_vec());
            }
        }

        assert_eq!(got, expected);
    }

    #[test]
    fn unrank_factoradic_serves_n_beyond_usize_ranks() {
        let n = 30;
        let s = precompute(n - 1);
        for k in [0usize, 1, 5039, 1 << 40] {
            let got = unrank_factoradic(&s, n, &Factoradic::from_rank(k, n));
            assert_eq!(got, unrank(&s, n, k), "n={n} k={k}");
        }
    }

    #[test]
    fn successive_ranks_of_a_large_n_differ_by_one_swap() {
        // Past n=20 a boundary is only reachable through its factoradic, so there is no
        // usize unrank() to check it against. Heap's algorithm touches two positions per
        // step, which pins the unranked permutation just as well.
        let n = 25;
        let s = precompute(n - 1);
        let mut k = split_boundary(n, 7, 3);
        let mut previous = unrank_factoradic(&s, n, &k);

        for _ in 0..64 {
            k.add_rank(1);
            let next = unrank_factoradic(&s, n, &k);
            let moved: Vec<usize> = (0..n).filter(|&i| previous[i] != next[i]).collect();
            assert_eq!(moved.len(), 2);
            assert_eq!(previous[moved[0]], next[moved[1]]);
            assert_eq!(previous[moved[1]], next[moved[0]]);
            previous = next;
        }
    }

    #[test]
    fn large_n_spans_are_even() {
        // Spans are shorter than a usize long before their endpoints are, which is what
        // lets a job at this n be handed a start and a count.
        let n = 25;
        let parts = 60_000_000;
        let expected = factorial_div_rem(n, parts).0.to_rank().unwrap();

        let boundaries: Vec<Factoradic> = (0..=4)
            .map(|index| split_boundary(n, parts, index))
            .collect();
        for span in boundaries.windows(2) {
            let mut length = span[1].clone();
            length.sub_assign(&span[0]);
            let length = length.to_rank().expect("span fits in a usize");
            assert!(length == expected || length == expected + 1);
        }
    }
}
