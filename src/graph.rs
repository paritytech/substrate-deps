use crate::error::*;
use crate::metadata::Manifest;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};
use clap::ArgMatches;
use std::{
    fs,
    io::{self, Write},
};

lazy_static! {
    static ref FRAME: [String; 34] = [
        "frame-assets".to_owned(),
        "frame-aura".to_owned(),
        "frame-authority-discovery".to_owned(),
        "frame-authorship".to_owned(),
        "frame-babe".to_owned(),
        "frame-balances".to_owned(),
        "frame-collective".to_owned(),
        "frame-contracts".to_owned(),
        "frame-democracy".to_owned(),
        "frame-elections".to_owned(),
        "frame-elections-phragmen".to_owned(),
        "frame-evm".to_owned(),
        "frame-example".to_owned(),
        "frame-executive".to_owned(),
        "frame-finality-tracker".to_owned(),
        "frame-generic-asset".to_owned(),
        "frame-grandpa".to_owned(),
        "frame-im-online".to_owned(),
        "frame-indices".to_owned(),
        "frame-membership".to_owned(),
        "frame-metadata".to_owned(),
        "frame-nicks".to_owned(),
        "frame-offences".to_owned(),
        "frame-randomness-collective-flip".to_owned(),
        "frame-scored-pool".to_owned(),
        "frame-session".to_owned(),
        "frame-staking".to_owned(),
        "frame-sudo".to_owned(),
        "frame-support".to_owned(),
        "frame-system".to_owned(),
        "frame-timestamp".to_owned(),
        "frame-transaction-payment".to_owned(),
        "frame-treasury".to_owned(),
        "frame-utility".to_owned(),
    ];
}

pub fn execute_graph(m: &ArgMatches) -> CliResult<()> {
    // debug!("Manifest path: {:?}", manifest_path);

    let mut cfg = Config::from_matches(m)?;
    let manifest = read_manifest(&cfg.manifest_path)?;

    let mut filter = vec![manifest.package().as_ref().unwrap().name().to_owned()];
    filter.append(&mut FRAME.to_vec());
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
