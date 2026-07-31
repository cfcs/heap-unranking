// Ranking / unranking functions for Heap's algorithm
// cfcs, 2026
//
// unrank(n,k): "skip" k outputs of Heap's algorithm
// rank(P): calculate "k" (how many iterations of Heap's algorithm it took to produce P)
//
// Both functions use precomputed tables obtainable with precompute(n-1)

// Permutation elements are u8, so no array can hold more than 256 distinct values.
// rank()/unrank() size their stack buffers to this, which also lets the compiler see
// that a u8 read out of the tables is always an in-bounds index (see gather_in_place).
const MAX_N: usize = 256;

// arr[0 .. p.len()] = [arr[p[0]], arr[p[1]], ..]; the tail of arr is left untouched.
// the "scratch" buffer is used to store the collected elements
// so we don't have to worry about overwriting entries we need later in the loop
#[inline(always)]
fn gather_in_place(p: &[u8], arr: &mut [u8; MAX_N], scratch: &mut [u8; MAX_N]) {
    for (dst, &src) in scratch.iter_mut().zip(p) {
        *dst = arr[src as usize];
    }
    arr[..p.len()].copy_from_slice(&scratch[..p.len()]);
}

//
// Heap's algorithm at "level" i -- the stage that settles position i -- repeats a
// fixed two-part move: apply the level-(i-1) prefix permutation to positions 0..i,
// then swap one of those with position i. Neither part inspects the values in the
// array, only positions, so q repetitions of the move compose into a single
// permutation of positions 0 ..= i determined by (i, q) alone.
//
// Replaying those q moves one at a time costs O(n) per move and up to i moves per
// level, i.e. O(n^3) overall. Storing the composite for every (i, q) instead turns
// each level into one O(n) gather, so rank() and unrank() become O(n^2) -- which is
// what the prefix-repetition property promises in the first place. The table costs
// O(n^3) bytes, a few kilobytes at the n <= 20 that a usize k can address.
//
#[derive(Clone)]
pub struct Prefixes {
    // Level i's block holds i+1 rows of i+1 bytes at block_base(i), row-major.
    // Row q maps each destination position to its source position after q moves.
    steps: Box<[u8]>,

    // Level i's block holds i+1 bytes at column_base(i), inverting the last entry of
    // each row: source_to_q[c] is the move count that lands position c's value on
    // position i. rank() recovers a factoradic digit from exactly that.
    source_to_q: Box<[u8]>,

    max_level: usize,
}

// Levels 0 .. level occupy (j+1)^2 and (j+1) bytes respectively, so their offsets are
// the closed forms for the sums of squares and of integers up to `level`.
#[inline(always)]
fn block_base(level: usize) -> usize {
    level * (level + 1) * (2 * level + 1) / 6
}

#[inline(always)]
fn column_base(level: usize) -> usize {
    level * (level + 1) / 2
}

impl Prefixes {
    // The largest n that rank()/unrank() can serve is max_n() + 1.
    pub fn max_n(&self) -> usize {
        self.max_level
    }

    #[inline(always)]
    fn step(&self, level: usize, q: usize) -> &[u8] {
        let width = level + 1;
        let start = block_base(level) + q * width;
        &self.steps[start..start + width]
    }

    #[inline(always)]
    fn q_by_source(&self, level: usize, source: usize) -> usize {
        self.source_to_q[column_base(level) + source] as usize
    }
}

// Precompute the composite move tables for levels 0 ..= max_n, which serves
// rank()/unrank() for permutations of up to max_n + 1 elements.
// Runtime: O(n^3)
// Space:   O(n^3)
pub fn precompute(max_n: usize) -> Prefixes {
    assert!(
        max_n < u8::MAX as usize,
        "precompute({max_n}): move counts must fit in a u8"
    );

    let mut steps = vec![0u8; block_base(max_n + 1)];
    let mut source_to_q = vec![u8::MAX; column_base(max_n + 1)];

    // The final state Heap's algorithm reaches for an array of length i -- s[i-1] in
    // the original formulation. Only the current level's copy is ever needed, so it
    // rolls forward in place rather than being kept per level.
    // level 0 is the one-element identity: its only row is already zeroed, and it
    // seeds `prefix` with s[0] = [0]
    let mut prefix = [0u8; MAX_N];
    let mut row = [0u8; MAX_N];
    let mut scratch = [0u8; MAX_N];
    source_to_q[0] = 0;

    for level in 1..=max_n {
        let width = level + 1;
        let base = block_base(level);
        let qbase = column_base(level);

        // zero moves is the identity, and leaves position `level` where it started
        for (t, r) in row[..width].iter_mut().enumerate() {
            *r = t as u8;
        }
        steps[base..base + width].copy_from_slice(&row[..width]);
        source_to_q[qbase + level] = 0;

        for q in 1..=level {
            gather_in_place(&prefix[..level], &mut row, &mut scratch);
            row.swap((level & 1) * (q - 1), level);

            let start = base + q * width;
            steps[start..start + width].copy_from_slice(&row[..width]);
            source_to_q[qbase + row[level] as usize] = q as u8;
        }

        // the level ends with one more prefix application after its `level` moves,
        // and that end state is the prefix the next level repeats
        gather_in_place(&prefix[..level], &mut row, &mut scratch);
        prefix[..width].copy_from_slice(&row[..width]);
    }

    // rank() indexes source_to_q without a fallback, which is only sound because each
    // level's moves land a distinct source on the settled position.
    assert!(
        !source_to_q.contains(&u8::MAX),
        "move counts do not cover every source position"
    );

    Prefixes {
        steps: steps.into_boxed_slice(),
        source_to_q: source_to_q.into_boxed_slice(),
        max_level: max_n,
    }
}

// Write the k'th output of Heap's algorithm for a permutation of out.len() elements.
// Runtime: O(n^2) worst case, O(n) best case
pub fn unrank_into(prefixes: &Prefixes, mut k: usize, out: &mut [u8]) {
    let n = out.len();
    let mut arr = [0u8; MAX_N];
    for (t, a) in arr[..n].iter_mut().enumerate() {
        *a = t as u8; // 0, 1, .., n-1
    }

    // Translate k to factoradic digits: qs[i-1] is level i's move count. k running dry
    // means every remaining digit is zero, so `top` is the highest level that moves at
    // all -- values of k well below n! skip the wide levels entirely.
    //
    // The bases are spelled out as literals so the compiler can turn each division into
    // a multiply, which on a 64-bit usize is worth about a third of unrank()'s running
    // time. usize::MAX < 21!, so dividing through base 21 always drains k and the
    // trailing loop is dead -- it is here to stay correct for a wider usize.
    let mut qs = [0u8; MAX_N];
    let mut top = 0;

    macro_rules! digits {
        ($($base:literal),*) => {$(
            if k != 0 && $base <= n {
                let q = k % $base;
                k /= $base;
                if q != 0 {
                    qs[$base - 2] = q as u8;
                    top = $base - 1;
                }
            }
        )*};
    }
    digits!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21);

    let mut base = 22;
    while k != 0 && base <= n {
        let q = k % base;
        k /= base;
        if q != 0 {
            qs[base - 2] = q as u8;
            top = base - 1;
        }
        base += 1;
    }

    assert!(
        top <= prefixes.max_level,
        "unrank() needs precompute({top}), got precompute({})",
        prefixes.max_level
    );

    let mut scratch = [0u8; MAX_N];
    for level in (1..=top).rev() {
        let q = qs[level - 1] as usize;
        if q != 0 {
            gather_in_place(prefixes.step(level, q), &mut arr, &mut scratch);
        }
    }

    out.copy_from_slice(&arr[..n]);
}

// Compute the k'th output of Heap's algorithm for a permutation of n elements.
// unrank_into() is the same thing without the per-call allocation.
pub fn unrank(prefixes: &Prefixes, n: usize, k: usize) -> Box<[u8]> {
    let mut out = vec![0u8; n].into_boxed_slice();
    unrank_into(prefixes, k, &mut out);
    out
}

//
// rank(precompute(), P) computes result k such that unrank(precompute(), k) == P
// it computes the number of iterations of Heap's algorithm needed
// to reach P.
// Runtime: O(n^2)
//
pub fn rank(prefixes: &Prefixes, permutation: &[u8]) -> usize {
    let n = permutation.len();
    assert!(
        n <= prefixes.max_level + 1,
        "rank() needs precompute({}), got precompute({})",
        n.saturating_sub(1),
        prefixes.max_level
    );

    // `pos` is indexed by value rather than searched, so an out-of-range or repeated
    // element would quietly produce a wrong k instead of being caught.
    let mut seen = [0u64; MAX_N / 64];
    for &v in permutation {
        let v = v as usize;
        assert!(
            v < n && seen[v >> 6] & (1 << (v & 63)) == 0,
            "rank() argument is not a permutation of 0..n"
        );
        seen[v >> 6] |= 1 << (v & 63);
    }

    let mut arr = [0u8; MAX_N];
    let mut pos = [0u8; MAX_N]; // inverse of arr: pos[value] is where that value sits
    for t in 0..n {
        arr[t] = t as u8;
        pos[t] = t as u8;
    }
    let mut scratch = [0u8; MAX_N];

    // Horner over the factoradic digits, most significant first. No intermediate
    // exceeds the final k, unlike accumulating q * i! per digit, which overflows for
    // some k that are themselves representable.
    let mut k = 0usize;
    for level in (1..n).rev() {
        // q moves bring the value at source position step(level, q)[level] to
        // position `level`, so where the wanted value currently sits fixes the digit.
        // Keeping the inverse up to date makes that a lookup rather than a search,
        // trading a branch the predictor keeps missing for a handful of stores.
        let source = pos[permutation[level] as usize] as usize;
        let q = prefixes.q_by_source(level, source);
        k = k * (level + 1) + q;

        if q != 0 {
            let step = prefixes.step(level, q);
            for (t, (dst, &src)) in scratch.iter_mut().zip(step).enumerate() {
                let v = arr[src as usize];
                *dst = v;
                pos[v as usize] = t as u8;
            }
            arr[..step.len()].copy_from_slice(&scratch[..step.len()]);
        }
    }

    k
}

// functional/recursive implementation of factorizing k into factoradic digits
pub fn get_qs(n: usize, i: usize, k: usize, acc: Vec<usize>) -> Vec<usize> {
    assert!(n > 0); // calling get_qs with n=0 is an error
    if n == 1 {
        acc
    } else {
        let acc2 = vec![k % (i + 1)].into_iter().chain(acc).collect(); // k%i :: acc
        get_qs(n - 1, i + 1, k / (i + 1), acc2)
    }
}

//
// Functional/recursive version of unrank()
// This is pretty inefficient, but is here to serve as an alternative explanation
// of what is going on, or to assist in porting/proving efforts.
//
pub fn unrank_recursive(n: usize, k: usize) -> Vec<u8> {
    assert!(n > 0); // invalid if k >= factorial(n), which means n==0 /\ k==0 is invalid
    fn reset_permutation_functional(prefixes: &[u8], arr: Vec<u8>) -> Vec<u8> {
        prefixes
            .iter()
            .map(|&p| arr[p as usize])
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
