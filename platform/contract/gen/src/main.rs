//! `contract-gen write [folder]` writes the generated files. `contract-gen check` fails when the
//! committed files differ from what the contract generates.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let contract_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the generator sits inside the contract folder")
        .to_path_buf();
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let out = args
        .next()
        .map_or_else(|| contract_dir.join("generated"), PathBuf::from);
    let fixtures = match contract_gen::fixture_names(&contract_dir) {
        Ok(names) => names,
        Err(error) => {
            eprintln!("cannot read fixtures: {error}");
            return ExitCode::FAILURE;
        }
    };
    let files = contract_gen::generate(&fixtures);
    match command.as_str() {
        "write" => match contract_gen::write_all(&files, &out) {
            Ok(()) => {
                println!("wrote {} files to {}", files.len(), out.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot write: {error}");
                ExitCode::FAILURE
            }
        },
        "check" => match contract_gen::differences(&files, &out) {
            Ok(problems) if problems.is_empty() => {
                println!("{} generated files match", files.len());
                ExitCode::SUCCESS
            }
            Ok(problems) => {
                problems.iter().for_each(|problem| eprintln!("{problem}"));
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("cannot read: {error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("usage: contract-gen write [folder] | contract-gen check [folder]");
            ExitCode::FAILURE
        }
    }
}
