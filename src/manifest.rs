use crate::error::{CliError, CliResult};

use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
};

use cargo_edit::{Dependency, Manifest};

pub fn find_manifest_file(file: &str) -> CliResult<PathBuf> {
    let pwd = env::current_dir()?;
    let manifest = pwd.join(file);
    let file_name = manifest.file_name().unwrap();
    let mut dir = manifest.parent().unwrap().to_path_buf();
    let mut first_try = true;

    loop {
        let try_manifest = dir.join(file_name);

        if let Ok(metadata) = fs::metadata(&try_manifest) {
            if metadata.is_file() {
                if !first_try {
                    eprintln!("Found {:?} in {:?}.", file_name, dir.display());
                }

                return Ok(try_manifest);
            }
        }

        if first_try {
            eprintln!(
                "Could not find {:?} in {:?}, searching parent directories.",
                file_name,
                dir.display()
            );
            first_try = false;
        }

        dir = match dir.parent() {
            None => {
                return Err(CliError::Generic(format!(
                    "Could not find {:?} in {:?} or any parent directory",
                    file,
                    manifest.parent().unwrap()
                )));
            }
            Some(ref dir) => dir.to_path_buf(),
        };
    }
}

pub fn add_pallet_to_manifest(
    manifest_path: &Path,
    dependency: &Dependency,
    alias: &Option<&str>,
    registry: Option<&str>,
) -> CliResult<()> {
    // Open TOML manifest
    let mut manifest = Manifest::open(&Some(manifest_path.to_path_buf()))
        .map_err(|e| CliError::Manifest(e.to_string()))?;

    let name = &inflector::cases::camelcase::to_camel_case(pallet_alias(dependency, alias));

    // Generate TOML table for pallet dependency
    let dep_toml = pallet_dependency_to_toml(
        name,
        dependency.version().unwrap(),
        &dependency.name,
        registry,
    );

    // Add pallet TOML table to dependencies table
    insert_into_table(&mut manifest, &["dependencies".to_owned()], dep_toml)?;

    // Add pallet/std to features table
    let feature = format!("{}/std", name);
    insert_into_array(&mut manifest, &["features".to_owned()], "std", feature)?;

    // Write modified TOML manifest
    Manifest::find_file(&Some(manifest_path.to_path_buf()))
        .and_then(|mut file| manifest.write_to_file(&mut file))
        .map_err(|e| CliError::Manifest(e.to_string()))?;

    Ok(())
}

fn insert_into_table(
    manifest: &mut Manifest,
    table_path: &[String],
    table_entry: (String, toml_edit::Table),
) -> CliResult<()> {
    let (entry_name, entry_table) = table_entry;
    let table = manifest
        .get_table(table_path)
        .map_err(|e| CliError::Manifest(e.to_string()))?;

    if table[&entry_name].is_none() {
        let entries = table.as_table_mut().ok_or_else(|| {
            CliError::Manifest(format!(
                "Error updating '{}' runtime manifest.",
                table_path.first().unwrap()
            ))
        })?;

        let _ = entries
            .entry(&entry_name)
            .or_insert(toml_edit::Item::Table(entry_table));
        entries.sort_values();
    } /*else {
          // update an existing entry
          merge_dependencies(&mut table[&dep.name], dep);
          if let Some(t) = table.as_inline_table_mut() {
              t.fmt()
          }
      }*/

    Ok(())
}

fn insert_into_array(
    manifest: &mut Manifest,
    table_path: &[String],
    table_entry: &str,
    array_entry: String,
) -> CliResult<()> {
    let table = manifest
        .get_table(table_path)
        .map_err(|e| CliError::Manifest(e.to_string()))?;

    let array = table
        .as_table_mut()
        .and_then(|tm| tm.entry(table_entry).as_array_mut())
        .ok_or_else(|| {
            CliError::Manifest(format!(
                "Error updating '{}' in runtime manifest.",
                table_path.first().unwrap()
            ))
        })?;

    if !array.iter().any(|v| v.as_str() == Some(&array_entry)) {
        array.push(format!("'{}'", array_entry));
    }

    Ok(())
}

pub fn pallet_alias<'a>(dependency: &'a Dependency, alias: &Option<&'a str>) -> &'a str {
    match alias {
        Some(alias) => alias,
        None => &dependency.name,
    }
}

fn pallet_dependency_to_toml(
    name: &str,
    version: &str,
    package: &str,
    registry: Option<&str>,
) -> (String, toml_edit::Table) {
    let mut data = toml_edit::Table::new();
    data["package"] = toml_edit::value(format!("'{}'", package));
    data["version"] = toml_edit::value(format!("'{}'", version));
    data["default-features"] = toml_edit::value(false);
    if let Some(registry) = registry {
        data["registry"] = toml_edit::value(format!("'{}'", registry));
    }
    (name.to_string(), data)
}
