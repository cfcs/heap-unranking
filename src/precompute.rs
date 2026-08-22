//!
//! Slower alternative implementations of ranking/unranking
//!

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
///
/// It seems entirely possible to skip these.
///
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

/// Both functions use a precomputed table of final states for prefixes 1 ..= n-1
/// obtainable with precompute(n-1)
///
/// `arr\[ 0 : len(S) \] = arr\[p\]` for p in S
/// the "scratch" buffer is used to store the collected elements
/// so we don't have to worry about overwriting entries we need later in the loop
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
pub fn unrank(prefixes: &[Box<[u8]>], n: usize, mut k: usize) -> Box<[u8]> {
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
                reset_permutation(&mut scratch, prefix, &mut permutation); // O(n)
                permutation.swap(0, n); // O(1)
            }
        } else {
            assert!(q < permutation.len()); // try to get the compiler to elide bounds checks below
            for i in 0..q {
                // O(n)
                reset_permutation(&mut scratch, prefix, &mut permutation); // O(n)
                permutation.swap(i, n); // O(1)
            }
        }
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
