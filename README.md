# Heap's algorithm: Ranking and unranking functions

@cfcs, July 2026

## TL;DR

- `unrank(n:nat, rank: nat) -> Permutation:array`: "Seek"/skip to a numbered permutation output by Heap's algorithm for an array of length `n`.
- `rank(Permutation:array) -> rank:nat`: Identify the offset ("rank") of a given permutation from the start.

## Introduction

Heap's algorithm generates all permutations of an array of length `n` by swapping exactly two elements at each "step". Its simplicity of implementation and low overhead per step sometimes makes it an attractive alternative to lexicographical enumeration of permutations which requires division calculations that can be costly.

One would think that these properties would also make Heap's algorithm popular for parallel / distributed computations on permutations, but in order to split up the work of processing the $n$ permutations, a method to "resume" Heap's algorithm from a distant offset/rank is required. Unfortunately the Internet isn't exactly abundant with implementations of such unranking functions; I couldn't find any.

## How it works

Besides touching only two elements per step, Heap's algorithm other interesting properties:
1. The algorithm works by transposing two elements per step (swapping by indices), and does not examine the element values. This means the resulting pattern is *the same* for any `n` (and *solely dependent* on `n`).
2. The prefix permutation repeats, for example the permutations of `n` will consist of the permutation pattern for `n-1` applied `n` times, with an additional swap at the end. We can use this property to "fast-forward" by caching these end-state prefix permutations and keeping track of how many times they would have been applied.

This yields an $O(n^3)$ solution, which is "slow", but it's a lot faster than $O(\text{factorial}(n))$, and thus enables parallel programs to use Heap's algorithm for enumerating the permutations.

3. It is perhaps also worth mentioning that fewer than $n-1$ prefixes are needed for small $k$; for example $ k = 0 $ does not make use of the prefix-enabled skipping.

4. It [seems likely](https://github.com/cfcs/heap-unranking/pull/1) that there exists a trade-off spectrum between the size of the prefix table and the runtime complexity per `rank()`/`unrank()` that yields an $O(n^2)$ implementation, at the cost of an $O(n^3)$ *space* to store multiple prefixes to cache each of the the "jumps" for each factoradic digit value. Storing a fraction of these (say half of them, or $\log{n}$ of them), can then be balanced to a sliding scale like $O(n^2 \log{n})$ runtime + space.

## Source code index

So here are a couple of implementations, based on exploiting the property that the prefix permutations repeat. Note that since the rust code uses `usize`, overflows for `n > 20` aren't handled. The python implementation is backed by a bigint library and should work correctly for any `n` and `k`.

Rust source code in `src/lib.rs`:
- `pub fn precompute(n)`: Precompute the prefix permutations up to `n` in $ O(\frac{1}{2}n^3 + n) $ time and $ O(\frac{1}{2} n^2) $ space.
- `pub fn unrank_recursive(n,k)`: functional, immutable, slow version
  - `python`: `heaps.py:HeapUnranker.unrank(self, n,k)` (more or less)
- `pub fn unrank(prefixes, n, k)`: return the `k`'th output of Heap's algorithm in $ O(\frac{1}{2}n^3) $ time.
  - `python`: `heaps.py:HeapUnranker.unrank_loop(self, n, k)`
- `pub fn rank(prefixes, permutation)`: return `k` such that `permutation == unrank(n,k)`, in $ O(\frac{1}{2}n^3 + n) $ time.
  - `python`: `heaps.py:HeapUnranker.rank(self, n, P)`
- `tests/oeis.rs`: Calculations related to [OEIS A280318](https://oeis.org/A280318) in time less than $O(n!)$
  - `check_oeis_table_5040()`: `a(n)` for an arbitrary `n`.
  - `rank_example_for_n_4()`: Finding `n` given `a(n)`, using `rank()`.

## Missing

- What's currently missing from this repo is an efficient algorithm for job splitting, computing either `k` spans or, probably more interesting, factoradic spans to cover $$k \in 0 .. \text{factorial}(n) - 1$$ for a given number of partitions.

## References

- [Wikipedia on Heap's algorithm](https://en.wikipedia.org/wiki/Heap%27s_algorithm)
- [Stackexchange explanation of the problem we're solving](https://cs.stackexchange.com/questions/165155/rank-and-unrank-for-heaps-algorithm)
- [Ruslan Ledesma-Garza's article about Heap's Algorithm](https://ruslanledesma.com/2016/06/17/why-does-heap-work.html)
- Knuth volume 4A: section `7.2.1.2`: Generating all permutations
  - "Bypassing unwanted blocks", see the part where Knuth talks about Heap's algorithm being a special case of "Algorithm G"
  - essentially rank() corresponds to converting k to step G1, and executing step G4, with unrank() is being the inverse
- [Factoradic / Factorial number system](https://en.wikipedia.org/wiki/Factorial_number_system)

### References (other algorithms)
- *More on permutation generation methods*, Lipski, 1979
- *Ranking and unranking permutations in linear time*, Myrvold & Ruskey, 2000
- [Unranking permutations in transposition order and linear time, Konstantinos A. Blekos](https://arxiv.org/abs/0806.1371)
  - This paper is pretty sparse, but it is interesting because its permutations *also only differ by one swap per output.* It seems like it's an open problem to derive a Heaps' algorithm-like imperative implementation that efficiently steps through these permutations, at least I couldn't find one. But if it can be made to run as fast as Heap's algorithm, the $O(n)$ complexity of the unranking would make this permutation order **preferable to Heap's algorithm**. Exercise for the reader. :-)
- *Generation of Permutations by Transposition*, Mark B. Wells, August 1960
- *Permutation Enumeration: Four new permutation algorithms*, F.M. Ives, 1976 (with PL/I implementations!)
- *A Unified Framework to Discover Permutation Generation Algorithms*, Ganapath & Chowdhury, 2021
- *Strictly In-Place Algorithms for Permuting and Inverting Permutations*, Dudek & Gawrychowski & Pokorski, 2021
- [Efficient Algorithms to Rank and Unrank Permutations in Lexicographic Order, Bonet](https://bonetblai.github.io/reports/AAAI08-ws10-ranking.pdf)