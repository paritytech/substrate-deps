#![warn(clippy::all)]

mod add;
mod error;
mod manifest;
mod metadata;
mod registry;
mod runtime;

#[macro_use]
extern crate lazy_static;

use crate::manifest::find_manifest_file;
use clap::{crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand};
use log::{warn, LevelFilter};
use std::env;

const SUBSTRATE_REGISTRY: &str = "substrate-mods";

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

        if let Err(err) = add::handle_add(&manifest_path, module, alias, registry) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
