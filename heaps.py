"""
Ranking / unranking functions for Heap's algorithm.
cfcs, july-aug 2026

The rust implementation has received more love, but the more the merrier.
See the README in https://github.com/cfcs/heap-unranking for details.
"""


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
            if i & 1 == 0:
                arr[0], arr[i] = arr[i], arr[0]
            else:
                arr[c[i]], arr[i] = arr[i], arr[c[i]]
            yield tuple(arr)
            c[i] += 1
            i = 1
        else:
            c[i] = 0
            i += 1


####### Algorithms:


def forward_by_q(n, q, even_tmp, permutation):
    """Fast-forward a prefix permutation by (q) steps."""
    if n & 1 == 0:
        if n < 4:
            permutation[0], permutation[1 + (q & 1)] = (
                permutation[1 + (q & 1)],
                permutation[0],
            )
            permutation[1], permutation[2] = permutation[2], permutation[1]
            return
        even_tmp.clear()
        even_tmp.append(permutation[0])
        even_tmp.append(permutation[n - 1])
        even_tmp.append(permutation[n - 2])
        even_tmp.extend(permutation[1 : n - 2])
        even_tmp.append(permutation[n])
        permutation[0] = even_tmp[len(even_tmp) - q]
        permutation[n] = even_tmp[n - q]
        permutation[n - 1] = even_tmp[len(even_tmp) + 1 - q - len(even_tmp) * (1 == q)]
        permutation[n - 2] = even_tmp[len(even_tmp) + 2 - q - len(even_tmp) * (q <= 2)]
        start = len(even_tmp) + 3 - q - len(even_tmp) * (q <= 3)
        pivot = min(len(even_tmp) - start, n - 3)
        permutation[1 : pivot + 1] = even_tmp[start : start + pivot]
        permutation[1 + pivot : n - 2] = even_tmp[: n - 3 - pivot]
    else:
        permutation[n - 1], permutation[0], permutation[n] = (
            permutation[0],
            permutation[n],
            permutation[n - 1],
        )
        if q >= 2:
            permutation[1], permutation[2:q], permutation[n] = (
                permutation[n],
                permutation[1 : q - 1],
                permutation[q - 1],
            )
        if q & 1 == 0:
            permutation[0], permutation[n - 1] = permutation[n - 1], permutation[0]


def rank(identity, permutation) -> int:
    """The rust equivalent is rank_noprecomp_gen, O(1/2 n^2).

    Returns the rank k such that unrank(n, k) == P.
    Note that rank() also computes the factoradic encoding of (k) as (qs),
    computing the factorial inline at the cost of an extra loop.
    """

    # assert len(identity) == len(permutation)
    if len(permutation) <= 1:
        return 0
    arr = list(identity)
    qs = [0] * (len(permutation))
    even_tmp = [0] * len(permutation)
    for i in reversed(range(len(permutation))):
        if arr[i] == permutation[i]:
            continue
        if arr[0] == permutation[i]:
            qs[i] = i
            forward_by_q(i, i, even_tmp, arr)
            continue

        idx = arr.index(permutation[i], 1, i)
        if i & 1 == 1:
            qs[i] = 1 if idx + 1 == i else idx + 1
        else:
            qs[i] = i - idx - 2 if i - idx > 2 else idx
        forward_by_q(i, qs[i], even_tmp, arr)
    # Decode the factoradic rank into a (non-negative) integer rank:
    k = qs[0]
    fact_i = 1
    for i, q in enumerate(qs[1:]):
        fact_i *= i + 1
        k += q * fact_i
    return k


def unrank(identity, k: int) -> list[int]:
    """The rust equivalent is unrank_noprecomp_gen, O(1/2 n^2).
    Returns the k-th permutation of size n (equivalent to running (k) steps of Heap's algorithm).
    Invariants:
        0 < n
        0 <= k < n!
    """
    permutation = list(identity)

    # decode `k` into factorial number system digits (q);
    # we could pass in `k` already in this encoding to alleviate
    # this awkward loop:
    qs = [0] * (len(permutation))
    i_k = 1
    for _ in range(len(permutation) - 1):
        k //= i_k
        i_k += 1
        qs[i_k - 1] = k % i_k

    even_tmp = []
    # Iterate from the largest block down to the smallest:
    for n in reversed(range(len(permutation))):
        if 0 == qs[n]:
            continue
        forward_by_q(n, qs[n], even_tmp, permutation)
    return tuple(permutation)


####### Test suite:

"""
EXP: 3628799 (7, 8, 1, 2, 3, 4, 5, 6, 9, 0)
RES: 3628799 (7, 8, 1, 2, 3, 4, 5, 6, 9, 0) 10
Testing n=11 (39916800 permutations)...
EXP: 39916799 (10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0)
RES: 39916799 (10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0) 11
Testing n=12 (479001600 permutations)...
"""

import math


def run_test(n):
    print(f"Testing n={n} ({math.factorial(n)} permutations)...")

    last = None
    for k, expected in enumerate(heap_generate(n)):
        rk = rank(range(n), expected)
        assert rk == k, (rk, k)
        result = unrank(range(n), k)
        assert result == expected, (result, expected)
        # Validate that we did in fact only swap two elements:
        if last:
            assert verify_single_swap(last, expected), (last, expected)
        last = list(expected)  # copy

    print("EXP:", k, expected)
    print("RES:", k, result, n)
    if result != expected:
        print(f"Failure at k={k}!")
        print(f"Expected: {expected}")
        print(f"Got:      {result}")
        exit()

    print(f"All {math.factorial(n)} permutations match.")


if __name__ == "__main__":
    for size in range(1, 11):
        run_test(size)
