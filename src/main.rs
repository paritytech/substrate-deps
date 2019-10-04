#![warn(clippy::all)]

mod error;
mod metadata;
mod module;
mod registry;
mod runtime;
mod util;

use crate::error::*;
use crate::metadata::get_metadata;
use crate::module::to_toml;
use crate::registry::registry_path;
use crate::util::find_manifest_file;
use cargo_edit::Dependency;

use std::{
    env,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use cargo::{
    core::Workspace,
    ops::{fetch, FetchOptions},
    util::Config,
};
use cargo_edit::{get_latest_dependency, registry_url, update_registry_index, Manifest};
use cargo_metadata::MetadataCommand;
use clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use toml::{self, Value};
use toml_edit::Item as TomlItem;
use url::Url;

fn handle_add(manifest_path: &PathBuf, module: &str, registry: Option<&str>) -> CliResult<()> {
    println!("Manifest path: {:?}", manifest_path);
    println!("Module: {:?}", module);
    println!("Registry: {:?}", registry.unwrap());
    // let manifest_toml = toml_from_file(manifest).unwrap();

    let reg_url = registry_url(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    println!("Registry URL: {:?}", reg_url);
    //TODO: add offline flag and check it
    let _ = update_registry_index(&reg_url);

    //TODO: do we still need this if we fetch the manifest online ?
    // Verify module exists in registry, and get latest version
    let mut dep = get_latest_dependency(module, false, manifest_path.as_ref(), &Some(reg_url))
        .map_err(|e| CliError::Dependency(e.to_string()))?;
    println!("Module found: {:?}", dep);

    // let metadata = MetadataCommand::new().manifest_path(manifest_path).exec();
    // println!("Module metadata: {:?}", metadata);

    let reg_path = registry_path(manifest_path.as_ref(), registry)
        .map_err(|e| CliError::Registry(e.to_string()))?;
    println!("Registry path: {:?}", reg_path);

    let metadata = get_metadata(&dep, manifest_path, &reg_path)?;
    println!("Module metadata: {:?}", &metadata);

    // dep = dep
    //     .set_registry("substrate-mods")
    //     .set_default_features(false);

    // Add module to runtime Cargo.toml
    let mut manifest = Manifest::open(&Some(manifest_path.to_path_buf())).unwrap();
    let _ = insert_into_table(&mut manifest, &["dependencies".to_owned()], &dep)
        .map(|_| {
            manifest
                .get_table(&["dependencies".to_owned()])
                .map(TomlItem::as_table_mut)
                .map(|table_option| {
                    table_option.map(|table| {
                        // if args.sort {
                        table.sort_values();
                        // }
                    })
                })
        })
        .unwrap();

    let std_features = manifest
        .get_table(&["features".to_owned()])
        .unwrap()
        .as_table_mut()
        .unwrap()
        .entry("std")
        .as_array_mut()
        .unwrap();

    let dep_feature = format!("{}/std", metadata.module_name());
    if !std_features
        .iter()
        .any(|v| v.as_str() == Some(&dep_feature))
    {
        std_features.push(dep_feature);
    }

    let mut file = Manifest::find_file(&Some(manifest_path.to_path_buf())).unwrap();
    manifest.write_to_file(&mut file).unwrap();

    runtime::patch_runtime(manifest_path.as_ref(), metadata);

    // Do cargo fetch, to fetch module & its dependencies
    let cfg = Config::default().unwrap();
    // let ws_manifest = &manifest_path
    //     .parent()
    //     .unwrap()
    //     .parent()
    //     .unwrap()
    //     .join("Cargo.toml");
    // println!("{}", ws_manifest.as_path().display());
    // let ws = Workspace::new(&manifest_path, &cfg).unwrap();
    // let opts = FetchOptions {
    //     config: &cfg,
    //     target: None,
    // };
    // let _result = fetch(&ws, &opts).unwrap();

    // Build deps map, parse modules metadata, and add
    // modules concerned by 'defaults' in metadata.
    Ok(())
}

pub fn insert_into_table(
    manifest: &mut Manifest,
    table_path: &[String],
    dep: &Dependency,
) -> CliResult<()> {
    let table = manifest.get_table(table_path).unwrap();

    if table[&dep.name].is_none() {
        // insert a new entry
        let (name, new_dependency) = to_toml(&dep);
        // let root = manifest.data.as_table_mut();
        let deps = table.as_table_mut().unwrap();
        let _ = deps
            .entry(&name)
            .or_insert(toml_edit::Item::Table(new_dependency));
    } /*else {
          // update an existing entry
          merge_dependencies(&mut table[&dep.name], dep);
          if let Some(t) = table.as_inline_table_mut() {
              t.fmt()
          }
      }*/
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
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a module to the Substrate runtime.")
                .arg(
                    Arg::with_name("module")
                        .help("Module to be added e.g. srml-staking")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches()
}

fn main() {
    let m = parse_cli();

    // let registry_path = cargo_edit::registry_url(manifest_path: &Path, registry: Option<&str>)

    if let Some(m) = m.subcommand_matches("add") {
        //TODO: move to config.rs
        let manifest = m.value_of("manifest-path").unwrap();
        let manifest_path = find_manifest_file(manifest).unwrap();
        let module = m.value_of("module").unwrap();
        let registry = Some("substrate-mods");
        //--
        if let Err(err) = handle_add(&manifest_path, module, registry) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
