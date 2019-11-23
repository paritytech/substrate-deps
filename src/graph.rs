use crate::error::*;
use crate::metadata::Manifest;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};
use clap::ArgMatches;
use std::{
    fs,
    io::{self, Write},
};

lazy_static! {
    static ref PALETTE: [String; 34] = [
        "pallet-assets".to_owned(),
        "pallet-aura".to_owned(),
        "pallet-authority-discovery".to_owned(),
        "pallet-authorship".to_owned(),
        "pallet-babe".to_owned(),
        "pallet-balances".to_owned(),
        "pallet-collective".to_owned(),
        "pallet-contracts".to_owned(),
        "pallet-democracy".to_owned(),
        "pallet-elections".to_owned(),
        "pallet-elections-phragmen".to_owned(),
        "pallet-evm".to_owned(),
        "pallet-example".to_owned(),
        "pallet-executive".to_owned(),
        "pallet-finality-tracker".to_owned(),
        "pallet-generic-asset".to_owned(),
        "pallet-grandpa".to_owned(),
        "pallet-im-online".to_owned(),
        "pallet-indices".to_owned(),
        "pallet-membership".to_owned(),
        "pallet-metadata".to_owned(),
        "pallet-nicks".to_owned(),
        "pallet-offences".to_owned(),
        "pallet-randomness-collective-flip".to_owned(),
        "pallet-scored-pool".to_owned(),
        "pallet-session".to_owned(),
        "pallet-staking".to_owned(),
        "pallet-sudo".to_owned(),
        "pallet-support".to_owned(),
        "pallet-system".to_owned(),
        "pallet-timestamp".to_owned(),
        "pallet-transaction-payment".to_owned(),
        "pallet-treasury".to_owned(),
        "pallet-utility".to_owned(),
    ];
}

pub fn execute_graph(m: &ArgMatches) -> CliResult<()> {
    // debug!("Manifest path: {:?}", manifest_path);

    let mut cfg = Config::from_matches(m)?;
    let manifest = read_manifest(&cfg.manifest_path)?;

    let mut filter = vec![manifest.package().as_ref().unwrap().name().to_owned()];
    filter.append(&mut PALETTE.to_vec());
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
