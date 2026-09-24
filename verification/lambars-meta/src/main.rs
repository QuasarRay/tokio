#![forbid(unsafe_code)]

use lambars::effect::Writer;
use lambars::optics::Lens;
use lambars::persistent::PersistentVector;
use lambars::typeclass::{Foldable, FunctorMut};
use lambars::pipe;
use lambars_derive::Lenses;
use std::env;
use std::process::ExitCode;
use tokio_verification::patterns::PATTERNS;
use tokio_verification::proof_families::{
    validate_pattern_coverage, Coverage, PROOF_FAMILIES,
};

#[derive(Clone, Debug, Lenses)]
struct PlanEntry {
    pattern: String,
    backend: String,
    mechanism: String,
    coverage: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tokio-lambars-meta: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let command = env::args().nth(1).unwrap_or_else(|| "check".to_owned());
    let coverage_problems = validate_pattern_coverage();
    if !coverage_problems.is_empty() {
        return Err(format!("proof-family coverage failed: {coverage_problems:#?}"));
    }

    let writer = build_plan().and_then(|plan| {
        let rendered = pipe!(plan, normalize_first_entry, render_markdown);
        Writer::new(rendered, vec!["rendered deterministic proof plan".to_owned()])
    });
    let (report, log) = writer.run();

    match command.as_str() {
        "check" => {
            let expected = PATTERNS.len();
            let rows = report
                .lines()
                .filter(|line| line.starts_with("| ") && !line.starts_with("| Pattern"))
                .count();
            if rows != expected {
                return Err(format!(
                    "generated plan contains {rows} pattern rows; expected {expected}"
                ));
            }
            for message in log {
                eprintln!("meta: {message}");
            }
            println!("lambars metaverification passed: {rows} proof families");
            Ok(())
        }
        "render" => {
            print!("{report}");
            Ok(())
        }
        other => Err(format!("unknown command {other}; expected check or render")),
    }
}

fn build_plan() -> Writer<Vec<String>, PersistentVector<PlanEntry>> {
    let entries: Vec<PlanEntry> = PATTERNS.to_vec().fmap_mut(|pattern| {
        let family = PROOF_FAMILIES
            .iter()
            .find(|family| family.pattern_id == pattern.id)
            .expect("pattern coverage was validated before plan construction");

        PlanEntry {
            pattern: pattern.id.to_owned(),
            backend: format!("{:?}", family.backend),
            mechanism: family.mechanism.to_owned(),
            coverage: format!("{:?}", family.coverage),
        }
    });

    let generated = entries.clone().fold_left(0usize, |count, entry| {
        count + usize::from(entry.coverage == format!("{:?}", Coverage::Generated))
    });

    let plan: PersistentVector<PlanEntry> = entries.into_iter().collect();
    Writer::new(
        plan,
        vec![
            format!("diagnosed {} recurring proof patterns", PATTERNS.len()),
            format!("{generated} proof families already have generated backend models"),
            "persistent proof plan constructed with structural sharing".to_owned(),
        ],
    )
}

fn normalize_first_entry(plan: PersistentVector<PlanEntry>) -> PersistentVector<PlanEntry> {
    let Some(first) = plan.get(0).cloned() else {
        return plan;
    };

    let normalized = PlanEntry::mechanism_lens().modify_ref(first, |mechanism| {
        mechanism.split_whitespace().collect::<Vec<_>>().join(" ")
    });

    match plan.update(0, normalized) {
        Some(updated) => updated,
        None => plan,
    }
}

fn render_markdown(plan: PersistentVector<PlanEntry>) -> String {
    let mut output = String::from(
        "# Generated proof plan\n\n| Pattern | Backend | Mechanism | Coverage |\n|---|---|---|---|\n",
    );
    for entry in plan.iter() {
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            entry.pattern, entry.backend, entry.mechanism, entry.coverage
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambars_pipeline_covers_every_diagnosed_pattern() {
        let (plan, log) = build_plan().run();
        assert_eq!(plan.len(), PATTERNS.len());
        assert!(!log.is_empty());
        assert!(plan.iter().all(|entry| !entry.mechanism.is_empty()));
    }

    #[test]
    fn optics_normalization_preserves_plan_length() {
        let (plan, _) = build_plan().run();
        let len = plan.len();
        let normalized = normalize_first_entry(plan);
        assert_eq!(normalized.len(), len);
    }
}
