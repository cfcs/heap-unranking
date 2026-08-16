//! Ranking / unranking functions for Heap's algorithm
//! cfcs, 2026
//!
//! unrank(n,k): "skip" k outputs of Heap's algorithm
//! rank(P): calculate "k" (how many iterations of Heap's algorithm it took to produce P)
//
//! Both functions use a precomputed table of final states for prefixes 1 ..= n-1
//! obtainable with precompute(n-1)

//! arr\[ 0 : len(S) \] = arr\[p\] for p in S
//! the "scratch" buffer is used to store the collected elements
//! so we don't have to worry about overwriting entries we need later in the loop

/// Applies the permutation `s` in-place to the prefix of `arr`,
/// composing the `arr` permutation with `s`.
/// The idea is that the permutation pattern `s` is the same, but the `arr`
/// prefix will hold context-specific elements due to suffix values being swapped in
/// from the suffix.
#[inline(always)]
pub fn reset_permutation(scratch: &mut Vec<u8>, s: &[u8], arr: &mut [u8]) {
    // assert!(scratch.capacity() >= s.len());

    // for n <=16 each u8 will be < 16, which means we could use these:
    // https://doc.rust-lang.org/core/arch/x86/fn._mm_shuffle_epi8.html
    // x86_64, ssse3
    // https://doc.rust-lang.org/core/arch/x86/fn._mm_shufflelo_epi16.html
    // https://doc.rust-lang.org/core/arch/x86/fn._mm_shufflehi_epi16.html
    // x86_64, sse2
    // in chunks of 16 bytes
    // we'd have to set the high bits of `s`[..s.len()]
    // _mm_shuffle_epi8(arr, s)

    scratch.clear();
    scratch.extend(s.iter().map(|&p| arr[p as usize]));
    // see kat.rs:precompute_kats:f(), we should probably just use that directly
    // and do away with the precomputation step?
    arr[0..s.len()].copy_from_slice(scratch);
}

///
/// Precompute final states for each prefix of (excluding) `max_n`
///
/// * Runtime: O( 0.5 * n^2 * n + n ) -> O(n^3)
/// * Space:   O( 0.5 * n^2         ) -> O(n^2)
///
pub fn precompute(max_n: usize) -> Vec<Box<[u8]>> {
    let mut s: Vec<Box<[u8]>> = Vec::with_capacity(max_n);
    s.push(Box::new([0; 1])); // trivial, permutations of length 1

    let mut scratch = Vec::with_capacity(max_n - 1);
    for n in 1..max_n {
        // O(n)
        let mut arr: Box<[u8]> = (0..=(n as u8)).collect();
        for j in 0..n {
            // this step happens sum(1 .. max_n-1) times, O(n^2)

            reset_permutation(&mut scratch, &s[n - 1], &mut arr); // O(n)

            arr.swap(j * (n & 1), n); // O(1)
        }

        reset_permutation(&mut scratch, &s[n - 1], &mut arr); // O(n)

        // assert!(s.capacity() > s.len());
        // since we need to look up the result of the previous iteration s[n-1],
        // we can't use rust iterators (??), and therefore we use indexing and push()
        s.push(arr);
    }
    s
}

///
/// Compute the k'th output of Heap's algorithm for a permutation of n elements
///
/// Runtime: Worst case: n+(n-1)n/2 * n -> n + 0.5n^3 -> O(n^3)
///          Best case: n+n -> O(n)
///
pub fn unrank(prefixes: &Vec<Box<[u8]>>, n: usize, mut k: usize) -> Box<[u8]> {
    // Translate k to factoradic digits:
    let qs = (2usize..)
        .take(n - 1)
        .map(|i| {
            let t_q = k % i;
            k /= i;
            t_q
        })
        .collect::<Box<[_]>>();

    let mut scratch: Vec<u8> = Vec::with_capacity(n - 1);

    let mut permutation: Box<[u8]> = (0u8..(n as u8)).collect(); // 0, 1, .., n-1

    // n: from n-1 to 1, step -1  --- (1..permutation.len()) to help the bounds check elision
    // prefix: &prefixes[n-1] at each step
    // q: qs[n-1] at each step
    // O(n)
    for ((n, prefix), q) in (1..permutation.len())
        .zip(prefixes.iter().take(n - 1))
        .zip(qs)
        .rev()
    {
        if n & 1 == 0 {
            for _ in 0..q {
                // O(n)
                reset_permutation(&mut scratch, &prefix, &mut permutation); // O(n)
                permutation.swap(0, n); // O(1)
            }
        } else {
            assert!(q < permutation.len()); // try to get the compiler to elide bounds checks below
            for i in 0..q {
                // O(n)
                reset_permutation(&mut scratch, &prefix, &mut permutation); // O(n)
                permutation.swap(i, n); // O(1)
            }
        }
    }

    permutation
}

///
/// Functional/recursive implementation of factorizing k into factoradic digits
///
fn get_qs(n: usize, i: usize, k: usize, acc: Vec<usize>) -> Vec<usize> {
    assert!(n > 0); // calling get_qs with n=0 is an error
    if n == 1 {
        acc
    } else {
        let acc2 = std::iter::once(k % (i + 1)).chain(acc).collect(); // k%i :: acc
        get_qs(n - 1, i + 1, k / (i + 1), acc2)
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

///
/// Functional/recursive version of [`unrank()`]
///
/// This is pretty inefficient, but is here to serve as an alternative explanation
/// of what is going on, or to assist in porting/proving efforts.
///
pub fn unrank_recursive(n: usize, k: usize) -> Vec<u8> {
    assert!(n > 0); // invalid if k >= factorial(n), which means n==0 /\ k==0 is invalid
    fn reset_permutation_functional<E: std::marker::Copy + Into<usize>>(
        prefixes: &[E],
        arr: Vec<E>,
    ) -> Vec<E> {
        prefixes
            .iter()
            .map(|&p| arr[p.into()])
            .chain(arr[prefixes.len()..].iter().copied()) // remainder left untouched
            .collect()
    }
    fn swap_functional<T: Clone>(arr: Vec<T>, a: usize, b: usize) -> Vec<T> {
        arr.iter()
            .enumerate()
            .map(|(i, val)| {
                if i == a {
                    &arr[b]
                } else if i == b {
                    &arr[a]
                } else {
                    val // neutral case, element remains as-is
                }
            })
            .cloned()
            .collect()
    }
    pub fn precompute_functional(max_n: usize) -> Vec<Vec<u8>> {
        // note that this is like precompute().reverse()
        (0..max_n).fold(vec![], |prefixes, n| {
            let empty = vec![];
            let last_prefix = prefixes.first().unwrap_or(&empty);

            let arr = reset_permutation_functional(
                &last_prefix,
                (0..n).fold((0..=(n as u8)).collect(), |acc, j| {
                    // this step happens sum(1 .. max_n-1) times, O(n^2)
                    swap_functional(
                        reset_permutation_functional(&last_prefix, acc),
                        if n & 1 == 1 { j } else { 0 },
                        n,
                    )
                }),
            );

            // arr :: prefixes
            std::iter::once(arr).chain(prefixes.into_iter()).collect()
        })
    }
    // this is independent of k, and can be precomputed for any n, but if you do
    // so you need to drop the head elements until prefixes.len() == n-1:
    let prefixes = precompute_functional(n - 1);
    // factor k into factoradic digits:
    let qs = get_qs(n, 1, k, vec![]);
    (1..n)
        .rev()
        .zip(qs)
        .zip(prefixes)
        .fold((0u8..(n as u8)).collect(), |acc, ((n, q), pref)| {
            (1..=q).rev().fold(acc, |acc, i| {
                swap_functional(
                    reset_permutation_functional(&pref, acc),
                    if 1 == n & 1 { q - i } else { 0 },
                    n,
                )
            })
        })
}

/// computes the transpositions for each prefix:
/// for n <= 3:
///    produces the indices in reverse order: `[0], [1,0], [2,1,0]`
/// for n >= 4, there are two cases:
///    n & 1 == 0:
///    n & 1 == 1:
/// more details in tests/kat.rs
///
/// # TODO
/// (see `fn f` in tests/kat.rs)
/// So there are a number of cases for `i`.
/// Most of them have `i`-independent RHS.
/// The last case for the "middle" `i`:
///  - `n` is odd: `i`
///  - `n` is even: `i-1`
/// It seems entirely possible to skip these.
#[inline]
pub fn precomp_digit(n: usize, i: usize) -> u8 {
    // assert!(i <= n);
    let nu8 = n as u8;
    match i as u8 {
        // n==2: when n==2 is 0 we want to return 1u8
        0 => nu8 + 2 * (nu8 & 1 | (nu8 <= 2) as u8) - 3,
        1 if n & 1 == 0 => nu8 - 2,
        i if i == nu8 - 1 => 0, // last element is always zero
        i => i + (nu8 & 1) - 1 + 2 * (i == nu8 - 2 && nu8 & 1 == 0) as u8,
    }
}

///
/// The inner body of the loop to advance by a factorial digit `q` in $O(n)$.
/// - `even_tmp` is cleared at entry, and not on exit.
///
/// [`forward_by_q()`] operates solely on the indices of `permutation`, as evident
/// from the type signature which only requires the [`std::marker::Copy`] trait.
///
/// # Examples
///
/// Equivalent to the O(n^2) version:
/// ```
///   # use heap_unranking::{precompute, reset_permutation, forward_by_q};
///   let mut permutation1 = [0,1,2,3,4];
///   let mut permutation2 = permutation1.clone();
///   # let n = 4 ; assert!(n < permutation1.len());
///   let mut scratch = Vec::with_capacity(n-1);
///   # let q = 4; assert!(q <= n);
///   forward_by_q(n, q, &mut scratch, &mut permutation2);
///
///   let prefixes = precompute(n);
///   for i in 0..q {
///     reset_permutation(&mut scratch, &prefixes[n-1], &mut permutation1);
///     permutation1.swap(i * (n & 1), n);
///   }
///
///   assert_eq!(permutation1, permutation2);
/// ```
///
#[inline]
pub fn forward_by_q<E: std::marker::Copy>(
    n: usize,
    q: usize,
    even_tmp: &mut Vec<E>,
    permutation: &mut [E],
) {
    debug_assert!(q <= n);
    debug_assert!(n < permutation.len());
    if n & 1 == 0 {
        if n < 4 {
            // special case for n == 2; 0 < q <= 2, so there are only the odd/even cases:
            permutation.swap(0, 1 + (q & 1));
            permutation.swap(1, 2);
            return;
        }
        // for even n, we can do the q rotations and swaps linearly like this,
        // which is equivalent to:
        // for _ in q { reset_permutation(); permutation.swap(0, n); }:
        even_tmp.clear();
        even_tmp.push(permutation[0]);
        even_tmp.push(permutation[n - 1]);
        even_tmp.push(permutation[n - 2]);
        even_tmp.extend_from_slice(&permutation[1..n - 2]);
        even_tmp.push(permutation[n]);

        permutation[0] = even_tmp[even_tmp.len() - q];
        permutation[n] = even_tmp[n - q];
        permutation[n - 1] = even_tmp[even_tmp.len() + 1 - q - even_tmp.len() * (1 == q) as usize];
        permutation[n - 2] = even_tmp[even_tmp.len() + 2 - q - even_tmp.len() * (q <= 2) as usize];

        let start = even_tmp.len() + 3 - q - even_tmp.len() * (q <= 3) as usize;
        let pivot = (even_tmp.len() - start).min(n - 3);

        permutation[1..=pivot].copy_from_slice(&even_tmp[start..start + pivot]);
        permutation[1 + pivot..n - 2].copy_from_slice(&even_tmp[..n - 3 - pivot]);
    } else {
        assert!(q < permutation.len()); // try to get the compiler to elide bounds checks below
        permutation.swap(0, n - 1);
        permutation.swap(0, n);
        for i in 1..q {
            permutation.swap(i, n);
        }
        if q & 1 == 0 {
            permutation.swap(0, n - 1);
        }
    }
}

///
/// Compute the internal state required for resuming Heap's Algorithm
/// at offset `k`.
/// - `l`: `self.state.len()`
///
pub fn heaps_state_at_k(n: usize, k: usize) -> HeapsAlgorithm<u8> {
    let mut h: HeapsAlgorithm<u8> = HeapsAlgorithm::new((0..(n as u8)).collect::<Vec<u8>>());

    // TODO: should use a generic get_qs(), and one that doesn't need reversing:
    h.counters = get_qs(n + 1, 0, k, vec![]).into_iter().rev().collect(); // get_qs returns them reversed

    // now need to unrank self.state (the permutation), so we copy-paste from
    // unrank_noprecomp_gen():
    /*
     * We look for the first non-zero digit and subtract one.
     * Our goal is to end up with `i` as the parity bit (0 or 1), usually `1`.
     * We also need to fix up the borrowing in the digits preceding the non-zero digit.
     * I'm not sure we can't just resume at the factorization directly, but this way it is
     * easy to test.
     */

    let mut even_tmp: Vec<u8> = Vec::with_capacity(h.state.len() + 1);
    let qs: Vec<usize> = h.counters.iter().skip(1).copied().collect();
    for (n, q) in (1..h.state.len())
        .zip(qs) // TODO we should not clone here
        .rev()
        .filter(|(_, q)| !q.is_zero())
    {
        forward_by_q(n, q, &mut even_tmp, &mut h.state);
    }

    // TODO: This part below needs cleaning up. Seems like we could just
    // use the qs directly?

    h.i = 0; // It may not have yielded yet.
    for idx in 1..h.counters.len() {
        if h.counters[idx] == 0 {
            continue;
        }
        h.i = 1; // Heap's algo sets i=1 every time it yields
        h.counters[idx] -= 1;
        // we know that all of these are 0, because we are only at idx if
        // we skipped idx-... on the way here:
        for idx_borrowing in (1..idx).rev() {
            h.counters[idx_borrowing] += idx_borrowing;
        }
        break;
    }
    if h.i != 0 {
        // Simulate the while-loop in Heap's algo to fixup the counters
        for idx in h.i..h.counters.len() {
            if h.counters[idx] >= idx {
                h.counters[idx] = 0;
                h.i += 1;
                continue;
            }
            break;
        }
        // This swap will be performed by .next(), so we perform it in reverse here:
        if h.i & 1 == 0 {
            h.state.swap(0, h.i);
        } else {
            h.state.swap(h.counters[h.i], h.i);
        }
    }

    h
}

///
/// Like [`unrank()`], but without the precomputation table.
///
/// Runtime: O(n^2)
///
/// # Examples
///
/// ```
/// # use heap_unranking::unrank_noprecomp;
/// assert_eq!(unrank_noprecomp(3, 5), [2, 1, 0].into());
/// ```
pub fn unrank_noprecomp(n: usize, k: usize) -> Box<[u8]> {
    unrank_noprecomp_gen(0..(n as u8), k)
}

pub fn unrank_noprecomp64(n: usize, k: u64) -> Box<[u8]> {
    unrank_noprecomp_gen(0..(n as u8), k)
}

use num_traits::Zero;
///
/// Unrank a permutation of length `n` given the "identity" indices `(0..n)`
///
pub fn unrank_noprecomp_gen<R, E, K>(identity: R, mut k: K) -> Box<[E]>
where
    R: IntoIterator<Item = E>,
    E: std::marker::Copy + std::fmt::Debug,
    K: for<'a> std::ops::Rem<&'a K, Output = K>
        + for<'a> std::ops::DivAssign<&'a K>
        + TryInto<usize>
        + std::ops::AddAssign<K>
        + num_traits::Zero
        + num_traits::One
        + Clone
        + std::fmt::Debug,
    <K as TryInto<usize>>::Error: std::fmt::Debug,
{
    let mut permutation: Box<[E]> = identity.into_iter().collect();

    // Translate k to factoradic digits:
    let mut i_k = K::one();
    let qs: Box<[usize]> = (0..permutation.len() - 1)
        .map(|_| {
            k /= &i_k;
            i_k += K::one();
            let t_q = k.clone() % &i_k;
            <K as TryInto<usize>>::try_into(t_q).unwrap()
        })
        .collect();

    let mut even_tmp: Vec<E> = Vec::with_capacity(permutation.len() + 1);

    // n: from n-1 to 1, step -1  --- (1..permutation.len()) to help the bounds check elision
    // q: qs[n-1] at each step
    // O(n * 0.5 n) -> O(0.5n^2), the 0.5 comes from the inner n's being sum(1, 2, .. n)
    for (n, q) in (1..permutation.len())
        .zip(qs)
        .rev()
        .filter(|(_, q)| !q.is_zero())
    {
        forward_by_q(n, q, &mut even_tmp, &mut permutation); // O(n)
    }

    permutation
}

///
/// rank([precompute()], `P`) computes result `k` such that [unrank]([precompute()], `k`) == `P`
///
/// It computes the number of iterations of Heap's algorithm needed to reach `P`.
///
/// Runtime:
/// - Worst case: (n-1)n/2 * n + n -> 0.5n^3 + n -> O(n^3)
/// - Best case: n+n -> O(n)
///
pub fn rank(prefixes: &Vec<Box<[u8]>>, permutation: Box<[u8]>) -> usize {
    if permutation.len() <= 1 {
        return 0;
    }
    let mut arr: Box<[u8]> = (0u8..(permutation.len() as u8)).collect();
    let mut scratch = Vec::with_capacity(permutation.len() - 1);
    let mut qs = vec![0; permutation.len() - 1].into_boxed_slice();

    for (qq, (prefix, (i, &permutation_i))) in qs
        .iter_mut()
        .zip(prefixes.iter().zip(permutation.iter().enumerate().skip(1)))
        .rev()
    {
        // O(n)
        let mut q = 0;
        while arr[i] != permutation_i
        /* arr[i] != permutation[i] */
        {
            // O(n)
            reset_permutation(&mut scratch, prefix /* &s[i-1] */, &mut arr); // O(n)
            arr.swap((i & 1) * q, i); // O(1)
            q += 1;
        }
        *qq = q; // qs[i-1] = q;
    }

    // O(n)
    let mut k: usize = qs[0];
    let mut fact_i = 1;
    for (i, q) in qs.iter().enumerate().skip(1) {
        // TODO this can overflow if factorial(permutation.len()) > usize::MAX
        fact_i *= i + 1;
        k += q * fact_i; // k is < factorial(permutation.len())
    }

    k
}

///
/// Rank a permutation
/// Runtime: O(n^2), amortized with word operations
///
pub fn rank_noprecomp(permutation: &[u8]) -> usize {
    if permutation.len() <= 1 {
        return 0;
    }
    let mut arr: Box<[u8]> = (0u8..(permutation.len() as u8)).collect();
    let mut qs = vec![0; permutation.len() - 1].into_boxed_slice();
    let mut even_tmp: Vec<u8> = Vec::with_capacity(permutation.len() + 1); // TODO
    for (qq, (i, &permutation_i)) in qs
        .iter_mut()
        .zip(permutation.iter().enumerate().skip(1))
        .rev()
    {
        // O(n)

        // fast-track heuristics:
        if arr[i] == permutation_i {
            // no swaps required to make 0..=i have a suffix of permutation[i]
            continue; // q:=0; O(1) -> O((1/n)0.5n)
        }
        if arr[0] == permutation_i {
            *qq = i; // q:=i; when arr[0] == permutation[i]
            forward_by_q(i, i, &mut even_tmp, &mut arr); // O(n) -> O((1/n)0.5n^2)
            continue;
        }
        if i == 2 && arr[1] == permutation_i {
            *qq = 1; // special case for len 3 not covered by the two rules above
            arr.swap(1, i);
            arr.swap(0, 1);
            continue;
        }

        if true {
            // these are interesting observations that seem to hold, but I have not
            // managed to pin down the argument for why they work. It feels like
            // there is a rule that would let us generalize these to odd/even cases.
            let idx = arr.iter().position(|&x| x == permutation_i).unwrap();
            assert!(idx > 0); // idx can't be 0 (handled above)
            assert!(idx < i); // // idx can't be i (handled above)
            if true && i == 3 {
                *qq = i - idx; // { idx != i -> 3-idx != 0}
                assert_ne!(*qq, 0);
                forward_by_q(i, *qq, &mut even_tmp, &mut arr);
                continue;
            }
            if true && i == 4 {
                *qq = idx;
                forward_by_q(i, *qq, &mut even_tmp, &mut arr);
                continue;
            }
            // Above we have handled i=[1,2,3,4] and 2/n of the other cases.
        }

        if (i & 1) == 1 {
            // unrolled version of forward_by_q(i, 1, &mut even_tmp, &mut arr);
            // for the odd case we only do a constant number of swaps, making
            // the the total complexity of the inner loop (0.5n * 0.5n) -> O(\frac{1}{4}n^2)
            arr.swap(0, i - 1);
            arr.swap(0, i);
            for q in 1..i {
                // from 1 to i-1
                if permutation_i == arr[i] {
                    *qq = q;
                    break;
                }
                // The four swaps we are left with cancel out to two:
                // arr.swap(0, i - 1);
                // arr.swap(0, i); arr.swap(i, q); arr.swap(0, q);
                // -> arr.swap(i, q); arr.swap(0,q); arr.swap(0,q); -> arr.swap(i, q)
                arr.swap(0, i - 1);
                arr.swap(i, q);
            }
            debug_assert_ne!(*qq, 0); // we should always be able to find it
            continue;
        } else {
            // we have now established these pre-conditions:
            //   { q > 0 } (that is, it will be)
            //   { i >= 4 }
            //   { i & 1 == 0 -> i is even }
            //   { arr[i] != permutation[i] }
            //   { arr[0] != permutation[i] }

            let mut q = 0;
            let mut bmap = 0u32;
            for (xi, &xv) in arr.iter().enumerate() {
                bmap |= ((xv == permutation_i) as u32) << xi;
            }
            // It is worth noting that below the only operation we perform that involves
            // the elements of even_tmp / permutation_i is comparing whether or not a given element
            // is equal to permutation_i or not, so I used a bitmap below to amortize the O(n^2)
            // looping over forward_by_q() for the even `i`s, arriving at this
            // [amortized] O(n) solution. First we use the bitmap version of forward_by_q, tracking
            // only the permutation[i], and we use that to find `q`.
            // O(0.5n * 0.5 n * ceil(n/wordsize))
            for _ in 1..i {
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
                    bmap &= !(1 << (i - 1));
                    // Since assert_eq!(bmap & (1<<i), 0) doesn't change, we simulate continue; and
                    // proceed:
                    q += 2;
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
                                                      //CAN continue; if rhs!=0

                bmap |= (emap & (1 << i)) >> i; // set bmap[0] if even_tmp[i] == permutation_i
                                                // CAN: continue; if rhs != 0

                let esuffix = (emap >> 2) & ((1 << (i - 3)) - 1);
                bmap |= (((!bmap) >> 1) & esuffix) << 1; // secondcopy_from_slice(even_tmp, ..n)
            }
            // Having established `q` we can now advance the prefix permutation by `q` in O(n)
            // in order to prepare &arr for the next loop iteration:
            forward_by_q(i, q, &mut even_tmp, &mut arr);
            *qq = q;
            continue;
        }
        /* Alternatively (for both even and off cases):
           while permutation_i != arr[i] {
             forward_by_q(i, 1, &mut even_tmp, &mut arr); // O(n)
            if (i & 1) * q > 0 { arr.swap(i, q); arr.swap(0, q); }
           }
        */
        /* Alternatively:
           // precomped_digits.extend((0..i).map(|d| precomp_digit(i, d)));
               scratch.clear();
               scratch.extend(precomped_digits.iter().map(|&d| arr[d as usize]));
               arr[0..i].copy_from_slice(&scratch); // O(n)
               arr.swap((i & 1) *q, i);
               q += 1
        */
    }

    // O(n)
    let mut k: usize = qs[0];
    let mut fact_i = 1;
    for (i, q) in qs.iter().enumerate().skip(1) {
        // TODO this can overflow if factorial(permutation.len()) > usize::MAX
        fact_i *= i + 1;
        k += q * fact_i; // k is < factorial(permutation.len())
    }

    k
}

use num_bigint::BigUint;

pub fn unrank_bigint(n: usize, k: BigUint) -> Box<[usize]> {
    unrank_noprecomp_gen(0..n, k)
}

#[test]
fn bigint_30() {
    let perm = unrank_bigint(2, BigUint::new_const(2));
    assert_eq!(&perm[..], [0, 1]);
}

///
/// Classic implementation of Heap's algorithm for iterating over permutations.
///
pub struct HeapsAlgorithm<E>
where
    E: Clone,
{
    state: Box<[E]>,
    counters: Box<[usize]>,
    i: usize,
    k: usize, // used exclusively for debugging
}

impl<E: Clone> HeapsAlgorithm<E> {
    ///
    /// Given an "alphabet", the [`Iterator`] yields the permutations of the alphabet.
    ///
    /// # Examples:
    /// ```
    /// # use heap_unranking::HeapsAlgorithm;
    /// for (k, p) in HeapsAlgorithm::new::<Vec<&str>>(vec!["a","b","c","d"]).enumerate() {
    ///     println!("{k}: {:?}", p);
    ///     match k {
    ///        12 => { assert_eq!(p, vec!["a", "c", "d", "b"].into()); }
    ///        23 => { assert_eq!(p, vec!["b", "c", "d", "a"].into()); }
    ///        _ => { /* todo!() */ }
    ///     }
    /// }
    /// ```
    pub fn new<R>(initial: R) -> HeapsAlgorithm<E>
    where
        R: IntoIterator<Item = E>,
        Box<[E]>: From<R> + Clone,
    {
        let state: Box<[E]> = initial.into();
        let n = state.len();
        HeapsAlgorithm {
            state: state,
            counters: (0..n).map(|_| 0).collect(),
            i: 0, // sentinel for starts
            k: 0,
        }
    }
}
use std::num::NonZero;
impl<E: Clone> Iterator for HeapsAlgorithm<E> {
    // we will be counting with usize
    type Item = Box<[E]>;

    //fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
    //	Ok()
    //}

    ///
    /// Heap's algorithm: `counters` and `i`:
    /// - `counters` is the factoradic digits encoding rank
    /// - `counters[0]` is always zero, D*0!
    ///   - that is of course a waste, but it saves us having to do arithmetic in the comparison
    ///     (i==2 addresses D*2!
    ///   - it also helps with self.state.swap(self.counters[self.i], self.i); not counter-acting
    ///     self.state.swap(0, self.i)
    /// - `i` is the current "remainder"
    ///
    ///
    /*
    k=46 i=2 [0, 0, 1, 3, 1]
    t_q[0]: 0
    t_q[1]: 2
    t_q[2]: 3
    t_q[3]: 1
    k=47 i=1 [0, 0, 2, 3, 1]
    t_q[0]: 1
    t_q[1]: 2
    t_q[2]: 3
    t_q[3]: 1
    k=48 i=4 [0, 0, 0, 0, 1] <- why is this i=0?
    t_q[0]: 0     0*1!
    t_q[1]: 0     0*2!
    t_q[2]: 0     0*3!
    t_q[3]: 2     2*4! <-- 48

    */
    // next() is the only required method
    fn next(&mut self) -> Option<Box<[E]>> {
        if self.i == 0 {
            self.i = 1;
            self.k += 1;
            return Some(self.state.clone());
        }
        while self.i < self.state.len() {
            if self.counters[self.i] < self.i {
                if self.i & 1 == 0
                /* self.i.is_even() */
                {
                    self.state.swap(0, self.i)
                } else {
                    self.state.swap(self.counters[self.i], self.i);
                }
                self.counters[self.i] += 1;
                self.i = 1;
                self.k += 1;
                return Some(self.state.clone());
            } else {
                self.counters[self.i] = 0;
                self.i += 1;
            }
        }
        None
    }

    ///
    /// `nth()` is very similar to heaps_state_at_k(n, k)
    /// except it's a relative jump from the current `k`.
    /// It ought to be equal to `heaps_state_at_k(n, k-self.k)`,
    /// but it would be nice to not rely on maintaining a self.k
    /// so we can avoid dealing with overflows.
    ///
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        /*
        say we're have output k=19 and we need to skip 1:
        k=19 i=1 [0, 0, 0, 3]
        t_q[0]: 1    x-------- 0 < 1, so: bump counter, i := 1
        t_q[1]: 0
        t_q[2]: 3
                     x-------- 1 < 1 is false, so set it to zero and bump i:=2
                 [0, 1, 0, 3]
        k=20 i=2 [0, 0, 0, 3]
        t_q[0]: 0       x------- 0 < 2, so: bump counter, i := 1
        t_q[1]: 1
        t_q[2]: 3    x---------- 0 < 2, so: bump counter, i := 1
                 [0, 1, 1, 3]
        k=21 i=1 [0, 0, 1, 3]
        t_q[0]: 1
        t_q[1]: 1
        t_q[2]: 3
         */
        // so we have a bunch of options:
        // - run the loop (slow)
        // - sum the factoradic digits
        //   - overflow sucks.
        //     - we can double the base of each, capping at 2i?
        //       - we don't need to compute factorials
        //       - can manage carry cheaply in O(n)
        // - decode the current factoradic to an int, add toskip, decode

        let mut toskip = n;
        // forward_by_q(self.state, .. );
        while toskip > 0 {
            if self.counters[self.i] < self.i {
                self.counters[self.i] += 1;
                self.i = 1;
                self.k += 1;
                toskip -= 1;
            } else {
                // fix up factoradic digit overflow
                //
                self.counters[self.i] = 0;
                self.i += 1;
            }
        }
        panic!("NOT IMPLEMENTED");
        // loop from 1..
        None
    }
}
