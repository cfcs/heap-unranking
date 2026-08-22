//! Ranking / unranking functions for Heap's algorithm
//! by `@cfcs`, 2026
//!
//! - [`precompute::unrank`]`(n,k)`: "skip" k outputs of Heap's algorithm
//! - [`precompute::rank`]`(P)`: calculate "k" (how many iterations of Heap's algorithm it took to produce P)
//! - [`HeapsAlgorithm::at_k()`]: Heap's algorithm starting at a given rank `k`
//! - [`HeapsAlgorithm::step()`]: Heap's algorithm, step-by-step
//! - [`HeapsAlgorithm::previous()`]: Heap's algorithm in reverse, step-by-step
//!
pub mod precompute;

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
/// Functional/recursive version of [`precompute::unrank()`]
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
                last_prefix,
                (0..n).fold((0..=(n as u8)).collect(), |acc, j| {
                    // this step happens sum(1 .. max_n-1) times, O(n^2)
                    swap_functional(
                        reset_permutation_functional(last_prefix, acc),
                        if n & 1 == 1 { j } else { 0 },
                        n,
                    )
                }),
            );

            // arr :: prefixes
            std::iter::once(arr).chain(prefixes).collect()
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
///   # use heap_unranking::forward_by_q;
///   use heap_unranking::precompute::{precompute, reset_permutation};
///   let mut permutation1 = [0,1,2,3,4];
///   let mut permutation2 = permutation1.clone();
///   # let n = 4 ; assert!(n < permutation1.len());
///   let mut scratch = Vec::with_capacity(n-1);
///   # let q = 4; assert!(q <= n);
///   forward_by_q(n, q, &mut scratch, &mut permutation2);
///
///   // NB: see rank_noprecomp_gen() where it computes `idx` and `*qq`:
///   // if n - idx - 3 { q == n - idx - 3} else { q == 1 + idx }
///   assert_eq!(permutation2[n-3 /* == idx */ ], 4);
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

use num_traits::ConstOne;
use num_traits::Zero;

pub fn rank_noprecomp_gen<R, E, K>(identity: R, permutation: &[E]) -> K
where
    R: IntoIterator<Item = E>,
    E: std::marker::Copy + std::cmp::PartialEq,
    K: for<'a> std::ops::MulAssign<&'a K>
        + std::ops::AddAssign<usize>
        + std::ops::AddAssign<K>
        + std::convert::From<usize>
        + num_traits::Zero
        + ConstOne,
{
    if permutation.len() <= 1 {
        return K::zero();
    }
    let mut arr: Box<[E]> = identity.into_iter().collect();
    let mut qs: Box<[usize]> = vec![0; permutation.len() - 1].into_boxed_slice();
    let mut even_tmp: Vec<E> = Vec::with_capacity(permutation.len() + 1); // TODO
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

        //   { arr[i] != permutation[i] }
        //   { arr[0] != permutation[i] }

        if (i & 1) == 1 {
            // unrolled version of forward_by_q(i, 1, &mut even_tmp, &mut arr);
            arr.swap(0, i - 1);
            arr.swap(0, i); // we have already established arr[0] != permutation_i
            *qq = i - 1; // if we have checked 1..=i-2 => q MUST be i-1 because it's not arr[i]
            for q in 1..=i - 2 {
                if permutation_i == arr[i] {
                    *qq = q;
                    break;
                }
                arr.swap(i, q);
            }
            if *qq & 1 == 0 {
                arr.swap(0, i - 1);
            }
            continue;
        }

        // we have now established these pre-conditions:
        //   { q > 0 } (that is, it will be because arr[i] != permutation_i)
        //   { i >= 4 } \/ { i == 2 && arr[1] == permutation_i }
        //   { i & 1 == 0 -> i is even }

        let idx = arr
            .iter()
            .skip(1)
            .take(i - 1)
            .position(|&e| e == permutation_i)
            .unwrap();
        *qq = if i - idx > 3 { i - idx - 3 } else { 1 + idx };

        // Having established `q` we can now advance the prefix permutation by `q` in O(n)
        // in order to prepare &arr for the next loop iteration:
        forward_by_q(i, *qq, &mut even_tmp, &mut arr);

        /* Alternatively (for both even and odd cases):
           while permutation_i != arr[i] {
             forward_by_q(i, 1, &mut even_tmp, &mut arr); // O(n)
            if (i & 1) * q > 0 { arr.swap(i, q); arr.swap(0, q); }
           }
        */
    }

    let mut k: K = qs[0].into();
    let mut fact_i = K::ONE;
    for (i, q) in qs.iter().enumerate().skip(1) {
        // TODO this can overflow if factorial(permutation.len()) > usize::MAX
        let mut tmp = K::from(i + 1);
        fact_i *= &tmp; // fact_i *= i + 1
        tmp.set_zero();
        tmp.add_assign(*q);
        tmp *= &fact_i;
        k += tmp; // k += q * fact_i;  k is < factorial(permutation.len())
    }

    k
}

///
/// Like [`precompute::unrank()`], but without the precomputation table.
///
/// Runtime: $O(n^2)$
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

///
/// Unrank a the permutation indices at rank `k` for an array of length `n`
/// given the "identity" indices `(0..n)`.
///
/// # Examples
/// ```rust
/// # use heap_unranking::unrank_noprecomp_gen;
/// fn unrank_noprecomp64(n: usize, k: u64) -> Box<[u8]> {
///    unrank_noprecomp_gen(0..(n as u8), k)
/// }
/// assert_eq!([0, 2, 1, 3], unrank_noprecomp64(4, 3)[..]);
/// ```
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
/// Rank a permutation, returning `k` such that [`unrank(permutation.len(), k)`][unrank_noprecomp] `== permutation`
/// Runtime: $O(n \frac{1}{2} n)$
///
/// # Examples
pub fn rank_noprecomp(permutation: &[u8]) -> usize {
    rank_noprecomp_gen(0..permutation.len() as u8, permutation)
}

use num_bigint::BigUint;

pub fn rank_bigint(permutation: &[usize]) -> BigUint {
    rank_noprecomp_gen(0..permutation.len(), permutation)
}

///
/// Compute the `k`'th permutation output by Heap's algorithm for arrays of size `n`.
///
/// # Examples
/// ```
/// # use heap_unranking::unrank_bigint;
/// use num_bigint::BigUint;
/// let p = unrank_bigint(21, BigUint::from(8_526_381_368_646_914_543u64));
/// assert_eq!(p[..], [0, 2, 4, 16, 1, 7, 11, 5, 18, 3, 13, 10, 14, 20, 9, 17, 19, 8, 12, 6, 15]);
/// ```
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
/// [`HeapsAlgorithm::previous()`] and [`HeapsAlgorithm::at_k`] are the interesting ones.
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

impl<E: Clone + std::marker::Copy + std::fmt::Debug> HeapsAlgorithm<E> {
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
    ///
    /// ```rust
    /// # use heap_unranking::HeapsAlgorithm;
    /// assert_eq!(
    ///     (1..=4).product::<usize>(), // 24
    ///     HeapsAlgorithm::new(vec![10,20,30,40]).count(),
    ///     "Yields factorial(n) outputs"
    /// );
    /// ```
    pub fn new<R>(initial: R) -> HeapsAlgorithm<E>
    where
        R: IntoIterator<Item = E>,
        Box<[E]>: From<R> + Clone,
    {
        let state: Box<[E]> = initial.into();
        let n = state.len();
        HeapsAlgorithm {
            state,
            counters: (0..n).map(|_| 0).collect(),
            i: 0, // sentinel for starts
            k: 0,
        }
    }

    ///
    /// Compute the internal state required for resuming Heap's Algorithm
    /// at offset `k` in $O(n^2)$ where `n = identity.len()`.
    /// - `state`: equivalent to [`unrank_noprecomp_gen`]`(identity, k)`
    /// - `counters`: factoradic rank, except `i` represents a carry of `1` in `counters[i]`
    ///
    pub fn at_k<R, K>(identity: R, mut k: K) -> HeapsAlgorithm<E>
    where
        R: IntoIterator<Item = E> + Clone,
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
            .zip(qs.iter())
            .rev()
            .filter(|(_, q)| !q.is_zero())
        {
            forward_by_q(n, *q, &mut even_tmp, &mut permutation); // O(n)
        }

        // The above is essentially unrank_noprecomp_gen(), but
        // we can't use thta because we need to store the qs in a form that
        // Heap's algorithm can resume from:

        let mut counters = std::iter::once(0).chain(qs).collect();
        let i = fixup_factorial(&mut counters);
        if i != 0 {
            // This swap will be performed by .next(), so we perform it in reverse here:
            if i & 1 == 0 {
                permutation.swap(0, i);
            } else {
                permutation.swap(counters[i], i);
            }
        }

        HeapsAlgorithm {
            state: permutation,
            counters,
            i,
            k: 0, // TODO remove this field
        }
    }

    ///
    /// In this implementation it holds the "actual" implementation of Heap's algorithm
    /// since it is very similar to `next()`, but the predicate can operate on an immutable pointer,
    /// so we can implement it more efficiently than `next()` which always clones.
    ///
    /// # Details:
    /// `counters` and `i`:
    /// - `counters` is the factoradic digits encoding rank
    /// - `counters[0]` is always zero, D*0!
    ///   - that is of course a waste, but it saves us having to do arithmetic in the comparison
    ///     (i==2 addresses D*2!
    ///   - it also helps with self.state.swap(self.counters[self.i], self.i); not counter-acting
    ///     self.state.swap(0, self.i)
    /// - `i` is the current "remainder". Note that it is always reset to `1` when we yield.
    ///
    #[inline]
    pub fn step<P>(&mut self, mut predicate: P) -> Option<&Box<[E]>>
    where
        Self: Sized,
        P: FnMut(&Box<[E]>) -> bool,
    {
        if self.i == 0 {
            self.i = 1;
            self.k += 1;
            if predicate(&self.state) {
                return Some(&self.state);
            }
        }
        while self.i < self.state.len() {
            if self.counters[self.i] < self.i {
                if self.i & 1 == 0 {
                    self.state.swap(0, self.i)
                } else {
                    self.state.swap(self.counters[self.i], self.i);
                }
                self.counters[self.i] += 1;
                self.i = 1;
                self.k += 1;
                if predicate(&self.state) {
                    return Some(&self.state);
                }
            } else {
                self.counters[self.i] = 0;
                self.i += 1;
            }
        }
        None
    }

    ///
    /// Heap's algorithm in reverse, inverse of [`HeapsAlgorithm::next()`].
    ///
    /// # Examples
    /// ## Literal examples
    /// ```rust
    /// # use heap_unranking::HeapsAlgorithm;
    /// let mut heap1 : HeapsAlgorithm<usize> = HeapsAlgorithm::new((0..4).collect::<Vec<_>>());
    ///
    /// assert_eq!(None, heap1.previous());
    ///
    /// assert_eq!([0, 1, 2, 3], heap1.next().unwrap()[..]);
    /// assert_eq!([1, 0, 2, 3], heap1.next().unwrap()[..]);
    ///
    /// assert_eq!([2, 0, 1, 3], heap1.next().unwrap()[..]);
    ///
    /// assert_eq!([1, 0, 2, 3], heap1.previous().unwrap()[..]);
    /// assert_eq!([0, 1, 2, 3], heap1.previous().unwrap()[..]);
    ///
    /// assert_eq!(None, heap1.previous());
    /// ```
    /// ## Equivalence to [`HeapsAlgorithm::at_k`]`(identity, k -1).next()`
    ///
    /// ```rust
    /// # use heap_unranking::{HeapsAlgorithm, unrank_noprecomp};
    /// let k: usize = 5000;
    /// let elements = (0..10).collect::<Box<[u8]>>();
    /// let mut heap1: HeapsAlgorithm<u8> = HeapsAlgorithm::new(elements.clone());
    /// for _ in 0 ..= k {
    ///   heap1.next(); // rank=0 through rank=k
    /// }
    /// let k_minus_1 = heap1.previous().unwrap();
    /// assert_eq!(*k_minus_1, HeapsAlgorithm::at_k(elements, k-1).next().unwrap());
    /// ```
    ///
    /// ## Reverse up to 200 steps:
    /// ```rust
    /// # use heap_unranking::HeapsAlgorithm;
    /// for n in 1..100 {
    ///   let steps = if n >= 6 { 200 } else { (1..=n).product::<usize>() - 1 };
    ///   let mut heap2 : HeapsAlgorithm<usize> = HeapsAlgorithm::new((0..n).collect::<Vec<_>>());
    ///   let first_x: Vec<_> = (0..steps).map(|_| heap2.next()).collect();
    ///   heap2.next(); // advance one (the midpoint)
    ///   for (k, old) in first_x.iter().enumerate().rev() {
    ///     assert_eq!(old.as_ref(), heap2.previous(), "k={k}");
    ///   }
    /// }
    /// ```
    pub fn previous(&mut self) -> Option<&<HeapsAlgorithm<E> as Iterator>::Item> {
        // We need to reconstruct self.i from the previous iteration,
        // and subtract one from the factorial representation.
        // Amortized O(1), worst case O(n)
        let mut borrow = 1_isize;
        let mut last_borrow = 0;
        for (i, c) in self.counters.iter_mut().enumerate().skip(1) {
            let s = (*c as isize) - borrow;
            last_borrow = i;
            if s < 0 {
                borrow = 1;
                *c = i;
                continue;
            }
            borrow = 0;
            *c = s as usize;
            break;
        }

        // If it underflows, it will start generating the Heap's algorithm sequence anew.
        // Our subtraction will have modified self.counters, and self.i was 1, so we reset the state:
        if borrow != 0 {
            self.counters.fill(0);
            self.i = 0;
            return None;
        }

        // Undo the previous next()'s swap:
        if last_borrow & 1 == 0 {
            self.state.swap(0, last_borrow);
        } else {
            self.state.swap(self.counters[last_borrow], last_borrow);
        }

        Some(&self.state)
    }

    ///
    /// This doesn't work yet, but the idea is to skip `k` steps ahead.
    ///
    pub fn nth_optimized(&mut self, k: usize) -> Option<<HeapsAlgorithm<E> as Iterator>::Item> {
        // We ought to be able to use forward_by_q() to transform the self.state,
        // and to manipulate the self.counters by encoding `k` as factoradic digits,
        // to figure out how many steps of forward_by_q().
        // We may have to .next() or .previous() some steps to achieve block alignment.

        let mut o: HeapsAlgorithm<usize> =
            HeapsAlgorithm::new((0..self.state.len()).collect::<Box<[usize]>>());
        {
            o.counters = get_qs(self.state.len() + 1, 0, k, vec![])
                .into_iter()
                .rev()
                .collect();
            let mut even_tmp: Vec<E> = Vec::with_capacity(o.state.len() + 1);
            let mut bigger: &Box<[usize]>;
            let mut smaller: &Box<[usize]>;
            bigger = &self.counters;
            smaller = &o.counters;
            let mut sum: Box<[isize]> = (0..self.counters.len()).map(|_| 0).collect();
            for (_i, (a, b)) in self
                .counters
                .iter()
                .zip(o.counters.iter())
                .enumerate()
                .rev()
            {
                if a == b {
                    continue;
                }
                if a > b {
                    bigger = &self.counters;
                    if self.i != 0 {
                        sum[self.i] = 1;
                    }
                    smaller = &o.counters;
                    println!("self > o");
                    break;
                }
                if b > a {
                    smaller = &self.counters;
                    if self.i != 0 {
                        sum[self.i] -= 1;
                    }
                    bigger = &o.counters;
                    println!("o > self so sum[{:?}] -= 1", self.i);
                    break;
                }
            }
            println!("big: {:?}", bigger);
            println!("low: {:?}", smaller);
            // factorial sub:
            let mut borrow = 0_isize;
            for i in 1..self.counters.len() {
                println!(
                    "  sum[{i}] (was {:?}) += {:?} - {borrow} - {:?}",
                    sum[i], bigger[i], smaller[i]
                );
                sum[i] += (bigger[i] as isize) - borrow - (smaller[i] as isize);
                borrow = 0;
                if sum[i] < 0 {
                    borrow = (0 - sum[i]) / (i as isize);
                    assert_ne!(0, borrow);
                    println!("  [{i}]borrow: {:?}   sum[{i}]=={:?}", borrow, sum[i]);
                    sum[i] += borrow * (i as isize + 1);
                    if sum[i] > i as isize {
                        println!("we borrowed too much: {:?} > {i}", sum[i]);
                    }
                }
            }
            println!("sum: {:?}", sum);
            let mut u_sum: Box<[usize]> =
                (0..self.counters.len()).map(|i| sum[i] as usize).collect();
            for (i, x) in o.counters.iter_mut().enumerate() {
                *x = sum[i] as usize;
            }
            let qs: Vec<usize> = u_sum.iter().skip(1).copied().collect();
            // Transform the permutation state by `n` steps:
            for (n, q) in (1..o.state.len())
                .zip(qs) // TODO we should not clone here
                .rev()
                .filter(|(_, q)| !q.is_zero())
            {
                forward_by_q::<E>(n, q, &mut even_tmp, &mut self.state[..]);
                println!("  fwd [{n}] by q={q}: {:?}", self.state);
            }
            o.i = fixup_factorial(&mut o.counters);
            if self.i != 0 {
                println!("WE HAVE A SELF i:{:?} and we would like to swap!", self.i);
                if self.i & 1 == 0 {
                    println!("  swap(0, {:?});", self.i);
                } else {
                    println!("  swap(X, {:?}); //ctrs: {:?}", self.i, self.counters);
                    // this ... seems to work for some cases:
                    //self.state.swap(0,2);
                    //self.state.swap(0,3);
                    //self.state.swap(3, 2);
                }
            }
            let sumi = fixup_factorial(&mut u_sum);
            println!("  SUMS: self:{:?} o:{:?} sumi:{sumi}", self.i, o.i);
            if u_sum.len() <= 4 && sumi != 0 {
                println!("    sumi:{sumi}");
                // This swap will be performed by .next(), so we perform it in reverse here:
                if sumi & 1 == 0 {
                    self.state.swap(0, sumi);
                    println!(
                        "    heaps_state_at_k: even branch: o.i={:?}: counters={:?} state={:?}",
                        o.i, u_sum, self.state
                    );
                } else {
                    println!("....odd before: {:?}", self.state);
                    self.state.swap(u_sum[sumi], sumi);
                    println!(
                        "    heaps_state_at_k: odd branch: o.i={:?}: counters={:?} state={:?} self.counters={:?} o.counters={:?}",
                        o.i, u_sum, self.state, self.counters, o.counters
                    );
                }
            } else {
                println!("  sumi zero");
            }
            //println!("o.state: {:?}", o.state);
            //let mut scratch: Vec<E> = Vec::with_capacity(o.state.len() + 1);
            //scratch.clear();
            //scratch.extend(o.state.iter().map(|&p| self.state[p]));
            // see kat.rs:precompute_kats:f(), we should probably just use that directly
            // and do away with the precomputation step?
            //self.state[0..scratch.len()].copy_from_slice(&scratch[..]);
            //self.state.copy_from_slice(&o.state);
        }

        //println!("self.counters: {:?}", &self.counters);
        // now we need to add together the factoradic digits of
        // o and self, and restore the `i`:
        self.counters = sum_factorial(self.i, &self.counters, o.i, &o.counters);
        println!("  sum_factorial before fixup: {:?}", self.counters);
        //println!("self.i was {:?} o.i=={:?}", self.i, o.i);
        //println!("self.counters: before fixup {:?}", &self.counters);
        self.i = fixup_factorial(&mut self.counters);
        println!(
            "  SUM counters after fixup: {:?} i=={:?}",
            self.counters, self.i
        );
        self.k += k;
        println!(
            "nth alen:{:?} k={k} self.i:={:?} o.i=={:?} self.counters:{:?} self.state:{:?}",
            self.state.len(),
            self.i,
            o.i,
            self.counters,
            self.state
        );

        self.next()
    }
}

///
/// Sum two arrays of factoradic digits.
/// - The `i1` and `i2` arguments are the optional `HeapPermutation.i` remainders.
/// - The arrays must be 0-prefixed for (d % 1)*0! (so for n=3 the length must be 4)
///
/// We'll still need to:
/// - reconstruct `self.i` (see [`fixup_factorial()`])
/// - [`forward_by_q()`] the `self.state`, see [`heaps_state_at_k()`]
///
#[inline]
fn sum_factorial(i1: usize, qs1: &Box<[usize]>, i2: usize, qs2: &Box<[usize]>) -> Box<[usize]> {
    assert_eq!(qs1.len(), qs2.len());
    let mut carry = 0;
    let mut sum: Box<[usize]> = vec![0; qs1.len()].into_boxed_slice();
    sum[i1] += (i1 != 0) as usize;
    sum[i2] += (i2 != 0) as usize;
    for ((i, sum_i), (q1, q2)) in sum.iter_mut().enumerate().zip(qs1.iter().zip(qs2)).skip(1) {
        *sum_i += q1 + q2 + carry;
        carry = 0;
        if *sum_i > i {
            carry = *sum_i / (i + 1);
            *sum_i %= i + 1;
        }
        debug_assert!(*sum_i <= i);
    }
    debug_assert_eq!(sum[0], 0, "the prefix digit should be zero");
    sum
}

#[test]
fn sum_factorial_kat1() {
    // 0 + (1+2) == (1+2)
    let mut s = sum_factorial(0, &[0, 0, 0, 0].into(), 1, &[0, 0, 1, 0].into());
    assert_eq!(&s[..], [0, 1, 1, 0], "plain sum");
    let i = fixup_factorial(&mut s); // split out the remainder
    assert_eq!(1, i);
    assert_eq!(&s[..], [0, 0, 1, 0]);
}

///
/// This needs a better name. It takes a factorial rank and performs the
/// transformation that our implementation Heap's sequential algorithm expects,
/// returning the `i` remainder.
///
fn fixup_factorial(counters: &mut Box<[usize]>) -> usize {
    let mut i = 0; // It may not have yielded yet.
    for idx in 1..counters.len() {
        if counters[idx] == 0 {
            continue;
        }
        i = 1; // Heap's algo sets i=1 every time it yields
        counters[idx] -= 1;

        // we know that all of these are 0, because we are only at idx if
        // we skipped idx-... on the way here:
        for idx_borrowing in (1..idx).rev() {
            counters[idx_borrowing] += idx_borrowing;
        }
        break;
    }
    if i != 0 {
        // Simulate the while-loop in Heap's algo to fixup the counters
        for idx in i..counters.len() {
            if counters[idx] >= idx {
                counters[idx] = 0;
                i += 1;
                continue;
            }
            break;
        }
    }
    i
}

#[test]
fn test_sum_factorial_qs_1_6() {
    for n in 1..=6 {
        let n_factorial = (1..=n).product();
        for k1 in 0..=n_factorial {
            for k2 in 0..=n_factorial - k1 {
                let mut qs1 = get_qs(n + 1, 0, k1, vec![]).into_iter().rev().collect();
                let mut qs2 = get_qs(n + 1, 0, k2, vec![]).into_iter().rev().collect();
                let i1 = fixup_factorial(&mut qs1);
                let i2 = fixup_factorial(&mut qs2);
                let mut qs3: Box<_> = get_qs(n + 1, 0, k1 + k2, vec![])
                    .into_iter()
                    .rev()
                    .collect();
                let i3 = fixup_factorial(&mut qs3);
                let mut sum = sum_factorial(i1, &qs1, i2, &qs2);
                let i4 = fixup_factorial(&mut sum);
                assert_eq!(
                    qs3, sum,
                    "k1={k1} k2={k2} factoradic sums should match {i1} + {i2} == {i3} == {i4}"
                );
            }
        }
    }
}

impl<E: Clone + std::marker::Copy + std::fmt::Debug> Iterator for HeapsAlgorithm<E> {
    // we will be counting with usize
    type Item = Box<[E]>;

    ///
    /// Estimate number of remaining elements. For `n` where `usize` can hold the top rank
    /// without overflowing, the estimate is exact.
    ///
    /// # Examples
    /// ```
    /// # use heap_unranking::HeapsAlgorithm;
    /// let mut h = HeapsAlgorithm::new(vec![0,1,2,3]);
    /// let (lower, upper) = h.size_hint();
    /// assert_eq!(24, lower, "24 when nothing has been consumed");
    /// assert_eq!(Some(24), upper);
    /// let _ = h.next(); assert_eq!(23, h.size_hint().0, "when one has been consumed");
    /// let _ = h.next(); assert_eq!(22, h.size_hint().0, "when two have been consumed");
    /// for _ in (1..21) {
    ///     h.next();
    /// }
    /// let _ = h.next();
    ///   assert_eq!(h.size_hint().0,       1, "when 23 have been consumed");
    ///   assert_eq!(h.size_hint().1, Some(1), "when 23 have been consumed");
    /// let _ = h.next();
    ///   assert_eq!(h.size_hint().0,       0, "when 24 have been consumed");
    ///   assert_eq!(h.size_hint().1, Some(0), "when 24 have been consumed");
    /// let _ = h.previous();
    /// let _ = h.previous();
    ///   assert_eq!(h.size_hint().0,       2, "when 22 have been consumed");
    ///   assert_eq!(h.size_hint().1, Some(2), "when 22 have been consumed");
    /// ```
    ///
    /// ```rust
    /// # use heap_unranking::HeapsAlgorithm;
    /// let mut h = HeapsAlgorithm::new(vec![0; 500]);
    /// let (lower, upper) = h.size_hint();
    /// assert!(lower >= 5760, "at least factorial(8) <= usize::MAX on all platforms");
    /// assert_eq!(None, upper, "can't provide an exact upper bound");
    /// ```
    ///
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.i >= self.state.len() {
            return (0, Some(0));
        }
        let mut rem = 1_usize.saturating_sub(self.i); // self.i == 0 before the first next()
        let mut factorial: usize = 1;
        for (i, &q) in self.counters.iter().enumerate().skip(1) {
            rem += match (i).saturating_sub(q).checked_mul(factorial) {
                None => return (rem, None),
                Some(r) => r,
            };
            factorial = match factorial.checked_mul(i + 1) {
                None => return (rem, None),
                Some(n) => n,
            };
        }
        (rem, Some(rem))
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.step(predicate).cloned()
    }

    ///
    /// Next step of Heap's algorithm.
    ///
    ///
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.step(|_| true).cloned()
    }

    ///
    /// Skip `n` outputs and return the next permutation.
    ///
    /// `nth()` is very similar to [`heaps_state_at_k(n, k)`]
    /// except it's a relative jump from the current `k`.
    ///
    /// It should be equal to [`heaps_state_at_k(n, k-self.k)`],
    /// without relying on maintaining a `self.k`, so we can avoid
    /// dealing with overflows.
    ///
    fn nth(&mut self, k: usize) -> Option<Self::Item> {
        if self.i == 0 && k > self.state.len() {
            // Special case: we know how to skip from offset 0 to k:
            *self = Self::at_k(self.state.clone(), k);
            return self.next();
        }

        // This runs in O(k) == O(n!):
        for _ in self.i..k {
            self.step(|_| true);
        }
        self.next()
    }
}
