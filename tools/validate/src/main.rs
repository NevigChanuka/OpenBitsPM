use std::{env, path::Path, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let root = args.next().unwrap_or_else(|| "manifests".into());
    if args.next().is_some() {
        eprintln!("usage: openbits-validate [manifests-directory]");
        return ExitCode::from(2);
    }

    match openbits_validate::load_catalog(Path::new(&root)) {
        Ok(catalog) => {
            println!(
                "Validated {} publishable manifest(s) and {} requiring review.",
                catalog.verified.len(),
                catalog.requires_verification.len()
            );
            for id in catalog.requires_verification {
                println!("Requires verification: {id}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
