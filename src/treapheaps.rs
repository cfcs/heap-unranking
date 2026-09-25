//! Implicit Treap-backed [`forward_by_qs`], [`rank_treap`], [`unrank_treap`] in $O(n log n)$
//!//!
//! **NB:* The treap skeleton was helpfully donated by an LLM. YMMV.
//!
//! # The idea (`forward_by_qs`)
//!
//! The expensive part of `forward_by_q()` is the transposition, specifically
//! the "shift" stride. Here we defer moving the elements until the final materialization
//! of the transposition, and instead keep track of the source index ranges in an implicit
//! cartesian tree so we can perform range operations over the (index range) elements
//! in amortized O(log n).
//!
//! This yields a total runtime of $O(n log n)$.
//!
//! - We build a n augmented implicit treap in $O(n)$, see `ImplicitTreap::new` and `ImplicitTreap::build_cartesian_tree`.
//!   - The augmentations:
//!     - Our tree is *very* implicit in that it has neither keys nor values
//!     - We add a `parent` pointer to support `rank(e)` (`position(element)`), not to be confused with
//!       the Heap's Algorithm ranking. It maps from a source index (permutation identity) to a
//!       transposed index and is required for the [`rank_treap`] operation.
//! - For each of the `n` factoradic digits `q` we run `forward_odd` or `forward_even` (respectively).
//!   - They each synthesize a constant number of index ranges.
//!   - Then apply the ranges to `ImplicitTreap::compose_reorders` which XXX.
//!     - Then apply the result to `ImplicitTreap::reorder_prefix2` which performs a number of treap
//!       operations governed by the number of ranges.
//!       Each of the treap operations (`split`, `merge`) run in amortized $O(log n)$.
//! - We serialize and materialize the final indices in $O(n)$, see `ImplicitTreap::into_indices`.
//!
//! # [`unrank_treap`]
//! A straight-forward wrapper around [`forward_by_qs`], with the factorization of `k` into factoradic digits.
//! # [`rank_treap`]
//! This is less straight-forward; in order to support the
//! "find the left-to-right position of element E" in $O(\log{n})$ we add
//! `parent`-pointers to the treap.
//!
//! # Benchmarks from my development computer (`forward_by_qs`)
//! Benchmarking in terms of `heap_unranking::forward_by_q`:
//! - For `n=100_005` this is 10 times faster.
//! - For `n=50_000` this is 5 times faster.
//! - `n=22_000` is where they break even
//! - For `n=10_000` this is about twice as *slow* as the `forward_by_q` algorithm.
//!

#[inline(always)]
fn push_range(ranges: &mut Vec<(u32, u32)>, start: u32, end: u32) {
    if start < end {
        ranges.push((start as u32, end as u32));
    }
}

#[derive(Debug)]
struct Node<I> {
    left: I,
    right: I,
    parent: I,
    priority: u32,
    size: I,
}

#[derive(Debug)]
pub struct ImplicitTreap {
    nodes: Vec<Node<u32>>,
    root: u32,
    rng_state: u32,
    pieces: Vec<u32>, // buffer, also used in ::new() as a stack
    ranges: Vec<(u32, u32)>,
    compose_buf: [(u32, u32); 10], // for the composition of the 6+4 ranges
}

const NODE_NONE: u32 = u32::MAX;

impl ImplicitTreap {
    /// Construct a max-heap Cartesian tree in index order.
    /// The tree has implicit keys (the element's most recent index), determined by the left/right
    /// pointers; and implicit values (the original index). The original index is the node index
    /// into self.nodes.
    #[inline]
    fn build_cartesian_tree(&mut self, num_values: usize) {
        if num_values == 0 {
            self.root = NODE_NONE;
            return;
        }

        for index in 0..num_values {
            let priority: u32 = self.next_priority();
            let mut last = NODE_NONE;

            while let Some(top) = self
                .pieces
                .pop_if(|&mut top| self.nodes[top as usize].priority < priority)
            {
                last = top;
            }

            if last != NODE_NONE {
                self.nodes[last as usize].parent = index as u32; // we become their parent
            }

            self.nodes.push(Node {
                priority,
                left: last,
                right: NODE_NONE,
                parent: NODE_NONE,
                size: 1,
            });
            if let Some(&top) = self.pieces.last() {
                self.nodes[top as usize].right = index as u32;
                self.nodes[index as usize].parent = top;
            }

            self.pieces.push(index as u32);
        }

        self.root = self.pieces[0];
        self.nodes[self.root as usize].parent = NODE_NONE;

        self.ranges.clear();
        self.ranges.push((self.root, 0_u32));

        while let Some((index, visited)) = self.ranges.pop() {
            if visited != 0 {
                self.update(index);
                continue;
            }
            self.ranges.push((index, 1));
            let node_at_index = &self.nodes[index as usize];
            if let right = node_at_index.right
                && right != NODE_NONE
            {
                self.ranges.push((right, 0));
            }

            if let left = node_at_index.left
                && left != NODE_NONE
            {
                self.ranges.push((left, 0));
            }
        }
        self.pieces.clear();
    }

    pub fn new(values: usize) -> Self {
        let mut treap = Self {
            nodes: Vec::with_capacity(values),
            root: NODE_NONE,
            rng_state: 0x1234_5678,
            pieces: Vec::with_capacity(values),
            ranges: Vec::with_capacity(values.min(8)),
            compose_buf: [(0_u32, 0); 10],
            //split_path: Vec::with_capacity(32), // should be log2 of values
        };

        treap.build_cartesian_tree(values);
        treap
    }

    #[inline]
    fn next_priority(&mut self) -> u32 {
        // xorshift32* from
        let mut x = self.rng_state;
        x ^= x >> 13;
        x ^= x << 17;
        x ^= x << 5;
        self.rng_state = x;
        x.wrapping_mul(0x9E37_79BB)
    }

    #[inline]
    fn size(&self, root: u32) -> u32 {
        if root == NODE_NONE {
            0_u32
        } else {
            self.nodes[root as usize].size
        }
    }

    #[inline]
    fn update(&mut self, root: u32) {
        let left_size = self.size(self.nodes[root as usize].left);
        let right_size = self.size(self.nodes[root as usize].right);

        self.nodes[root as usize].size = 1 + left_size + right_size;
    }

    #[inline]
    fn merge(&mut self, mut left: u32, mut right: u32) -> u32 {
        match (left, right) {
            (NODE_NONE, NODE_NONE) => {
                return NODE_NONE;
            }
            (NODE_NONE, _) => {
                self.nodes[right as usize].parent = NODE_NONE;
                return right;
            }
            (_, NODE_NONE) => {
                self.nodes[left as usize].parent = NODE_NONE;
                return left;
            }
            _ => {}
        }

        let mut root = NODE_NONE;
        let mut tail = NODE_NONE;

        // Indicates where the next node should be attached to `tail`.
        let mut attach_right = false;

        while left != NODE_NONE && right != NODE_NONE {
            let node;
            let next_attach_right;

            if self.nodes[left as usize].priority >= self.nodes[right as usize].priority {
                node = left;
                let next = self.nodes[node as usize].right;
                self.nodes[node as usize].right = NODE_NONE;
                left = next;
                next_attach_right = true; // The recursive result belongs in node.right.
            } else {
                node = right;
                let next = self.nodes[node as usize].left;
                self.nodes[node as usize].left = NODE_NONE;
                right = next;
                next_attach_right = false; // The recursive result belongs in node.left.
            }
            // The node may have had a parent in its original treap.
            self.nodes[node as usize].parent = NODE_NONE;
            if tail == NODE_NONE {
                root = node;
            } else {
                if attach_right {
                    self.nodes[tail as usize].right = node;
                } else {
                    self.nodes[tail as usize].left = node;
                }
                self.nodes[node as usize].parent = tail;
            }
            tail = node;
            attach_right = next_attach_right;
        }
        // One treap is exhausted. Attach the remaining subtree at the
        // position where the recursive merge result belongs.
        let remaining = if left != NODE_NONE { left } else { right };
        if attach_right {
            self.nodes[tail as usize].right = remaining;
        } else {
            self.nodes[tail as usize].left = remaining;
        }
        if remaining != NODE_NONE {
            self.nodes[remaining as usize].parent = tail;
        }
        // Recompute sizes from the bottom of the merge path to the root.
        let mut current = tail;
        while current != NODE_NONE {
            let parent = self.nodes[current as usize].parent;
            self.update(current);
            current = parent;
        }
        self.nodes[root as usize].parent = NODE_NONE;
        root
    }

    ///
    /// Find the implicit rank / position of `element` within the left-to-right order.
    ///
    /// Walks from `node` to self.root, adding up the number of elements to its left along the way.
    ///
    ///
    ///
    /// # Examples
    /// ```
    /// let t = crate::heap_unranking::treapheaps::ImplicitTreap::new(2);
    /// assert_eq!(0, t.rank(0), "rank 66");
    /// assert_eq!(1, t.rank(1), "rank 55");
    /// ```
    #[inline]
    pub fn rank(&self, node: u32) -> u32 {
        let mut curr = &self.nodes[node as usize];
        let mut rank = self.size(curr.left);
        while curr.parent != NODE_NONE {
            let parent = &self.nodes[curr.parent as usize];
            if parent.right != NODE_NONE && std::ptr::eq(curr, &self.nodes[parent.right as usize]) {
                rank += parent.size - curr.size;
            }
            curr = parent;
        }
        rank
    }

    const MAX_PIECES: usize = 10;

    /// Extract ranges in old-sequence order. Once a range is extracted,
    /// replace its compose_buf entry with the resulting treap piece.
    #[inline]
    fn extract_ranges_replace_node_singles(
        &mut self,
        prefix: u32,
        prefix_len: u32,
        compose_buf_len: u32,
    ) {
        let mut remaining = prefix;
        for index in 0..compose_buf_len {
            let output = self.pieces[index as usize] as usize;

            let (start, end) = self.compose_buf[output];

            debug_assert!(start < end);
            debug_assert!(end <= prefix_len);

            let (piece, rest) = self.split(remaining, end - start);

            // The range is no longer needed. Store the extracted treap root
            // in its output-order slot.
            self.compose_buf[output] = (piece, NODE_NONE);

            remaining = rest;
        }
    }

    #[inline]
    fn extract_ranges_replace_node_multi(
        &mut self,
        prefix: u32,
        prefix_len: u32,
        compose_buf_len: u32,
    ) {
        let mut lengths = [0u32; Self::MAX_PIECES];
        for old_index in 0..compose_buf_len as usize {
            let output = self.pieces[old_index] as usize;
            let (start, end) = self.compose_buf[output];
            debug_assert!(start < end);
            debug_assert!(end <= prefix_len);
            lengths[old_index] = end - start;
        }
        // Split the prefix into old-sequence pieces.
        let mut piece_roots = [NODE_NONE; Self::MAX_PIECES];
        self.split_many(
            prefix,
            &lengths[..compose_buf_len as usize],
            &mut piece_roots[..compose_buf_len as usize],
        );
        // Put each extracted root into its desired output-order slot.
        for old_index in 0..compose_buf_len as usize {
            let output = self.pieces[old_index] as usize;
            self.compose_buf[output] = (piece_roots[old_index], NODE_NONE);
        }
    }

    fn split_many(&mut self, root: u32, lengths: &[u32], out: &mut [u32]) {
        let k = lengths.len();

        debug_assert!(k > 0);
        debug_assert!(k <= Self::MAX_PIECES);
        debug_assert_eq!(out.len(), k);
        debug_assert!(lengths.iter().all(|&len| len > 0));

        let mut prefix = [0u32; Self::MAX_PIECES + 1];

        for i in 0..k {
            prefix[i + 1] = prefix[i] + lengths[i];
        }

        debug_assert_eq!(prefix[k], self.size(root));

        // Explicit DFS stack. At most k tasks can be pending.
        let mut task_roots = [NODE_NONE; Self::MAX_PIECES];
        let mut task_lo = [0usize; Self::MAX_PIECES];
        let mut task_hi = [0usize; Self::MAX_PIECES];

        let mut stack_len = 1;
        task_roots[0] = root;
        task_hi[0] = k;

        while stack_len != 0 {
            stack_len -= 1;

            let current = task_roots[stack_len];
            let lo = task_lo[stack_len];
            let hi = task_hi[stack_len];

            if hi - lo == 1 {
                out[lo] = current;
                continue;
            }

            // Choose a boundary close to the element midpoint.
            let begin = prefix[lo];
            let end = prefix[hi];
            let target = begin + (end - begin) / 2;

            let mut mid = lo + 1;
            let mut best_distance = prefix[mid].abs_diff(target);

            for candidate in (lo + 2)..hi {
                let distance = prefix[candidate].abs_diff(target);

                if distance < best_distance {
                    best_distance = distance;
                    mid = candidate;
                }
            }

            let split_count = prefix[mid] - begin;
            let (left, right) = self.split(current, split_count);

            debug_assert!(stack_len + 2 <= Self::MAX_PIECES);

            // Push right first so left is processed first.
            task_roots[stack_len] = right;
            task_lo[stack_len] = mid;
            task_hi[stack_len] = hi;
            stack_len += 1;

            task_roots[stack_len] = left;
            task_lo[stack_len] = lo;
            task_hi[stack_len] = mid;
            stack_len += 1;
        }
    }

    ///
    /// Splits `root` before position `count`.
    ///
    /// The left result contains `count` elements, the right contains the rest.
    /// Iterative version using parent pointers to avoid building a stack.
    ///
    fn split(&mut self, root: u32, count: u32) -> (u32, u32) {
        if root == NODE_NONE {
            debug_assert_eq!(count, 0);
            return (NODE_NONE, NODE_NONE);
        }

        debug_assert!(count <= self.size(root));
        debug_assert_eq!(
            self.nodes[root as usize].parent, NODE_NONE,
            "split root must be a treap root",
        );

        let nodes = &mut self.nodes;

        let mut current = root;
        let mut remaining_count = count;

        // The deepest non-null node on the split path, and the direction
        // taken from that node to reach NODE_NONE.
        let mut bottom = NODE_NONE;
        let mut bottom_went_left = false;

        // Descend to find the split path
        while current != NODE_NONE {
            bottom = current;

            let left = nodes[current as usize].left;

            let left_size = if left == NODE_NONE {
                0
            } else {
                nodes[left as usize].size
            };

            if remaining_count <= left_size {
                // The split is in the left subtree.
                bottom_went_left = true;
                current = left;
            } else {
                // The current node belongs to the left result.
                remaining_count -= left_size + 1;

                bottom_went_left = false;
                current = nodes[current as usize].right;
            }
        }

        let mut left_result = NODE_NONE;
        let mut right_result = NODE_NONE;

        let mut node = bottom;
        let mut went_left = bottom_went_left;

        // Walk back up, reconstructing each of the two new treaps
        loop {
            let node_index = node as usize;

            // Save this before changing any links.
            let parent = nodes[node_index].parent;
            let other_branch;
            let new_branch;

            if went_left {
                nodes[node_index].left = right_result;
                new_branch = right_result;
                other_branch = nodes[node as usize].right;
                right_result = node;
            } else {
                nodes[node_index].right = left_result;
                new_branch = left_result;
                other_branch = nodes[node as usize].left;
                left_result = node;
            }

            let new_branch_size = if new_branch != NODE_NONE {
                // Fix the parent pointer and return the size:
                nodes[new_branch as usize].parent = node;
                nodes[new_branch as usize].size
            } else {
                0
            };
            let other_branch_size = if other_branch != NODE_NONE {
                nodes[other_branch as usize].size
            } else {
                0
            };
            nodes[node as usize].size = 1 + new_branch_size + other_branch_size;

            if parent == NODE_NONE {
                break;
            }

            // `node` is still the original child of `parent`; its parent
            // pointer has not been changed. Determine which recursive case
            // applies when processing `parent`.
            went_left = nodes[parent as usize].left == node;
            node = parent;
        }

        // Fix up `parent` for both the returned nodes:
        if left_result != NODE_NONE {
            nodes[left_result as usize].parent = NODE_NONE;
        }

        if right_result != NODE_NONE {
            nodes[right_result as usize].parent = NODE_NONE;
        }

        (left_result, right_result)
    }

    /// Reorders a prefix according to a partition of the old prefix.
    ///
    /// `self.compose_buf` describes the ranges of the old sequence in their
    /// desired output order. The ranges must be nonempty, disjoint, and
    /// cover exactly the prefix being reordered.
    ///
    /// The big thing here is if that this get called ~n times, so if
    /// is [`reorder_prefix`] is faster than O(n), that should win out.
    /// The theory is that it should be O(log n) on avg in terms of output indices.
    /// That seems true if output_ranges.len() is about log n.
    fn reorder_prefix2(&mut self, prefix_len: u32, compose_buf_len: u32) {
        debug_assert!(compose_buf_len > 0);

        let (prefix, suffix) = if prefix_len == self.nodes.len() as u32 {
            (self.root, NODE_NONE)
        } else {
            self.split(self.root, prefix_len)
        };

        // self.pieces temporarily stores output indices, sorted by their
        // position in the old sequence.
        self.pieces.clear();
        self.pieces.extend(0..compose_buf_len);

        let compose_buf = &self.compose_buf;

        self.pieces
            .sort_unstable_by_key(|&output| compose_buf[output as usize].0);

        // TODO: benchmark which one of these is more efficient:
        self.extract_ranges_replace_node_multi(prefix, prefix_len, compose_buf_len);
        //self.extract_ranges_replace_node_singles(prefix, prefix_len, compose_buf_len);

        /*
        let mut reordered = self.compose_buf[0].0.into();
            for output in 1..compose_buf_len as usize {
                reordered = self.merge(reordered, self.compose_buf[output].0);
        }
         */
        let reordered = self.merge_multi(compose_buf_len as usize);

        self.root = if suffix == NODE_NONE {
            reordered
        } else {
            self.merge(reordered, suffix)
        }
    }

    fn merge_multi(&mut self, mut len: usize) -> u32 {
        debug_assert!(len != 0);

        while len > 1 {
            let mut read = 0;
            let mut write = 0;

            while read + 1 < len {
                self.compose_buf[write].0 =
                    self.merge(self.compose_buf[read].0, self.compose_buf[read + 1].0);

                read += 2;
                write += 1;
            }

            // Carry an unpaired final root into the next round.
            if read < len {
                self.compose_buf[write].0 = self.compose_buf[read].0;
                write += 1;
            }

            len = write;
        }

        self.compose_buf[0].0
    }

    /// Visit in left-to-right order
    #[inline]
    fn into_indices(&self, mut visit: impl FnMut(usize, usize)) {
        let mut ctr = 0;

        let mut current = self.root;
        while current != NODE_NONE && self.nodes[current as usize].left != NODE_NONE {
            current = self.nodes[current as usize].left;
        }

        while current != NODE_NONE {
            visit(ctr, current as usize);
            ctr += 1;

            // Find next node in in-order traversal
            if self.nodes[current as usize].right != NODE_NONE {
                // Right subtree exists: go to leftmost node
                current = self.nodes[current as usize].right;
                while self.nodes[current as usize].left != NODE_NONE {
                    current = self.nodes[current as usize].left;
                }
            } else {
                // No right subtree: backtrack to first ancestor where current is in left subtree
                let mut node = current;
                current = self.nodes[node as usize].parent;
                while current != NODE_NONE && self.nodes[current as usize].right == node {
                    node = current;
                    current = self.nodes[node as usize].parent;
                }
            }
        }
    }

    /// Doesn't seem worth it
    fn compose_reorders_binary_search(&mut self, prefix_len: u32, first: &[(u32, u32)]) {
        //assert!(prefix_len < 20);
        assert!(first.len() <= 5);
        assert!(self.ranges.len() <= 6);
        let result = &mut self.compose_buf;

        let mut result_len = 0;

        /* llm */
        // `first` is ordered by position in the intermediate sequence.
        // Its source ranges do not need to be sorted or non-overlapping.

        self.pieces.clear();
        let intermediate_ends = &mut self.pieces;
        let mut intermediate_pos = 0;

        for (i, &(old_start, old_end)) in first.iter().enumerate() {
            debug_assert!(
                old_start < old_end,
                "invalid first range at index {i}: ({old_start}, {old_end})"
            );

            intermediate_pos += old_end - old_start;
            intermediate_ends.push(intermediate_pos);
        }

        debug_assert_eq!(
            intermediate_pos, prefix_len,
            "first ranges cover {intermediate_pos} intermediate elements, \
	     but prefix_len is {prefix_len}"
        );

        // `self.ranges` is intentionally processed in its original order.
        // Do not sort it.
        for (second_index, &(second_start, second_end)) in self.ranges.iter().enumerate() {
            debug_assert!(
                second_start < second_end,
                "invalid second range at index {second_index}: \
		 ({second_start}, {second_end})"
            );

            debug_assert!(
                second_end <= prefix_len,
                "second range at index {second_index} exceeds prefix_len: \
		 ({second_start}, {second_end}), prefix_len = {prefix_len}"
            );

            // Find the first first-stage piece whose intermediate end is
            // greater than second_start.
            //
            // Pieces ending exactly at second_start cannot overlap this range.
            let mut first_index = intermediate_ends
                .partition_point(|&intermediate_end| intermediate_end <= second_start);

            let mut intermediate_start = if first_index == 0 {
                0
            } else {
                intermediate_ends[first_index - 1]
            };

            while first_index < first.len() && intermediate_start < second_end {
                let (old_start, _) = first[first_index];
                let intermediate_end = intermediate_ends[first_index];

                let overlap_start = second_start.max(intermediate_start);
                let overlap_end = second_end.min(intermediate_end);

                if overlap_start < overlap_end {
                    let offset_start = overlap_start - intermediate_start;
                    let offset_end = overlap_end - intermediate_start;

                    let mapped_start = old_start + offset_start;
                    let mapped_end = old_start + offset_end;

                    debug_assert!(
                        result_len < result.len(),
                        "result buffer is too small while processing \
			 second range at index {second_index}"
                    );

                    // Merge adjacent ranges when possible.
                    if result_len > 0 {
                        let previous_end = result[result_len - 1].1;

                        if previous_end == mapped_start {
                            result[result_len - 1].1 = mapped_end;
                        } else {
                            result[result_len] = (mapped_start, mapped_end);
                            result_len += 1;
                        }
                    } else {
                        result[result_len] = (mapped_start, mapped_end);
                        result_len += 1;
                    }
                }

                intermediate_start = intermediate_end;
                first_index += 1;
            }
        }

        /* llm */

        self.reorder_prefix2(prefix_len, result_len as u32);
    }

    /// Compose two interval permutations, and reorder the prefixes.
    ///
    /// `first` describes an intermediate sequence in terms of the old sequence.
    /// `second` describes the final sequence in terms of the intermediate sequence.
    ///
    /// The resulting ranges describe the final sequence in terms of the old
    /// sequence, which allows the two transformations to be replaced by one
    /// reorder_prefix call. These ranges are passed to [`self.reorder_prefix`]
    fn compose_reorders(&mut self, prefix_len: u32, first: &[(u32, u32)]) {
        debug_assert!(!first.is_empty());
        debug_assert!(!self.ranges.is_empty()); // second
        debug_assert!(first.len() <= 5);
        debug_assert!(self.ranges.len() <= 6);

        let result = &mut self.compose_buf;
        let mut result_len = 0;

        for &(second_start, second_end) in self.ranges.iter() {
            debug_assert!(second_start < second_end);
            debug_assert!(second_end <= prefix_len);

            // `intermediate_pos` is the position occupied by the current
            // first-stage range in the intermediate sequence.
            let mut intermediate_pos = 0;

            for &(old_start, old_end) in first {
                let intermediate_end = intermediate_pos + (old_end - old_start);

                let overlap_start = second_start.max(intermediate_pos);
                let overlap_end = second_end.min(intermediate_end);

                if overlap_start < overlap_end {
                    let mapped_start = old_start + (overlap_start - intermediate_pos);
                    let mapped_end = old_start + (overlap_end - intermediate_pos);

                    if result_len > 0 && result[result_len - 1].1 == mapped_start {
                        // next one starts where the previous ends, so merge:
                        result[result_len - 1].1 = mapped_end;
                    } else {
                        result[result_len] = (mapped_start, mapped_end);
                        result_len += 1;
                    }
                }

                intermediate_pos = intermediate_end;

                if intermediate_pos >= second_end {
                    break;
                }
            }
        }

        self.reorder_prefix2(prefix_len, result_len as u32);
    }

    /// Applies the odd-n operation to the prefix `0..=n`.
    #[inline]
    fn forward_odd(&mut self, n: u32, q: u32) {
        debug_assert!(n & 1 == 1);
        //debug_assert!(q > 0 && q <= n);

        let prefix_len: u32 = n as u32 + 1;

        if n == 1 {
            // q must be 1 because 0 < q <= n
            // swap(1, 0)
            // swap(0, 0)
            self.compose_buf[0] = (1, 2);
            self.compose_buf[1] = (0, 1);
            self.reorder_prefix2(prefix_len, 2);
            return;
        }

        // State after:
        //
        // permutation.swap(n, n - 1);
        // if q is odd {
        //     permutation.swap(0, n - 1);
        // }
        //
        // We represent this state as ranges of the old sequence.

        if q < 2 {
            // 0 < q < 2 means q == 1, so the q & 1 == 1 case from below always matches.
            self.compose_buf[..4].copy_from_slice(&[(n, n + 1), (1, n - 1), (0, 1), (n - 1, n)]);
            self.reorder_prefix2(prefix_len, 4);
            return;
        }

        self.ranges.clear();
        self.ranges.push((0, 1)); // S[0]
        self.ranges.push((n, n + 1)); // S[n]
        // q >= 2 so q-1 >= 2-1, so if q==2 we need to skip:
        push_range(&mut self.ranges, 1, q - 1); // S[1..q-1] (copy_within)
        push_range(&mut self.ranges, q, n); // S[q..n]
        self.ranges.push((q - 1, q)); // S[q-1]

        if q & 1 == 1 {
            self.compose_reorders(prefix_len, &[(n, n + 1), (1, n - 1), (0, 1), (n - 1, n)]);
        } else {
            self.compose_reorders(prefix_len, &[(0, n - 1), (n, n + 1), (n - 1, n)]);
        }
    }

    /// Applies the even-n operation to the prefix `0..=n`.
    #[inline]
    fn forward_even(&mut self, n: u32, q: u32) {
        debug_assert!(n & 1 == 0);
        debug_assert!(q > 0 && q <= n);

        let prefix_len: u32 = n as u32 + 1;

        if n == 2 {
            // The special case is:
            //
            // permutation.swap(0, 1 + (q & 1));
            // permutation.swap(1, 2);
            if q & 1 == 1 {
                self.compose_buf[..3].copy_from_slice(&[(2, 3), (0, 1), (1, 2)]);
            } else {
                self.compose_buf[..3].copy_from_slice(&[(1, 2), (2, 3), (0, 1)]);
            }

            self.reorder_prefix2(prefix_len, 3);
            return;
        }

        let len = n + 1;

        let start = len + 3 - q - len * (q <= 3) as u32;
        let pivot = (len - start).min(n - 3);
        let last_minus_1 = len + 1 - q - len * (q == 1) as u32;
        let last_minus_2 = len + 2 - q - len * (q <= 2) as u32;

        self.ranges.clear();
        self.ranges.push((len - q, len - q + 1));
        debug_assert_ne!(pivot, 0);
        push_range(&mut self.ranges, start, start + pivot);
        push_range(&mut self.ranges, 0, n - 3 - pivot);
        self.ranges.push((last_minus_2, last_minus_2 + 1));
        self.ranges.push((last_minus_1, last_minus_1 + 1));
        self.ranges.push((n - q, n - q + 1));

        self.compose_reorders(
            prefix_len,
            &[(0, 1), (n - 1, n), (n - 2, n - 1), (1, n - 2), (n, n + 1)],
        );
    }
}

///
/// # Examples
/// ```
/// # use heap_unranking::forward_by_q;
/// # use heap_unranking::treapheaps::forward_by_qs;
/// for n in 2..20 {
///   let mut even_tmp = Vec::with_capacity(n);
///   for q in 1..n {
///     let mut perm1 = (0..n).collect::<Vec<_>>();
///     let mut perm2 = perm1.clone();
///     forward_by_q(n-1, q, &mut even_tmp, &mut perm1);
///     let qs:Vec<usize> = vec![0usize; n-2].into_iter().chain(std::iter::once(q)).collect();
///     forward_by_qs(&qs, &mut even_tmp, &mut perm2);
///     assert_eq!(perm1, perm2);
///   }
///   {
///     let mut perm1 = (0..n).collect::<Vec<_>>();
///     let mut perm2 = perm1.clone();
///     let qs = (1..n).collect::<Vec<_>>();
///
///     forward_by_qs(&qs, &mut even_tmp, &mut perm2);
///
///     // forward_by_qs() is equivalent to:
///     for (n, q) in (1..perm1.len())
///        .zip(qs)
///        .rev()
///        .filter(|(_, q)| *q != 0)
///     {
///        forward_by_q(n, q, &mut even_tmp, &mut perm1);
///     }
///
///     assert_eq!(perm1, perm2);
///   }
/// }
/// ```
pub fn forward_by_qs<E: Copy>(qs: &[usize], even_tmp: &mut Vec<E>, permutation: &mut [E]) {
    let length = permutation.len();

    if length <= 1 {
        return;
    }

    let operation_count = qs.len().min(length - 1);

    let mut source_treap: ImplicitTreap = ImplicitTreap::new(length);

    for offset in (0..operation_count).rev() {
        let q = qs[offset];

        if q == 0 {
            continue;
        }
        let n = offset + 1;

        debug_assert!(q <= n);

        if n & 1 == 1 {
            source_treap.forward_odd(n as u32, q as u32);
        } else {
            source_treap.forward_even(n as u32, q as u32);
        }
    }

    even_tmp.clear();
    even_tmp.extend_from_slice(permutation);

    source_treap.into_indices(|dst, src| {
        permutation[dst] = even_tmp[src];
    });
}

///
/// Reference impl of forward_by_q in a loop.
/// Should probably live in lib.rs.
///
pub fn reference_forward_by_qs<E: Copy>(
    qs: &[usize],
    mut even_tmp: &mut Vec<E>,
    permutation: &mut [E],
) {
    for (i, &q) in qs.iter().enumerate().rev() {
        if q == 0 {
            continue;
        }
        crate::forward_by_q(i + 1, q, &mut even_tmp, permutation);
    }
}

///
/// Unrank a the permutation indices at rank `k` for an array of length `n`
/// given the "identity" indices `(0..n)` in $O(n log n)$.
///
/// # Examples
/// ```rust
/// # use heap_unranking::treapheaps::unrank_treap;
/// assert_eq!([0, 1, 2, 3], unrank_treap(0..4, 0)[..]);
/// assert_eq!([0, 2, 1, 3], unrank_treap(0..4, 3)[..]);
/// ```
pub fn unrank_treap<R, E, K>(identity: R, mut k: K) -> Box<[E]>
where
    R: IntoIterator<Item = E>,
    E: std::marker::Copy,
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

    forward_by_qs(&qs, &mut even_tmp, &mut permutation);

    permutation
}

///
/// Implicit Treap-based variant of [`rank_noprecomp_gen`]. $O(n \log{n})$.
///
/// ## Original
/// The original [`forward_by_q`]-based implementation consists of a $n$-bound
/// loop over the factoradic digits,
/// and for each digit it:
/// - Looks for the index with `position()`, in $O(n)$.
/// - Calculates `q`, $O(1)$.
/// - Transpose `permutation` with `forward_by_q` using the `q`, in $O(n)$.
///
/// ## Implicit Treap
/// For each factoradic digit (of which there are `n`):
/// - `position`: In the original function it is acceptable to index in $O(n)$ because
///    overall runtime is governed by transposition which is also $O(n)$, and with more
///    expensive constants. However, when  `permutation` is already represented using
///    an Implicit Treap, we can replace the `position()` call with an `ImplicitTreap::rank` lookup
///    to find the implicit index of the element in $O(\log{n})$.
/// - The `q` calculation from the index remains the same.
/// - The per-`q` transposition step is $O(\log{n}).
///
/// # Examples
///
/// ```rust
/// # use heap_unranking::treapheaps::rank_treap;
/// assert_eq!(0_usize, rank_treap([0], &[0_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(0_usize, rank_noprecomp_gen([0,1], &[0,1]));
/// assert_eq!(0_usize, rank_treap([0,1], &[0,1_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// assert_eq!(0_usize, rank_treap([0,1,2], &[0,1,2_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(0_usize, rank_noprecomp_gen([0,1,2,3], &[0,1,2,3]));
/// assert_eq!(0_usize, rank_treap([0,1,2,3], &[0,1,2,3_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(0_usize, rank_noprecomp_gen([0,1,2,3,4], &[0,1,2,3,4]));
/// assert_eq!(0_usize, rank_treap([0,1,2,3,4], &[0,1,2,3,4_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(0_usize, rank_noprecomp_gen([0,1,2,3,4,5], &[0,1,2,3,4,5]));
/// assert_eq!(0_usize, rank_treap([0,1,2,3,4,5], &[0,1,2,3,4,5_u32]));
/// ```
/// ```
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(0_usize, rank_noprecomp_gen([0,1,2,3,4,5,6], &[0,1,2,3,4,5,6]));
/// assert_eq!(0_usize, rank_treap([0,1,2,3,4,5,6], &[0,1,2,3,4,5,6_u32]));
/// ```
///
/// ```rust
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(57_usize, rank_noprecomp_gen([0,1,2,3,4], &[4, 1, 3, 0, 2]), "minim");
/// assert_eq!(57_usize, rank_treap([0,1,2,3,4], &[4, 1, 3, 0, 2_u32]));
/// ```
///
/// assert_eq!(100,200);
/// assert_eq!(123_usize, rank_noprecomp_gen([0,1,2,3,4,5], &[5, 2, 1, 3, 0, 4]));
/// assert_eq!(123_usize, rank_treap([0,1,2,3,4,5], &[5, 2, 1, 3, 0, 4]));
/// assert_eq!(1_usize, rank_treap([0,1], &[1,0]));
/// assert_eq!(1_usize, rank_treap([0,1,2], &[1,0,2]));
/// assert_eq!(1_usize, rank_treap([0,1,2,3], &[1,0,2,3]));
/// assert_eq!(1_usize, rank_treap([0,1,2,3,4], &[1,0,2,3,4]));
/// assert_eq!(1_usize, rank_treap([0,1,2,3,4,5], &[1,0,2,3,4,5]));
///
///
/// ```rust
/// # use heap_unranking::treapheaps::rank_treap;
/// # use heap_unranking::rank_noprecomp_gen;
/// assert_eq!(5_usize, rank_noprecomp_gen([0,1,2], &[2,1,0]));
/// assert_eq!(5_usize, rank_treap([0,1,2], &[2,1,0_u32]));
/// ```
///
/// ```rust
/// # use heap_unranking::treapheaps::rank_treap;
/// assert_ne!(0_usize, rank_treap([0,1,2,3], &[2,3,1,0_u32]));
/// ```
pub fn rank_treap<R, E, K>(identity: R, permutation: &[E]) -> K
where
    R: IntoIterator<Item = E>,
    E: std::marker::Copy + std::cmp::PartialEq,
    K: for<'a> std::ops::MulAssign<&'a K>
        + std::ops::AddAssign<usize>
        + std::ops::AddAssign<K>
        + std::convert::From<usize>
        + num_traits::Zero
        + num_traits::ConstOne,
    u32: From<E>,
    E: std::fmt::Debug,
{
    if permutation.len() <= 1 {
        return K::zero();
    }
    let mut qs: Box<[usize]> = vec![0; permutation.len() - 1].into_boxed_slice();

    /*
    use std::collections::HashMap;
    use std::hash::Hash;
    let permutation_i_to_identity : HashMap<E, usize, _> =
    identity.iter()
            .enumerate()
            .map(|(position, value)| (value, position))
        .collect();
    */

    let mut treap: ImplicitTreap = ImplicitTreap::new(
        permutation.len(),
        //&(0..permutation.len()).collect::<Vec<_>>()[..], // TODO should be putting the `identity` in here.
        // identity.into_iter().collect();
    );
    for (q_ptr, (i, &permutation_i)) in qs
        .iter_mut()
        .zip(permutation.iter().enumerate().skip(1))
        .rev()
    {
        let search: u32 = permutation_i.into();
        // `search` essentially should be:
        // (identity.position(permutation_i) as usize)
        // which can be done with O(n) preprocessing, and O(1) here,
        // but I haven't implemented that, so for now we do permutation_i.into()
        let idx = treap.rank(search); // O(log n)
        debug_assert_ne!(idx, NODE_NONE);
        if idx == i as u32 {
            // q := 0 : no swaps required to make 0..=i have a suffix of permutation[i]
            continue;
        }
        let q: u32 = {
            let i = i as u32;
            if idx == 0 {
                i
            } else if (i & 1) == 1 {
                if idx + 1 == i { 1 } else { idx + 1 }
            } else {
                if i - idx > 2 { i - idx - 2 } else { idx }
            }
        };
        *q_ptr = q as usize;

        // Forward the prefix permutation by `q` in O(log n):
        if i & 1 == 1 {
            debug_assert!(q as usize <= i);
            treap.forward_odd(i as u32, q);
        } else {
            debug_assert!(q as usize <= i);
            treap.forward_even(i as u32, q);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn next_random(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *state = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    #[test]
    fn treap_matches_reference_for_hand_cases() {
        let cases: &[&[usize]] = &[
            &[],
            &[0],
            &[1],
            &[0, 1],
            &[1, 1],
            &[1, 2],
            &[0, 2, 1],
            &[1, 2, 3],
            &[1, 2, 3, 4],
            &[1, 2, 3, 4, 5],
            &[1, 2, 3, 4, 5, 6],
            &[1, 2, 3, 4, 5, 6, 7],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 2, 3, 4, 5, 6, 7, 8, 9],
            &[1, 2, 0, 0, 0, 0, 0, 0, 0],
            &[1, 2, 0, 0, 0, 0, 0, 1, 0],
            &[0, 0, 0, 0, 5, 4, 3, 2, 1],
            &[1, 2, 3, 4, 5, 4, 3, 2, 1],
            &[0, 1, 0, 3, 1],
            &[1, 2, 3, 4, 4, 3, 7, 8],
            &[0, 2, 3, 4, 4, 3, 7, 8],
            &[0, 2, 3, 4, 0, 0, 7, 0],
            &[0, 2, 0, 0, 0, 0, 5, 0],
        ];

        let mut even_tmp =
            Vec::with_capacity(cases.iter().map(|case| case.len()).max().unwrap() + 1);

        for &qs in cases {
            let length = qs.len() + 1;

            let original: Vec<usize> = (0..length).map(|x| x * 17 + 3).collect();

            let mut expected = original.clone();
            even_tmp.clear();
            reference_forward_by_qs(qs, &mut even_tmp, &mut expected);

            let mut actual = original.clone();
            even_tmp.clear();
            forward_by_qs(qs, &mut even_tmp, &mut actual);

            assert_eq!(actual, expected, "mismatch for qs={qs:?}");
        }
    }

    #[test]
    fn treap_matches_reference_randomly() {
        let mut rng = 0x1234_5678_9abc_def0;

        for length in 2..=100 {
            let mut even_tmp = Vec::with_capacity(length + 1);

            for _case in 0..200 {
                let mut qs = vec![0; length - 1];

                for (offset, q) in qs.iter_mut().enumerate() {
                    let n = offset + 1;
                    let random = next_random(&mut rng);

                    // Include zero frequently so that skipped operations
                    // are tested as well.
                    *q = if random % 5 == 0 {
                        0
                    } else {
                        (random as usize % n) + 1
                    };
                }

                let original: Vec<usize> = (0..length)
                    .map(|i| {
                        let random = next_random(&mut rng);
                        i * 1_000_003 + random as usize
                    })
                    .collect();

                let mut expected = original.clone();
                even_tmp.clear();
                reference_forward_by_qs(&qs, &mut even_tmp, &mut expected);

                let mut actual = original.clone();
                even_tmp.clear();
                forward_by_qs(&qs, &mut even_tmp, &mut actual);

                assert_eq!(actual, expected, "mismatch for length={length}, qs={qs:?}");
            }
        }
    }

    #[test]
    fn treap_preserves_all_values() {
        let length = 128;

        let qs: Vec<usize> = (0..length - 1)
            .map(|offset| {
                let n = offset + 1;
                if n % 7 == 0 { 0 } else { n }
            })
            .collect();

        let original: Vec<usize> = (0..length).map(|x| x * 31 + 11).collect();

        let mut actual = original.clone();
        let mut even_tmp = Vec::new();

        forward_by_qs(&qs, &mut even_tmp, &mut actual);

        let mut sorted_actual = actual.clone();
        sorted_actual.sort_unstable();

        let mut sorted_original = original.clone();
        sorted_original.sort_unstable();

        assert_eq!(sorted_actual, sorted_original);
    }

    // should have a test for rank()
}
