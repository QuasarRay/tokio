#![forbid(unsafe_code)]
//! Reusable verification and metaverification primitives for Tokio.
//!
//! This is a nested workspace on purpose: Tokio's production workspace and
//! production dependency graph remain unchanged.

pub mod registry;
pub mod patterns;
pub mod models;

pub trait InvariantCheck {
    fn invariant_ok(&self) -> bool;
}

pub trait ProofFamily {
    const PROOF_FAMILY: &'static str;
}

/// Define an abstract state model and generate invariant boilerplate once.
///
/// Backends added by later stacked PRs reuse these generated methods rather
/// than duplicating invariant definitions.
#[macro_export]
macro_rules! invariant_model {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $ty:ty),* $(,)?
        }
        invariants {
            $($inv:ident => $predicate:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field_vis $field: $ty),*
        }

        impl $name {
            pub fn invariant_results(&self) -> ::std::vec::Vec<(&'static str, bool)> {
                ::std::vec![
                    $((::std::stringify!($inv), ($predicate)(self))),+
                ]
            }

            pub fn is_valid(&self) -> bool {
                true $(&& ($predicate)(self))+
            }

            pub fn assert_valid(&self) {
                $(assert!(
                    ($predicate)(self),
                    "verification invariant `{}` failed",
                    ::std::stringify!($inv)
                );)+
            }
        }

        impl $crate::InvariantCheck for $name {
            fn invariant_ok(&self) -> bool {
                self.is_valid()
            }
        }
    };
}

/// Embed a stable proof-obligation identifier in generated proof code.
#[macro_export]
macro_rules! proof_obligation {
    ($id:literal) => {
        const _: &str = $id;
    };
}

#[cfg(test)]
mod tests {
    invariant_model! {
        struct CounterModel {
            value: usize,
            limit: usize,
        }
        invariants {
            bounded => |s: &CounterModel| s.value <= s.limit,
        }
    }

    #[test]
    fn invariant_macro_generates_shared_checker() {
        let valid = CounterModel { value: 2, limit: 3 };
        let invalid = CounterModel { value: 4, limit: 3 };
        assert!(valid.is_valid());
        valid.assert_valid();
        assert!(!invalid.is_valid());
    }
}
