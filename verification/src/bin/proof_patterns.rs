use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio_verification::patterns::{check_expected_fragments, report, scan_tokio, validate};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tokio-proof-patterns: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_owned());
    let root = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()));

    let diagnoses = scan_tokio(&root)?;
    let rendered = report(&diagnoses);

    match command.as_str() {
        "report" => {
            print!("{rendered}");
            Ok(())
        }
        "check" => {
            let problems = validate(&diagnoses);
            if problems.is_empty() {
                print!("{rendered}");
                Ok(())
            } else {
                Err(format!("\n- {}", problems.join("\n- ")))
            }
        }
        "regression" => {
            let expected_path = root.join("verification/regression/patterns.expected");
            let expected = fs::read_to_string(&expected_path)
                .map_err(|error| format!("failed to read {}: {error}", expected_path.display()))?;
            let mut problems = validate(&diagnoses);
            problems.extend(check_expected_fragments(&rendered, &expected));
            if problems.is_empty() {
                print!("{rendered}");
                Ok(())
            } else {
                Err(format!("\n- {}", problems.join("\n- ")))
            }
        }
        other => Err(format!(
            "unknown command `{other}`; expected report, check, or regression"
        )),
    }
}
