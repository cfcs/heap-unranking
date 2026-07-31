// Job splitting for Heap's algorithm: carving 0 .. n! into contiguous spans of ranks.
//
// A rank is a usize elsewhere in this crate, which caps n at 20 -- yet splitting the work
// is exactly the case where n is large, and the split itself never needs to hold a rank in
// a register. Boundaries are therefore computed in the factorial number system, the same
// digits unrank() consumes, where n! is one 1 followed by n zeroes. Dividing n! by the
// number of jobs, scaling a quotient and adding are then O(n) passes over small digits,
// so n! never materializes as an integer and no bignum is needed.

use crate::MAX_N;
use std::cmp::Ordering;

// A number in the factorial number system: digits[i] is the coefficient of i!, bounded by
// i, so digits[0] is always zero. Keeping that dead digit makes the index and the place
// agree, and both agree with the level numbering that precompute() and unrank() use.
#[derive(Clone, Debug)]
pub struct Factoradic {
    digits: Box<[u8]>,
}

impl Factoradic {
    // Zero, sized for permutations of n elements: place n is one more than any rank below
    // n! needs, which leaves room for the exclusive upper bound n! itself.
    pub fn zero(n: usize) -> Self {
        assert!(n < MAX_N, "Factoradic::zero({n}): digits must fit in a u8");
        Self {
            digits: vec![0u8; n + 1].into_boxed_slice(),
        }
    }

    pub fn from_rank(k: usize, n: usize) -> Self {
        let mut f = Factoradic::zero(n);
        let mut rest = k;
        for (i, d) in f.digits.iter_mut().enumerate().skip(1) {
            *d = (rest % (i + 1)) as u8;
            rest /= i + 1;
        }
        assert!(
            rest == 0,
            "Factoradic::from_rank({k}, {n}): k is not below {n}!"
        );
        f
    }

    // None when the value is too large for a usize, which starts at 21! on 64-bit.
    pub fn to_rank(&self) -> Option<usize> {
        let mut k = 0usize;
        for (i, &d) in self.digits.iter().enumerate().rev() {
            k = k.checked_mul(i + 1)?.checked_add(d as usize)?;
        }
        Some(k)
    }

    // Place-indexed, least significant first: digits()[i] is the coefficient of i!.
    pub fn digits(&self) -> &[u8] {
        &self.digits
    }

    // Place i holds values below i+1, so a sum of two digits and a carry stays below
    // twice that: the carry out is a bit, as it is in any positional system.
    pub fn add_assign(&mut self, rhs: &Factoradic) {
        assert_eq!(
            self.digits.len(),
            rhs.digits.len(),
            "factoradic addition needs operands of the same width"
        );

        let mut carry = 0usize;
        for (i, d) in self.digits.iter_mut().enumerate() {
            let sum = *d as usize + rhs.digits[i] as usize + carry;
            carry = (sum > i) as usize;
            *d = (sum - carry * (i + 1)) as u8;
        }
        assert!(carry == 0, "factoradic addition overflowed");
    }

    pub fn sub_assign(&mut self, rhs: &Factoradic) {
        assert_eq!(
            self.digits.len(),
            rhs.digits.len(),
            "factoradic subtraction needs operands of the same width"
        );

        let mut borrow = 0usize;
        for (i, d) in self.digits.iter_mut().enumerate() {
            let take = rhs.digits[i] as usize + borrow;
            borrow = ((*d as usize) < take) as usize;
            *d = (*d as usize + borrow * (i + 1) - take) as u8;
        }
        assert!(borrow == 0, "factoradic subtraction underflowed");
    }

    // Add a plain integer. A carry into place i is worth i!, so seeding the carry at place
    // 1 -- worth 1! -- adds k, and the usual normalization spreads it over the digits.
    pub fn add_rank(&mut self, k: usize) {
        let mut carry = k;
        for (i, d) in self.digits.iter_mut().enumerate().skip(1) {
            if carry == 0 {
                return;
            }
            let sum = carry
                .checked_add(*d as usize)
                .expect("factoradic addition overflowed");
            *d = (sum % (i + 1)) as u8;
            carry = sum / (i + 1);
        }
        assert!(carry == 0, "factoradic addition overflowed");
    }

    // Multiply by a plain integer. Unlike add_assign() the carry is no longer a bit, but
    // it stays below `factor`, so the products fit as long as factor * width does.
    pub fn scale(&mut self, factor: usize) {
        assert!(
            factor.checked_mul(self.digits.len() + 1).is_some(),
            "factoradic scaling by {factor} overflows a usize"
        );

        let mut carry = 0usize;
        for (i, d) in self.digits.iter_mut().enumerate() {
            let product = *d as usize * factor + carry;
            *d = (product % (i + 1)) as u8;
            carry = product / (i + 1);
        }
        assert!(carry == 0, "factoradic scaling overflowed");
    }
}

// Digits are stored least significant first, so the derived lexicographic order would be
// the wrong one. Comparing from the top also lets operands of different widths compare,
// which the arithmetic deliberately does not allow.
impl Ord for Factoradic {
    fn cmp(&self, other: &Self) -> Ordering {
        let width = self.digits.len().max(other.digits.len());
        for i in (0..width).rev() {
            let a = self.digits.get(i).copied().unwrap_or(0);
            let b = other.digits.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Factoradic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Factoradic {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Factoradic {}

// Long division of n! by `divisor`, returning the quotient and the remainder.
//
// Place i is worth i!, so stepping down one place multiplies the running remainder by
// i+1 where a fixed radix would multiply by the radix; otherwise this is schoolbook long
// division over the n+1 digits of n!, which are a single 1 in place n.
// Runtime: O(n)
pub fn factorial_div_rem(n: usize, divisor: usize) -> (Factoradic, usize) {
    assert!(n > 0, "factorial_div_rem(): n must be positive");
    assert!(divisor > 0, "factorial_div_rem(): divisor must be positive");
    assert!(
        divisor.checked_mul(n + 1).is_some(),
        "factorial_div_rem({n}, {divisor}): divisor * (n + 1) overflows a usize"
    );

    let mut quotient = Factoradic::zero(n);
    let mut rem = 0usize;
    for i in (0..=n).rev() {
        rem = rem * (i + 1) + (i == n) as usize;
        quotient.digits[i] = (rem / divisor) as u8;
        rem %= divisor;
    }

    (quotient, rem)
}

// The `index`'th of `parts + 1` boundaries evenly splitting 0 .. n!, i.e. the rank
// floor(index * n! / parts). Boundary 0 is zero and boundary `parts` is n! itself, so
// job `j` owns the half-open span between boundaries j and j+1.
//
// Computing one boundary rather than the whole vector is the point: a worker that knows
// only its own index can derive its span without coordinating.
// Runtime: O(n)
pub fn split_boundary(n: usize, parts: usize, index: usize) -> Factoradic {
    assert!(
        index <= parts,
        "split_boundary(): index {index} is past the last boundary {parts}"
    );

    // n! = quotient * parts + rem, so floor(index * n! / parts) is index quotients plus
    // the accumulated remainders. The product is taken in u128 because index and rem are
    // both bounded by parts and nothing else bounds their product.
    let (mut boundary, rem) = factorial_div_rem(n, parts);
    boundary.scale(index);
    boundary.add_rank((index as u128 * rem as u128 / parts as u128) as usize);
    boundary
}

// Every boundary of an even split of 0 .. n! into `parts` spans, ascending, `parts + 1`
// of them. Equivalent to calling split_boundary() for each index, but the boundaries are
// stepped rather than recomputed.
// Runtime: O(n * parts)
pub fn split_factoradic(n: usize, parts: usize) -> Vec<Factoradic> {
    let (step, rem) = factorial_div_rem(n, parts);

    // Spans are `step` long, and `rem` of them are one longer. Spreading those extra ones
    // by Bresenham rather than front-loading them keeps every boundary equal to what
    // split_boundary() computes in closed form.
    let mut boundary = Factoradic::zero(n);
    let mut acc = 0usize;
    let mut boundaries = Vec::with_capacity(parts + 1);
    for _ in 0..parts {
        boundaries.push(boundary.clone());
        boundary.add_assign(&step);
        acc += rem;
        if acc >= parts {
            acc -= parts;
            boundary.add_rank(1);
        }
    }
    boundaries.push(boundary);

    boundaries
}

// split_factoradic() as plain ranks, for the n where the whole range fits in a usize.
// Panics above n = 20 on a 64-bit target, where the last boundary n! does not.
pub fn split_ranks(n: usize, parts: usize) -> Vec<usize> {
    split_factoradic(n, parts)
        .iter()
        .map(|b| {
            b.to_rank()
                .unwrap_or_else(|| panic!("split_ranks({n}, {parts}): {n}! exceeds a usize"))
        })
        .collect()
}
