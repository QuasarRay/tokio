//! Proof-obligation registry and metaverification.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Kani,
    Verus,
    Loom,
    Test,
}

impl Backend {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "kani" => Ok(Self::Kani),
            "verus" => Ok(Self::Verus),
            "loom" => Ok(Self::Loom),
            "test" => Ok(Self::Test),
            other => Err(format!("unknown verification backend `{other}`")),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kani => "kani",
            Self::Verus => "verus",
            Self::Loom => "loom",
            Self::Test => "test",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obligation {
    pub id: String,
    pub backend: Backend,
    pub assurance: String,
    pub source: PathBuf,
    pub anchor: String,
    pub harness: PathBuf,
    pub claim: String,
}

pub fn parse_registry(input: &str) -> Result<Vec<Obligation>, String> {
    let mut obligations = Vec::new();

    for (index, raw) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<_> = line.split('|').map(str::trim).collect();
        if fields.len() != 7 {
            return Err(format!(
                "line {line_number}: expected 7 pipe-separated fields, found {}",
                fields.len()
            ));
        }
        if fields.iter().any(|field| field.is_empty()) {
            return Err(format!("line {line_number}: fields must not be empty"));
        }

        obligations.push(Obligation {
            id: fields[0].to_owned(),
            backend: Backend::parse(fields[1])?,
            assurance: fields[2].to_owned(),
            source: PathBuf::from(fields[3]),
            anchor: fields[4].to_owned(),
            harness: PathBuf::from(fields[5]),
            claim: fields[6].to_owned(),
        });
    }

    if obligations.is_empty() {
        return Err("proof-obligation registry is empty".to_owned());
    }

    Ok(obligations)
}

pub fn load_registry(root: &Path) -> Result<Vec<Obligation>, String> {
    let path = root.join("verification/obligations.txt");
    let input = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    parse_registry(&input)
}

fn safe_relative(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

pub fn validate_registry(root: &Path, obligations: &[Obligation]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut ids = HashSet::new();
    let mut markers: HashMap<String, usize> = HashMap::new();

    for obligation in obligations {
        if !ids.insert(obligation.id.clone()) {
            problems.push(format!("duplicate obligation id `{}`", obligation.id));
        }
        if !safe_relative(&obligation.source) {
            problems.push(format!(
                "{}: source path must stay inside the repository: {}",
                obligation.id,
                obligation.source.display()
            ));
            continue;
        }
        if !safe_relative(&obligation.harness) {
            problems.push(format!(
                "{}: harness path must stay inside the repository: {}",
                obligation.id,
                obligation.harness.display()
            ));
            continue;
        }

        let source = root.join(&obligation.source);
        match fs::read_to_string(&source) {
            Ok(contents) => {
                if !contents.contains(&obligation.anchor) {
                    problems.push(format!(
                        "{}: source anchor is stale or missing in {}: `{}`",
                        obligation.id,
                        obligation.source.display(),
                        obligation.anchor
                    ));
                }
            }
            Err(error) => problems.push(format!(
                "{}: cannot read source {}: {error}",
                obligation.id,
                obligation.source.display()
            )),
        }

        let harness = root.join(&obligation.harness);
        match fs::read_to_string(&harness) {
            Ok(contents) => {
                let marker = format!("TOKIO_PROOF: {}", obligation.id);
                let count = contents.matches(&marker).count();
                if count != 1 {
                    problems.push(format!(
                        "{}: expected exactly one `{marker}` marker in {}, found {count}",
                        obligation.id,
                        obligation.harness.display()
                    ));
                }
                *markers.entry(obligation.id.clone()).or_default() += count;
            }
            Err(error) => problems.push(format!(
                "{}: cannot read harness {}: {error}",
                obligation.id,
                obligation.harness.display()
            )),
        }
    }

    for (id, count) in markers {
        if count != 1 {
            problems.push(format!(
                "{id}: proof marker must map one-to-one with its registry entry; found {count}"
            ));
        }
    }

    problems
}

pub fn markdown_matrix(obligations: &[Obligation]) -> String {
    let mut out = String::from(
        "| Obligation | Backend | Assurance | Source | Harness | Claim |\n|---|---|---|---|---|---|\n",
    );
    for obligation in obligations {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | {} |\n",
            obligation.id,
            obligation.backend.as_str(),
            obligation.assurance,
            obligation.source.display(),
            obligation.harness.display(),
            obligation.claim.replace('|', "\\|")
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_unknown_backend() {
        let err = parse_registry("x|unknown|model|a|b|c|claim").unwrap_err();
        assert!(err.contains("unknown verification backend"));
    }

    #[test]
    fn parser_rejects_wrong_field_count() {
        let err = parse_registry("x|kani|model|a|b|c").unwrap_err();
        assert!(err.contains("expected 7"));
    }

    // TOKIO_PROOF: metaverifier-self-test
    #[test]
    fn metaverifier_detects_stale_anchors_and_duplicate_ids() {
        crate::proof_obligation!("metaverifier-self-test");

        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("tokio-metaverify-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("source.rs"), "fn live_anchor() {}\n").unwrap();
        fs::write(
            root.join("proof.rs"),
            "// TOKIO_PROOF: duplicate\n// TOKIO_PROOF: stale\n",
        )
        .unwrap();

        let registry = parse_registry(
            "duplicate|test|meta|source.rs|live_anchor|proof.rs|first\n\
             duplicate|test|meta|source.rs|live_anchor|proof.rs|second\n\
             stale|test|meta|source.rs|missing_anchor|proof.rs|stale anchor",
        )
        .unwrap();
        let problems = validate_registry(&root, &registry);

        assert!(problems
            .iter()
            .any(|p| p.contains("duplicate obligation id")));
        assert!(problems
            .iter()
            .any(|p| p.contains("source anchor is stale")));
        fs::remove_dir_all(root).unwrap();
    }
}
