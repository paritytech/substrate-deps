use crate::error::*;
use crate::manifest::add_module_to_manifest;
use crate::metadata::get_module_metadata;
use crate::registry::registry_path;
use crate::runtime::add_module_to_runtime;

use cargo_edit::{get_latest_dependency, registry_url, update_registry_index};
use log::{debug, info};
use std::path::PathBuf;
use url::Url;

pub fn handle_add(
    manifest_path: &PathBuf,
    module: &str,
    alias: Option<&str>,
    registry: Option<&str>,
) -> CliResult<()> {
    debug!("Manifest path: {:?}", manifest_path);
    debug!("Module: {}", module);
    debug!("Alias: {:?}", alias);
    debug!("Registry: {:?}", registry);
    assert!(registry.is_some(), "Must use a registry for now.");

    // Lookup registry URL
    let reg_url = registry_url(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    debug!("Registry URL: {}", reg_url);

    // Lookup registry path
    let reg_path = registry_path(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    debug!("Registry path: {:?}", reg_path);

    info!("Using registry '{}' at: {}", registry.unwrap(), reg_url);

    // Update registry index
    //TODO: add offline flag and skip update if set
    update_registry_index(&reg_url).map_err(|e| CliError::Registry(e.to_string()))?;

    // Add module dependency (and related dependencies, recursively)
    add_module_dependency(
        manifest_path,
        module,
        alias,
        (registry, &reg_url, &reg_path),
    )?;

    Ok(())
}

fn add_module_dependency(
    manifest_path: &PathBuf,
    module: &str,
    alias: Option<&str>,
    (registry, reg_url, reg_path): (Option<&str>, &Url, &PathBuf),
) -> CliResult<()> {
    // Lookup module latest version
    let mod_dependency =
        get_latest_dependency(module, true, manifest_path.as_ref(), &Some(reg_url.clone()))
            .map_err(|e| CliError::Dependency(e.to_string()))?;

    let mod_name = &mod_dependency.name;
    let mod_version = &mod_dependency.version().unwrap();
    debug!("Module found: {} v{}", mod_name, mod_version);

    // Fetch module metadata
    let mod_metadata = get_module_metadata(&mod_dependency, manifest_path, &reg_path)?;
    match &mod_metadata {
        Some(metadata) => {
            if let Some(mod_deps) = metadata.module_deps_defaults() {
                for mod_dep in mod_deps {
                    add_module_dependency(
                        manifest_path,
                        &mod_dep.1,
                        None,
                        (registry, reg_url, reg_path),
                    )?;
                }
            };
        }
        None => info!("No metadata found for module {}", module),
    }

    // Add module default config to runtime's lib.rs
    add_module_to_runtime(
        manifest_path.as_ref(),
        &mod_dependency,
        &alias,
        &mod_metadata,
    )?;

    info!(
        "Added module {} v{} as dependency in your node runtime manifest.",
        mod_name, mod_version
    );

    // Add module to runtime manifest
    add_module_to_manifest(
        manifest_path.as_ref(),
        &mod_dependency,
        &alias,
        &mod_metadata,
        registry,
    )?;

    info!(
        "Added module {} v{} configuration in your node runtime.",
        mod_name, mod_version
    );

    Ok(())
}
