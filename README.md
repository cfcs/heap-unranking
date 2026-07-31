# Heap's algorithm: Ranking and unranking functions

@cfcs, 2026

## TL;DR

- `unrank(n, offset) -> Permutation`: "Seek"/skip to a numbered permutation output by Heap's algorithm for an array of length `n`.
- `rank(Permutation) -> int`: Identify the offset ("rank") of a given permutation from the start.
- `split_factoradic(n, parts) -> [Factoradic]`: Carve $$k \in 0 .. factorial(n) - 1$$ into one contiguous span per parallel job.

## Introduction

Heap's algorithm generates all permutations of an array of length `n` by swapping exactly two elements at each "step". Its simplicity of implementation and low overhead per step sometimes makes it an attractive alternative to lexicographical enumeration of permutations which requires division calculations that can be costly.

One would think that these properties would also make Heap's algorithm popular for parallel / distributed computations on permutations, but in order to split up the work of processing the $n$ permutations, a method to "resume" Heap's algorithm from a distant offset/rank is required. Unfortunately the Internet isn't exactly abundant with implementations of such unranking functions; I couldn't find any.

## How it works

Besides touching only two elements per step, Heap's algorithm has another interesting property:
The prefix permutation repeats, for example the permutations of `n` will consist of the permutation pattern for `n-1` applied n times, with an additional swap at the end. We can use this property to "fast-forward" by caching these end-state prefix permutations and keeping track of how many times they would have been applied.

Applying a cached prefix and its swap moves elements around without ever looking at their values, so it is a permutation of *positions*. That makes any number of repetitions at a given position collapse into one permutation, which `precompute()` tabulates for every (position, repetition count) pair. Each of the `n` positions then costs a single pass instead of a pass per repetition.

This yields an O(n^2) solution, which is "slow", but it's a lot faster than O(factorial(n)), and thus enables parallel programs to use Heap's algorithm for enumerating the permutations.

## Source code index

So here are a couple of implementations, based on exploiting the property that the prefix permutations repeat. Note that since the rust code uses `usize`, overflows for `n > 20` aren't handled. The python implementation is backed by a bigint library and should work correctly for any `n` and `k`.

Rust source code in `src/lib.rs`:
- `pub fn precompute(n)`: Precompute the prefix permutations up to `n`
- `pub fn unrank_recursive(n,k)`: functional, immutable, slow version
  - `python`: `heaps.py:HeapUnranker.unrank(self, n,k)` (more or less)
- `pub fn unrank(prefixes, n, k)`: return the `k`'th output of Heap's algorithm
  - `python`: `heaps.py:HeapUnranker.unrank_loop(self, n, k)`
- `pub fn unrank_into(prefixes, k, out)`: as `unrank`, writing into a caller-owned buffer
- `pub fn rank(prefixes, permutation)`: return `k` such that `permutation == unrank(n,k)`
  - `python`: `heaps.py:HeapUnranker.rank(self, n, P)`
- `pub fn unrank_factoradic(prefixes, n, k)`: as `unrank`, for a `k` too large to be a `usize`

Job splitting, in `src/split.rs`:
- `pub struct Factoradic`: a rank in the factorial number system, with the arithmetic the split needs
- `pub fn factorial_div_rem(n, divisor)`: divide `factorial(n)` without ever forming it
- `pub fn split_factoradic(n, parts)`: every boundary of an even split, ascending
- `pub fn split_boundary(n, parts, index)`: one boundary of that same split, in isolation
- `pub fn split_ranks(n, parts)`: `split_factoradic()` as plain `usize` ranks, for `n <= 20`

`cargo run --release --example bench` reports per-call throughput for both directions.

## Job splitting

`split_factoradic(n, parts)` returns `parts + 1` ascending boundaries, and job `j` owns the half-open span between boundaries `j` and `j+1`. Boundary `0` is `0` and boundary `parts` is `factorial(n)`, and no two spans differ in length by more than one permutation.

The obstacle is that a rank only fits in a `usize` up to `n = 20`, while splitting the work is exactly the case where `n` is larger. Boundaries are therefore computed in the factorial number system — the same digits `unrank()` consumes — in which `factorial(n)` is a single 1 followed by `n` zeroes. Dividing it by the number of jobs is then schoolbook long division over `n+1` small digits, differing from the fixed-radix version only in that stepping down one place multiplies the running remainder by `i+1` instead of by a constant. So `factorial(n)` never materializes as an integer and no bignum is involved.

Writing that division as $$factorial(n) = q \cdot parts + r$$, boundary `j` is $$j \cdot q + \lfloor j \cdot r / parts \rfloor$$. `split_boundary(n, parts, index)` evaluates it in O(n), so a worker that knows only its own index can derive its span without coordinating; `split_factoradic()` steps the boundaries instead, at one O(n) addition apiece.

Spans stop being longer than a `usize` well before their endpoints do, so at any `n` a job can be handed a start permutation and a count: `unrank_factoradic()` for the start, and the difference of two boundaries for the count.

## References

- [Wikipedia on Heap's algorithm](https://en.wikipedia.org/wiki/Heap%27s_algorithm)
- [Stackexchange explanation of the problem we're solving](https://cs.stackexchange.com/questions/165155/rank-and-unrank-for-heaps-algorithm)
- Knuth volume 4A: section `7.2.1.2`: Generating all permutations
  - "Bypassing unwanted blocks", see the part where Knuth talks about Heap's algorithm being a special case of "Algorithm G"
  - essentially rank() corresponds to converting k to step G1, and executing step G4, with unrank() is being the inverse

### References (other algorithms)
- *Ranking and unranking permutations in linear time*, Myrvold & Ruskey, 2000
- *Generation of Permutations by Transposition*, Mark B. Wells, August 1960
- *Permutation Enumeration: Four new permutation algorithms*, F.M. Ives, 1976 (with PL/I implementations!)
- *A Unified Framework to Discover Permutation Generation Algorithms*, Ganapath & Chowdhury, 2021
- *Strictly In-Place Algorithms for Permuting and Inverting Permutations*, Dudek & Gawrychowski & Pokorski, 2021
- [Efficient Algorithms to Rank and Unrank Permutations in Lexicographic Order, Bonet](https://bonetblai.github.io/reports/AAAI08-ws10-ranking.pdf)