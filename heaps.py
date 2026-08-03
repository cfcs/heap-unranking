"""
Ranking / unranking functions for Heap's algorithm.
cfcs, july 2026

The rust implementation has received more love, but the more the merrier.
See the README in https://github.com/cfcs/heap-unranking for details.
"""

import math

def verify_single_swap(p1, p2):
    """
    Returns True if p2 is reached from p1 by exactly one swap.
    """
    # Find all indices where the two permutations differ
    diffs = [i for i in range(len(p1)) if p1[i] != p2[i]]

    # A single swap must change exactly two positions
    if len(diffs) != 2:
        return False

    i, j = diffs
    # Verify that the values at those positions are swapped
    return p1[i] == p2[j] and p1[j] == p2[i]

def heap_generate(n):
    """
    Standard Heap's algorithm from Wikipedia:
    """
    arr = list(range(n))

    # iterative Heap's algorithm
    c = [0] * n
    yield tuple(arr)
    i = 1
    while i < n:
        if c[i] < i:
            if i % 2 == 0:
                arr[0], arr[i] = arr[i], arr[0]
            else:
                arr[c[i]], arr[i] = arr[i], arr[c[i]]
            yield tuple(arr)
            c[i] += 1
            i = 1
        else:
            c[i] = 0
            i += 1

import math

#######

class HeapUnranker:
    def __init__(self, max_n):
        self.S = {}  # Stores the full transformation permutation for each n
        self._precompute(max_n)

    def _precompute(self, max_n):
        """
        Precomputes S_n: The final state of the array after a full
        Heap's run on n elements, starting from [0, 1, ..., n-1].
        """
        self.S[1] = [0]
        self.fact = [1] * (max_n + 1)
        for n in range(1, max_n):
            if n < max_n - 1: # we don't need the last factorial:
                self.fact[n + 1] = self.fact[n] * (n + 1)
            arr = list(range(n + 1))
            for j in range(n):
                # this step happens sum(i for i in range(1, max_n)) times
                # Apply S_{n-1} to the first n-1 elements
                arr[:len(self.S[n])] = [arr[p] for p in self.S[n]]
                # if n is odd, j; when even: 0
                j_or_0 = (n & 1) * j
                arr[j_or_0], arr[n] = arr[n], arr[j_or_0]

            # Final block of S_{n-1}
            arr[:len(self.S[n])] = [arr[p] for p in self.S[n]]
            self.S[n + 1] = arr # TODO do we need the last at self.S[max_n] ??
            # TODO seems like we never actually access anything but max_n-1

    def rank(self, n, P):
        """
        Returns the rank k such that unrank(n, k) == P.
        Note that rank() also computes the factoradic encoding of (k) as (qs),
        computing the factorial inline at the cost of an extra loop.
        """
        arr = list(range(n))
        total_rank = 0
        qs = [0] * n # k encoded as factoradic

        # Iterate from the largest block down to the smallest
        for i in range(n-1, 0, -1): # O(n)
            q = 0
            # Transition through blocks until the last element matches target P
            # TODO how do we prove these cases terminate?
            if i & 1 == 1:
                while arr[i] != P[i]:
                    arr[:len(self.S[i])] = [arr[p] for p in self.S[i]]
                    arr[q], arr[i] = arr[i], arr[q]
                    q += 1
            else:
                # this branch could maybe run through self.S[i-1] looking for
                # P[i-1] and establish q based on that?
                while arr[i] != P[i]:
                    arr[:len(self.S[i])] = [arr[p] for p in self.S[i]]
                    arr[0], arr[i] = arr[i], arr[0]
                    q += 1
            qs[i] = q
            total_rank += q * self.fact[i]

        # alternative method of computing k from qs:
        q_rank = 0
        fact_i = 1
        for i in range(0,n-1):
            q_rank += qs[i] * fact_i
            fact_i *= (i+1)
        q_rank += qs[n-1] * fact_i # last iteration doesn't need fact_i *= (n-1)

        assert q_rank == total_rank

        return total_rank

    def unrank(self, n, k):
        """
        Returns the k-th permutation of size n.
        Invariants:
          0 < n
          0 <= k < n!
        """
        arr = list(range(n))
        return self._unrank_recursive(n, k, arr)

    def _unrank_recursive(self, n, k, arr):
        if n <= 1: # base case
            return tuple(arr)

        q, r = divmod(k, self.fact[n-1])
        # (k <= n!) ->
        #    { 0 <= q <= k <= n! }
        #    { 0 <= r <= k <= n! }
        # division by (n-1)! means:
        #    (k <= n!) -> { q < n! / (n-1)! } <=> { q <= n }
        #    { r < (n-1)! }

        # in each step we make at most q <= n swaps; O(n)
        # but we also recurse. since (n-1)! goes towards 1,
        # eventually r will be 0.
        # Another way to think about it is that we are counting down towards
        # 1 with our (n -1), so we have at most n-1 recursive calls, meaning we
        # have at most n function body invocations.
        # A conservative upper bound is thus O(n^2), assuming we can compute
        # divmod(k, (n-1)!) in O(n) too.

        # 1. Move the array state forward by q full blocks of size (n-1)!
        for j in range(q):
            # 2. Every full block consists of a full run of (n-1), which we
            #    have cached:
            arr[:len(self.S[n-1])] = [arr[p] for p in self.S[n-1]]
            # 3. followed by the specific swap for this level:
            j_or_0 = (n & 1 == 0) * j
            arr[j_or_0], arr[n-1] = arr[n-1], arr[j_or_0]

        # 4. Now we are at the start of the q-th block.
        #    We need to find the r-th permutation of the first n-1 elements.
        return self._unrank_recursive(n - 1, r, arr)

    def unrank_loop(self, nn, k):
        """Loop version.

        Unlike the other version, this doesn't use the memoized factorial table,
        but it does use the permutation table for S[0..n-1].
        Instead it computes the quotients from reducing k into factoriadic digits,
        which are used for the inner loop counter.
        The upside of that is that we can get away with not having factorial products
        and instead can operate on the indices. Downside is we need a forward-directed
        loop to nn in addition to the downwards loop. from nn to 0.

        Would probably make sense to have a version that doesn't rely on the
        self.S cache.
        """

        # decode `k` into factorial number system digits (q);
        # we could pass in `k` already in this encoding to alleviate
        # this awkward loop:
        qs = [0] * nn # we skip i=1 below because it computes k'=k; qs[0]=0 every time:
        for i in range(2, nn+1):
            k, qs[i-1] = divmod(k, i)

        arr = list(range(nn)) # iota n, 0-indexed
        for n in range(nn-1, 0, -1):
            # TODO "q" is probably a bad name since it's a loop from 0 to the q:
            for q in range(qs[n]):
                arr[:len(self.S[n])] = [arr[p] for p in self.S[n]]
                q_or_0 = (n & 1) * q
                arr[q_or_0], arr[n] = arr[n], arr[q_or_0]
                # assert len(arr) > len(self.S[n])
                # assert qs[n] >= q_or_0
                # assert q_or_0 < n
                # assert q_or_0 < len(self.S[n])
        return tuple(arr)

"""
PRECOMP 10 45
EXP: 3628799 (7, 8, 1, 2, 3, 4, 5, 6, 9, 0)
RES: 3628799 (7, 8, 1, 2, 3, 4, 5, 6, 9, 0) 10
Testing n=11 (39916800 permutations)...
PRECOMP 11 55
EXP: 39916799 (10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0)
RES: 39916799 (10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0) 11
Testing n=12 (479001600 permutations)...
PRECOMP 12 66
"""

def run_test(n):
    print(f"Testing n={n} ({math.factorial(n)} permutations)...")

    unranker = HeapUnranker(n)
    last = None
    for k, expected in enumerate(heap_generate(n)):
        uk = unranker.rank(n, expected)
        assert uk == k
        #print(k, "unranked", uk)
        #print(list(expected),k)
        if last:
            assert verify_single_swap(last, expected), (last, expected)
        last = list(expected) # copy
        result = unranker.unrank(n, k)
        assert expected == result, k
        result2 = unranker.unrank_loop(n, k)
        assert expected == result2

    print("EXP:", k, expected)
    print("RES:", k, result, n)
    if result != expected:
         print(f"Failure at k={k}!")
         print(f"Expected: {expected}")
         print(f"Got:      {result}")
         exit()

    print(f"All {math.factorial(n)} permutations match.")

if __name__ == "__main__":
    for size in range(1,11):
        run_test(size)
