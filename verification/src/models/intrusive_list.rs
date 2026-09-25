//! Pointer-free abstract model for the recurring intrusive-list proof shape.
//!
//! Assurance is intentionally abstract-model: this does not prove pointer
//! provenance or equivalence to Tokio's unsafe implementation.

use crate::{invariant_model, kani_sequence_proof, proof_family, ProofState};

const MAX_NODES: usize = 4;

invariant_model! {
    #[derive(Clone, Debug, PartialEq, Eq, ProofState)]
    pub struct IntrusiveListModel {
        head: Option<usize>,
        tail: Option<usize>,
        prev: [Option<usize>; MAX_NODES],
        next: [Option<usize>; MAX_NODES],
        present: [bool; MAX_NODES],
        len: usize,
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

    #[proof_family(
        id = "intrusive-list-transition-family",
        backend = "kani",
        assurance = "abstract-model"
    )]
    pub fn apply(&mut self, operation: u8) {
        let node = operation as usize % MAX_NODES;
        match (operation / MAX_NODES as u8) % 3 {
            0 => {
                self.push_front(node);
            }
            1 => {
                self.pop_front();
            }
            _ => {
                self.remove(node);
            }
        }
    }

    fn push_front(&mut self, node: usize) -> bool {
        if self.present[node] {
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

    fn pop_front(&mut self) -> Option<usize> {
        let node = self.head?;
        self.head = self.next[node];
        if let Some(next) = self.head {
            self.prev[next] = None;
        } else {
            self.tail = None;
        }
        self.detach(node);
        Some(node)
    }

    fn remove(&mut self, node: usize) -> bool {
        if !self.present[node] {
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
        self.detach(node);
        true
    }

    fn detach(&mut self, node: usize) {
        self.prev[node] = None;
        self.next[node] = None;
        self.present[node] = false;
        self.len -= 1;
    }

    fn structural_invariant(&self) -> bool {
        if self.len > MAX_NODES {
            return false;
        }
        if (self.head.is_none() || self.tail.is_none()) != (self.len == 0) {
            return false;
        }

        let mut present_count = 0usize;
        let mut index = 0usize;
        while index < MAX_NODES {
            if self.present[index] {
                present_count += 1;
                if let Some(prev) = self.prev[index] {
                    if prev >= MAX_NODES
                        || !self.present[prev]
                        || self.next[prev] != Some(index)
                    {
                        return false;
                    }
                }
                if let Some(next) = self.next[index] {
                    if next >= MAX_NODES
                        || !self.present[next]
                        || self.prev[next] != Some(index)
                    {
                        return false;
                    }
                }
            } else if self.prev[index].is_some() || self.next[index].is_some() {
                return false;
            }
            index += 1;
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
        let mut reached = 0usize;
        let mut last = None;
        while let Some(node) = cursor {
            if node >= MAX_NODES || seen[node] || !self.present[node] {
                return false;
            }
            seen[node] = true;
            reached += 1;
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
kani_sequence_proof!(
    id = "linked-list-model-kani",
    harness = linked_list_sequence_preserves_invariant,
    steps = 6,
    init = IntrusiveListModel::new()
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InvariantCheck;

    #[test]
    fn concrete_smoke_sequence_preserves_invariant() {
        let mut model = IntrusiveListModel::new();
        for operation in [0, 1, 4, 8, 2, 6, 9, 5] {
            model.apply(operation);
            assert!(model.invariant_holds());
        }
    }
}
