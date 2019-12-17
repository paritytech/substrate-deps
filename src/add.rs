use crate::error::*;
use crate::manifest::add_module_to_manifest;
use crate::metadata::get_module_metadata;
use crate::registry::registry_path;
use crate::runtime::add_pallet_to_runtime;

use cargo_edit::{get_latest_dependency, registry_url, update_registry_index};
use log::{debug, info};
use std::path::PathBuf;
use url::Url;

pub fn execute_add(
    manifest_path: &PathBuf,
    pallet: &str,
    alias: Option<&str>,
    registry: Option<&str>,
) -> CliResult<()> {
    debug!("Manifest path: {:?}", manifest_path);
    debug!("Pallet: {}", pallet);
    debug!("Alias: {:?}", alias);
    debug!("Registry: {:?}", registry);

    // Lookup registry URL
    let reg_url = registry_url(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    debug!("Registry URL: {}", reg_url);

    // Lookup registry path
    let reg_path = registry_path(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    debug!("Registry path: {:?}", reg_path);

    info!(
        "Using registry '{}' at: {}",
        registry.unwrap_or("crates-io"),
        reg_url
    );

    // Update registry index
    //TODO: add offline flag and skip update if set
    update_registry_index(&reg_url).map_err(|e| CliError::Registry(e.to_string()))?;

    // Add pallet dependency (and related dependencies, recursively)
    add_pallet_dependency(
        manifest_path,
        pallet,
        alias,
        (registry, &reg_url, &reg_path),
    )?;

    Ok(())
}

fn add_pallet_dependency(
    manifest_path: &PathBuf,
    pallet: &str,
    alias: Option<&str>,
    (registry, reg_url, reg_path): (Option<&str>, &Url, &PathBuf),
) -> CliResult<()> {
    // Lookup module latest version
    let dependency =
        get_latest_dependency(pallet, true, manifest_path.as_ref(), &Some(reg_url.clone()))
            .map_err(|e| CliError::Dependency(e.to_string()))?;

    let name = &dependency.name;
    let version = &dependency.version().unwrap();
    debug!("Pallet found: {} v{}", name, version);

    // Fetch module metadata
    let metadata = get_module_metadata(&dependency, manifest_path, &reg_path)?;
    match &metadata {
        Some(metadata) => {
            if let Some(mod_deps) = metadata.module_deps_defaults() {
                for mod_dep in mod_deps {
                    add_pallet_dependency(
                        manifest_path,
                        &mod_dep.1,
                        None,
                        (registry, reg_url, reg_path),
                    )?;
                }
            };
        }
        None => info!("No metadata found for pallet {}", pallet),
    }

    // Add module default config to runtime's lib.rs
    add_pallet_to_runtime(manifest_path.as_ref(), &dependency, &alias, &metadata)?;

    info!(
        "Added module {} v{} as dependency in your node runtime manifest.",
        name, version
    );

    // Add module to runtime manifest
    add_module_to_manifest(
        manifest_path.as_ref(),
        &dependency,
        &alias,
        &metadata,
        registry,
    )?;

    info!(
        "metadata {} v{} configuration in your node runtime.",
        name, version
    );

    Ok(())
}
