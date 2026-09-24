//! Pointer-free abstract model of Tokio's intrusive linked-list transitions.

use crate::{invariant_model, InvariantCheck, ProofFamily};
use tokio_verification_macros::{kani_proof, DelegateInvariant, ProofFamily};

const MAX_NODES: usize = 4;

invariant_model! {
    #[derive(Clone, Debug, PartialEq, Eq, ProofFamily)]
    #[proof_family(name = "intrusive-list")]
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

#[derive(DelegateInvariant)]
pub struct ListProofEnvelope {
    #[invariant_delegate]
    model: IntrusiveListModel,
    family: &'static str,
}

impl ListProofEnvelope {
    fn new(model: IntrusiveListModel) -> Self {
        Self {
            model,
            family: IntrusiveListModel::PROOF_FAMILY,
        }
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
#[kani_proof(id = "linked-list-model-kani", unwind = 12)]
pub fn intrusive_list_operations_preserve_invariants() {
    let operations: [u8; 6] = kani::any();
    let mut envelope = ListProofEnvelope::new(IntrusiveListModel::new());

    let mut i = 0;
    while i < operations.len() {
        let op = operations[i];
        let node = (op as usize) % MAX_NODES;

        match (op / MAX_NODES as u8) % 3 {
            0 => {
                envelope.model.push_front(node);
            }
            1 => {
                envelope.model.pop_front();
            }
            _ => {
                envelope.model.remove(node);
            }
        }

        assert!(envelope.invariant_ok());
        assert_eq!(envelope.family, "intrusive-list");
        i += 1;
    }
}
