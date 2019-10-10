#![warn(clippy::all)]

mod error;
mod manifest;
mod metadata;
mod registry;
mod runtime;

#[macro_use]
extern crate lazy_static;

use crate::error::*;
use crate::manifest::{add_module_to_manifest, find_manifest_file};
use crate::metadata::get_module_metadata;
use crate::registry::registry_path;
use crate::runtime::add_module_to_runtime;

use std::{env, path::PathBuf};

use cargo_edit::{get_latest_dependency, registry_url, update_registry_index};
use clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use log::{debug, info, warn, LevelFilter};
use url::Url;

const SUBSTRATE_REGISTRY: &str = "substrate-mods";

fn handle_add(
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

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .value_name("path")
                .help("Path to the manifest of the runtime.")
                .takes_value(true)
                .global(true)
                .default_value("Cargo.toml"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .global(true)
                .help("No output printed to stdout"),
        )
        .arg(
            Arg::with_name("registry")
                .long("registry")
                .value_name("registry")
                .help("Registry to use")
                .takes_value(true)
                .global(true)
                // For now, we target the Substrate alternative registry.
                // When Substrate stable modules & core crates are published
                // on crates.io, this default value will be removed and
                // crates.io will be used as the default registry.
                .default_value(SUBSTRATE_REGISTRY),
        )
        .arg(
            Arg::with_name("v")
                .long("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Use verbose output"),
        )
        //TODO: add support for verbose, quiet, (module) version,
        // offline, locked, no-default-features, etc
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a module to the Substrate runtime.")
                .arg(
                    Arg::with_name("module")
                        .help("Module to be added e.g. srml-staking")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("alias")
                        .long("alias")
                        .short("a")
                        .help("Alias to be used in code & config e.g. staking instead of srml-staking")
                        .takes_value(true)
                )
        )
        .get_matches()
}

fn config_log(m: &ArgMatches) {
    let log_level = if m.is_present("quiet") {
        LevelFilter::Error
    } else {
        match m.occurrences_of("v") {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2 | _ => LevelFilter::Trace,
        }
    };
    env_logger::from_env(env_logger::Env::default().default_filter_or(format!(
        "{}={}",
        crate_name!().replace("-", "_"),
        log_level
    )))
    .format_timestamp(None)
    .format_level(false)
    .format_module_path(false)
    .init();
}

fn main() {
    let m = parse_cli();
    config_log(&m);

    if let Some(m) = m.subcommand_matches("add") {
        //TODO: move to config.rs
        let module = m.value_of("module").unwrap(); // module arg is required so we can safely unwrap
        let alias = m.value_of("alias");
        let manifest = m.value_of("manifest-path").unwrap(); // manifest-path has a default value so we can safely unwrap
        let manifest_path = find_manifest_file(manifest).unwrap(); // -> Stop on error, if any
        let registry = m.value_of("registry");
        //TODO: should get (local registry path, registry uri)

        if let Err(err) = handle_add(&manifest_path, module, alias, registry) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
