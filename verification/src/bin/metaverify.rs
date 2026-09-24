use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio_verification::registry::{load_registry, markdown_matrix, validate_registry};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tokio-metaverify: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "check".to_owned());
    let root = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()));
    if args.next().is_some() {
        return Err("usage: tokio-metaverify [check|matrix] [repository-root]".to_owned());
    }

    let obligations = load_registry(&root)?;
    match command.as_str() {
        "check" => {
            let problems = validate_registry(&root, &obligations);
            if problems.is_empty() {
                println!("metaverification passed: {} obligations", obligations.len());
                Ok(())
            } else {
                Err(format!("\n- {}", problems.join("\n- ")))
            }
        }
        "matrix" => {
            print!("{}", markdown_matrix(&obligations));
            Ok(())
        }
        other => Err(format!(
            "unknown command `{other}`; expected `check` or `matrix`"
        )),
    }
}
