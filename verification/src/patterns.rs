//! Repository-wide diagnosis of recurring verification shapes.
//!
//! The regression format intentionally follows Kani's `expected` suite model:
//! stable expected fragments must occur in generated diagnostics, while volatile
//! counts may grow as Tokio evolves.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternSpec {
    pub id: &'static str,
    pub needles: &'static [&'static str],
    pub minimum_files: usize,
    pub generator: &'static str,
    pub proof_shape: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternDiagnosis {
    pub spec: PatternSpec,
    pub files: Vec<PathBuf>,
}

macro_rules! proof_pattern_catalog {
    (
        $(
            $id:literal => {
                needles: [$($needle:literal),+ $(,)?],
                minimum_files: $minimum:literal,
                generator: $generator:literal,
                proof_shape: $shape:literal $(,)?
            }
        ),+ $(,)?
    ) => {
        pub const PATTERNS: &[PatternSpec] = &[
            $(
                PatternSpec {
                    id: $id,
                    needles: &[$($needle),+],
                    minimum_files: $minimum,
                    generator: $generator,
                    proof_shape: $shape,
                }
            ),+
        ];
    };
}

proof_pattern_catalog! {
    "unsafe-cell-protocol" => {
        needles: ["UnsafeCell"],
        minimum_files: 1,
        generator: "attribute+declarative",
        proof_shape: "exclusive/shared access protocol and protected-field invariant",
    },
    "unsafe-send-sync" => {
        needles: ["unsafe impl", "Send for"],
        minimum_files: 1,
        generator: "derive+attribute",
        proof_shape: "thread-safety preconditions imply Send/Sync witness",
    },
    "nonnull-provenance" => {
        needles: ["NonNull"],
        minimum_files: 1,
        generator: "attribute+delegate",
        proof_shape: "non-null provenance, lifetime, ownership and alias discipline",
    },
    "pin-address-stability" => {
        needles: ["PhantomPinned"],
        minimum_files: 1,
        generator: "procedural-attribute",
        proof_shape: "address stability, intrusive membership and projection invariant",
    },
    "manual-drop" => {
        needles: ["ManuallyDrop"],
        minimum_files: 1,
        generator: "derive+delegate",
        proof_shape: "exactly-once destruction and refcount ownership",
    },
    "atomic-rmw-loop" => {
        needles: ["compare_exchange"],
        minimum_files: 1,
        generator: "declarative-state-machine",
        proof_shape: "linearizable RMW transition preserves state invariant",
    },
    "atomic-bitfield-state" => {
        needles: ["AtomicUsize", "MASK"],
        minimum_files: 1,
        generator: "derive+declarative",
        proof_shape: "bit partition, legal states, refcount bounds and transitions",
    },
    "intrusive-list" => {
        needles: ["LinkedList", "Pointers"],
        minimum_files: 1,
        generator: "derive+declarative+delegate",
        proof_shape: "membership, reachability, bidirectional links and pin ownership",
    },
    "raw-waker-vtable" => {
        needles: ["RawWaker", "RawWakerVTable"],
        minimum_files: 1,
        generator: "procedural-attribute+delegate",
        proof_shape: "clone/wake/drop refcount conservation and pointer validity",
    },
    "repr-c-layout" => {
        needles: ["#[repr(C)]"],
        minimum_files: 1,
        generator: "derive+const-proof",
        proof_shape: "field layout/offset agreement and cast validity",
    },
    "unsafe-vtable-functions" => {
        needles: ["struct Vtable", "unsafe fn"],
        minimum_files: 1,
        generator: "attribute+delegate",
        proof_shape: "function-pointer contract family over one raw task representation",
    },
    "ptr-write" => {
        needles: ["ptr::write"],
        minimum_files: 1,
        generator: "attribute+declarative",
        proof_shape: "slot initialization, ownership transfer and no double drop",
    },
    "maybe-uninit" => {
        needles: ["MaybeUninit"],
        minimum_files: 1,
        generator: "derive+attribute",
        proof_shape: "initialization-state typestate and valid read/drop",
    },
    "unsafe-op-scope" => {
        needles: ["allow(unsafe_op_in_unsafe_fn)"],
        minimum_files: 1,
        generator: "procedural-attribute+metaverifier",
        proof_shape: "every unchecked operation is covered by a named safety contract",
    }
}

pub fn scan_tokio(root: &Path) -> Result<Vec<PatternDiagnosis>, String> {
    let src = root.join("tokio/src");
    let mut rust_files = Vec::new();
    collect_rust_files(&src, &mut rust_files)?;
    rust_files.sort();

    let mut diagnoses = Vec::with_capacity(PATTERNS.len());
    for spec in PATTERNS {
        let mut files = Vec::new();
        for path in &rust_files {
            let text = fs::read_to_string(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            if spec.needles.iter().all(|needle| text.contains(needle)) {
                files.push(path.strip_prefix(root).unwrap_or(path).to_path_buf());
            }
        }
        diagnoses.push(PatternDiagnosis { spec: *spec, files });
    }
    Ok(diagnoses)
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|error| format!("failed to read directory {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn report(diagnoses: &[PatternDiagnosis]) -> String {
    let mut output = String::new();
    for diagnosis in diagnoses {
        let status = if diagnosis.files.len() >= diagnosis.spec.minimum_files {
            "covered"
        } else {
            "missing"
        };
        output.push_str(&format!(
            "PATTERN {} files={} minimum={} generator={} status={} shape={}\n",
            diagnosis.spec.id,
            diagnosis.files.len(),
            diagnosis.spec.minimum_files,
            diagnosis.spec.generator,
            status,
            diagnosis.spec.proof_shape
        ));
        for path in &diagnosis.files {
            output.push_str(&format!("  SOURCE {}\n", path.display()));
        }
    }
    output
}

pub fn validate(diagnoses: &[PatternDiagnosis]) -> Vec<String> {
    let mut problems = Vec::new();
    for diagnosis in diagnoses {
        if diagnosis.files.len() < diagnosis.spec.minimum_files {
            problems.push(format!(
                "pattern {} has {} matching files, expected at least {}",
                diagnosis.spec.id,
                diagnosis.files.len(),
                diagnosis.spec.minimum_files
            ));
        }
        if diagnosis.spec.generator.trim().is_empty() || diagnosis.spec.proof_shape.trim().is_empty() {
            problems.push(format!("pattern {} has no metaprogramming/proof strategy", diagnosis.spec.id));
        }
    }
    problems
}

pub fn check_expected_fragments(report: &str, expected: &str) -> Vec<String> {
    expected
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|fragment| !report.contains(fragment))
        .map(|fragment| format!("expected diagnostic fragment not found: {fragment}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_suite_semantics_match_kani_style_fragments() {
        let output = "alpha\nbeta details\ngamma\n";
        let expected = "# comment\nalpha\nbeta\n";
        assert!(check_expected_fragments(output, expected).is_empty());
        assert_eq!(
            check_expected_fragments(output, "missing"),
            vec!["expected diagnostic fragment not found: missing"]
        );
    }

    #[test]
    fn every_pattern_has_a_generator_and_shape() {
        assert!(PATTERNS.iter().all(|pattern| {
            !pattern.generator.is_empty()
                && !pattern.proof_shape.is_empty()
                && !pattern.needles.is_empty()
                && pattern.minimum_files > 0
        }));
    }
}
