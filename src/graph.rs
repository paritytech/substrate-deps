use crate::error::*;
use crate::metadata::Manifest;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};
use clap::ArgMatches;
use std::{
    fs,
    io::{self, Write},
};

lazy_static! {
    static ref PAINT: [String; 29] = [
        "paint-assets".to_owned(),
        "paint-aura".to_owned(),
        "paint-authority-discovery".to_owned(),
        "paint-authorship".to_owned(),
        "paint-babe".to_owned(),
        "paint-balances".to_owned(),
        "paint-collective".to_owned(),
        "paint-contracts".to_owned(),
        "paint-council".to_owned(),
        "paint-democracy".to_owned(),
        "paint-elections".to_owned(),
        "paint-example".to_owned(),
        "paint-executive".to_owned(),
        "paint-finality-tracker".to_owned(),
        "paint-generic-asset".to_owned(),
        "paint-grandpa".to_owned(),
        "paint-im-online".to_owned(),
        "paint-indices".to_owned(),
        "paint-membership".to_owned(),
        "paint-metadata".to_owned(),
        "paint-offences".to_owned(),
        "paint-scored-pool".to_owned(),
        "paint-session".to_owned(),
        "paint-staking".to_owned(),
        "paint-sudo".to_owned(),
        "paint-support".to_owned(),
        "paint-system".to_owned(),
        "paint-timestamp".to_owned(),
        "paint-treasury".to_owned(),
    ];
}

pub fn execute_graph(m: &ArgMatches) -> CliResult<()> {
    // debug!("Manifest path: {:?}", manifest_path);

    let mut cfg = Config::from_matches(m)?;
    let manifest = read_manifest(&cfg.manifest_path)?;

    let mut filter = vec![manifest.package().as_ref().unwrap().name().to_owned()];
    filter.append(&mut PAINT.to_vec());
    cfg.filter = Some(filter);
    cfg.transitive_deps = false;

    // Get dependency graph & render it
    let o = get_dep_graph(cfg).and_then(render_dep_graph)?;
    io::stdout()
        .write_all(&o.into_bytes())
        .expect("Unable to write graph");

    Ok(())
}

fn read_manifest(manifest: &str) -> CliResult<Manifest> {
    let s = fs::read_to_string(manifest)?;
    let manifest: Manifest = toml::from_str(&s).map_err(|_| {
        CliError::Metadata(
            "Error reading module metadata: could parse crate manifest as TOML.".to_owned(),
        )
    })?;
    Ok(manifest)
}
