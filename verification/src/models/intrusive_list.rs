//! Pointer-free abstract model for `tokio::util::linked_list`.
//!
//! This proves structural invariants of the abstract push/pop/remove state
//! machine. It deliberately does **not** claim pointer-provenance or semantic
//! equivalence with Tokio's unsafe implementation; those require additional
//! refinement/code-level proofs.

use crate::{invariant_model, kani_harness};

const MAX_NODES: usize = 4;

invariant_model! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct IntrusiveListModel {
        pub head: Option<usize>,
        pub tail: Option<usize>,
        pub prev: [Option<usize>; MAX_NODES],
        pub next: [Option<usize>; MAX_NODES],
        pub present: [bool; MAX_NODES],
        pub len: usize,
    }
    invariants {
        structure => |s: &IntrusiveListModel| s.structural_invariant(),
    }
}

impl IntrusiveListModel {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            prev: [None; MAX_NODES],
            next: [None; MAX_NODES],
            present: [false; MAX_NODES],
            len: 0,
        }
    }

    pub fn push_front(&mut self, node: usize) -> bool {
        if node >= MAX_NODES || self.present[node] {
            return false;
        }

        self.present[node] = true;
        self.prev[node] = None;
        self.next[node] = self.head;

        if let Some(old_head) = self.head {
            self.prev[old_head] = Some(node);
        } else {
            self.tail = Some(node);
        }

        self.head = Some(node);
        self.len += 1;
        true
    }

    pub fn pop_front(&mut self) -> Option<usize> {
        let node = self.head?;
        let next = self.next[node];
        self.head = next;

        if let Some(next) = next {
            self.prev[next] = None;
        } else {
            self.tail = None;
        }

        self.prev[node] = None;
        self.next[node] = None;
        self.present[node] = false;
        self.len -= 1;
        Some(node)
    }

    pub fn remove(&mut self, node: usize) -> bool {
        if node >= MAX_NODES || !self.present[node] {
            return false;
        }

        let prev = self.prev[node];
        let next = self.next[node];

        if let Some(prev) = prev {
            self.next[prev] = next;
        } else {
            self.head = next;
        }

        if let Some(next) = next {
            self.prev[next] = prev;
        } else {
            self.tail = prev;
        }

        self.prev[node] = None;
        self.next[node] = None;
        self.present[node] = false;
        self.len -= 1;
        true
    }

    fn structural_invariant(&self) -> bool {
        if self.len > MAX_NODES {
            return false;
        }
        if (self.head.is_none() || self.tail.is_none()) != (self.len == 0) {
            return false;
        }

        let mut present_count = 0;
        let mut i = 0;
        while i < MAX_NODES {
            if self.present[i] {
                present_count += 1;

                if let Some(prev) = self.prev[i] {
                    if prev >= MAX_NODES || !self.present[prev] || self.next[prev] != Some(i) {
                        return false;
                    }
                }
                if let Some(next) = self.next[i] {
                    if next >= MAX_NODES || !self.present[next] || self.prev[next] != Some(i) {
                        return false;
                    }
                }
            } else if self.prev[i].is_some() || self.next[i].is_some() {
                return false;
            }
            i += 1;
        }
        if present_count != self.len {
            return false;
        }

        if let Some(head) = self.head {
            if !self.present[head] || self.prev[head].is_some() {
                return false;
            }
        }
        if let Some(tail) = self.tail {
            if !self.present[tail] || self.next[tail].is_some() {
                return false;
            }
        }

        let mut seen = [false; MAX_NODES];
        let mut cursor = self.head;
        let mut reached = 0;
        let mut last = None;
        while let Some(node) = cursor {
            if node >= MAX_NODES || seen[node] || !self.present[node] {
                return false;
            }
            seen[node] = true;
            reached += 1;
            if reached > MAX_NODES {
                return false;
            }
            last = Some(node);
            cursor = self.next[node];
        }

        reached == self.len && last == self.tail
    }
}

impl Default for IntrusiveListModel {
    fn default() -> Self {
        Self::new()
    }
}

// TOKIO_PROOF: linked-list-model-kani
kani_harness!("linked-list-model-kani", arbitrary_operations_preserve_structure, {
    let mut model = IntrusiveListModel::new();
    let operations: [u8; 6] = kani::any();

    let mut i = 0;
    while i < operations.len() {
        let op = operations[i];
        let node = (op as usize) % MAX_NODES;
        match (op / MAX_NODES as u8) % 3 {
            0 => {
                model.push_front(node);
            }
            1 => {
                model.pop_front();
            }
            _ => {
                model.remove(node);
            }
        }
        assert!(model.is_valid());
        i += 1;
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_obligation;

    // TOKIO_PROOF: linked-list-model-test
    #[test]
    fn exhaustive_short_sequences_preserve_structure() {
        proof_obligation!("linked-list-model-test");

        for a in 0u8..12 {
            for b in 0u8..12 {
                for c in 0u8..12 {
                    let mut model = IntrusiveListModel::new();
                    for op in [a, b, c] {
                        let node = (op as usize) % MAX_NODES;
                        match (op / MAX_NODES as u8) % 3 {
                            0 => {
                                model.push_front(node);
                            }
                            1 => {
                                model.pop_front();
                            }
                            _ => {
                                model.remove(node);
                            }
                        }
                        model.assert_valid();
                    }
                }
            }
        }
    }
}
