# Heap's algorithm: Ranking and unranking functions

@cfcs, 2026

## TL;DR

- `unrank(n, offset) -> Permutation`: "Seek"/skip to a numbered permutation output by Heap's algorithm for an array of length `n`.
- `rank(Permutation) -> int`: Identify the offset ("rank") of a given permutation from the start.

## Introduction

Heap's algorithm generates all permutations of an array of length `n` by swapping exactly two elements at each "step". Its simplicity of implementation and low overhead per step sometimes makes it an attractive alternative to lexicographical enumeration of permutations which requires division calculations that can be costly.

One would think that these properties would also make Heap's algorithm popular for parallel / distributed computations on permutations, but in order to split up the work of processing the $n$ permutations, a method to "resume" Heap's algorithm from a distant offset/rank is required. Unfortunately the Internet isn't exactly abundant with implementations of such unranking functions; I couldn't find any.

## How it works

Besides touching only two elements per step, Heap's algorithm has another interesting property:
The prefix permutation repeats, for example the permutations of `n` will consist of the permutation pattern for `n-1` applied n times, with an additional swap at the end. We can use this property to "fast-forward" by caching these end-state prefix permutations and keeping track of how many times they would have been applied.

This yields an O(n^2) solution, which is "slow", but it's a lot faster than O(factorial(n)), and thus enables parallel programs to use Heap's algorithm for enumerating the permutations.

## Source code index

So here are a couple of implementations, based on exploiting the property that the prefix permutations repeat. Note that since the rust code uses `usize`, overflows for `n > 20` aren't handled. The python implementation is backed by a bigint library and should work correctly for any `n` and `k`.

Rust source code in `src/lib.rs`:
- `pub fn precompute(n)`: Precompute the prefix permutations up to `n`
- `pub fn unrank_recursive(n,k)`: functional, immutable, slow version
  - `python`: `heaps.py:HeapUnranker.unrank(self, n,k)` (more or less)
- `pub fn unrank(prefixes, n, k)`: return the `k`'th output of Heap's algorithm
  - `python`: `heaps.py:HeapUnranker.unrank_loop(self, n, k)`
- `pub fn rank(prefixes, permutation)`: return `k` such that `permutation == unrank(n,k)`
  - `python`: `heaps.py:HeapUnranker.rank(self, n, P)`

## Missing

What's currently missing from this repo is an efficient algorithm for job splitting, computing either `k` spans or, probably more interesting, factoradic spans to cover $$k \in 0 .. factorial(n) - 1$$ for a given number of partitions.

## References

- [Wikipedia on Heap's algorithm](https://en.wikipedia.org/wiki/Heap%27s_algorithm)
- [Stackexchange explanation of the problem we're solving](https://cs.stackexchange.com/questions/165155/rank-and-unrank-for-heaps-algorithm)
- [Efficient Algorithms to Rank and Unrank Permutations in Lexicographic Order, Bonet](https://bonetblai.github.io/reports/AAAI08-ws10-ranking.pdf) - interesting paper about solving it for a different permutation algorithm
- Knuth volume 4A: section `7.2.1.2`: Generating all permutations
  - "Bypassing unwanted blocks", see the part where Knuth talks about Heap's algorithm being a special case of "Algorithm G"
  - essentially rank() corresponds to converting k to step G1, and executing step G4, with unrank() is being the inverse
