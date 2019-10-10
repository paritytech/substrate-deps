use crate::error::{CliError, CliResult};
use crate::metadata::SubstrateMetadata;

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

pub fn add_module_to_manifest(
    manifest_path: &Path,
    mod_dependency: &Dependency,
    mod_alias: &Option<&str>,
    mod_metadata: &Option<SubstrateMetadata>,
    registry: Option<&str>,
) -> CliResult<()> {
    // Open TOML manifest
    let mut manifest = Manifest::open(&Some(manifest_path.to_path_buf()))
        .map_err(|e| CliError::Manifest(e.to_string()))?;

    let mod_name = &inflector::cases::camelcase::to_camel_case(module_name(
        mod_dependency,
        mod_alias,
        mod_metadata,
    ));

    // Generate TOML table for module dependency
    let mod_dep_toml = module_dependency_to_toml(
        mod_name,
        mod_dependency.version().unwrap(),
        &mod_dependency.name,
        registry,
    );

    // Add module TOML table to dependencies table
    insert_into_table(&mut manifest, &["dependencies".to_owned()], mod_dep_toml)?;

    // Add module/std to features table
    let mod_feature = format!("{}/std", mod_name);
    insert_into_array(&mut manifest, &["features".to_owned()], "std", mod_feature)?;

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
        array.push(array_entry);
    }

    Ok(())
}

pub fn module_name<'a>(
    mod_dependency: &'a Dependency,
    mod_alias: &Option<&'a str>,
    mod_metadata: &'a Option<SubstrateMetadata>,
) -> &'a str {
    match (mod_alias, mod_metadata) {
        (Some(alias), _) => alias,
        (None, Some(meta)) => meta.module_alias(),
        (None, None) => &mod_dependency.name,
    }
}

fn module_dependency_to_toml(
    name: &str,
    version: &str,
    package: &str,
    registry: Option<&str>,
) -> (String, toml_edit::Table) {
    let mut data = toml_edit::Table::new();
    data["package"] = toml_edit::value(package);
    data["version"] = toml_edit::value(version);
    data["default-features"] = toml_edit::value(false);
    if let Some(registry) = registry {
        data["registry"] = toml_edit::value(registry);
    }
    (name.to_string(), data)
}
