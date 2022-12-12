use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::Serialize;
use walkdir::DirEntry;

use crate::checks::CheckCode;
use crate::cli::Overrides;
use crate::fs::iter_python_files;
use crate::resolver::{discover_settings, Resolver};
use crate::settings::types::SerializationFormat;
use crate::{Configuration, Settings};

/// Print the user-facing configuration settings.
pub fn show_settings(
    configuration: &Configuration,
    project_root: Option<&Path>,
    pyproject: Option<&Path>,
) {
    println!("Resolved configuration: {configuration:#?}");
    println!("Found project root at: {project_root:?}");
    println!("Found pyproject.toml at: {pyproject:?}");
}

/// Show the list of files to be checked based on current settings.
pub fn show_files(files: &[PathBuf], default: &Settings, overrides: &Overrides) {
    // Discover the settings for the filesystem hierarchy.
    let settings = discover_settings(files, overrides);
    let resolver = Resolver {
        default,
        settings: &settings,
    };

    // Collect all files in the hierarchy.
    let mut entries: Vec<DirEntry> = files
        .iter()
        .flat_map(|path| iter_python_files(path, &resolver))
        .flatten()
        .collect();
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    // Print the list of files.
    for entry in entries {
        println!("{}", entry.path().to_string_lossy());
    }
}

#[derive(Serialize)]
struct Explanation<'a> {
    code: &'a str,
    category: &'a str,
    summary: &'a str,
}

/// Explain a `CheckCode` to the user.
pub fn explain(code: &CheckCode, format: SerializationFormat) -> Result<()> {
    match format {
        SerializationFormat::Text | SerializationFormat::Grouped => {
            println!(
                "{} ({}): {}",
                code.as_ref(),
                code.category().title(),
                code.kind().summary()
            );
        }
        SerializationFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&Explanation {
                    code: code.as_ref(),
                    category: code.category().title(),
                    summary: &code.kind().summary(),
                })?
            );
        }
        SerializationFormat::Junit => {
            bail!("`--explain` does not support junit format")
        }
        SerializationFormat::Github => {
            bail!("`--explain` does not support GitHub format")
        }
    };
    Ok(())
}
