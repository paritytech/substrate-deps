use crate::error::*;
use crate::metadata::Manifest;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};
use clap::ArgMatches;
use std::{
    fs,
    io::{self, Write},
};

lazy_static! {
    static ref SUBSTRATE_SRML: [String; 29] = [
        "srml-assets".to_owned(),
        "srml-aura".to_owned(),
        "srml-authority-discovery".to_owned(),
        "srml-authorship".to_owned(),
        "srml-babe".to_owned(),
        "srml-balances".to_owned(),
        "srml-collective".to_owned(),
        "srml-contracts".to_owned(),
        "srml-council".to_owned(),
        "srml-democracy".to_owned(),
        "srml-elections".to_owned(),
        "srml-example".to_owned(),
        "srml-executive".to_owned(),
        "srml-finality-tracker".to_owned(),
        "srml-generic-asset".to_owned(),
        "srml-grandpa".to_owned(),
        "srml-im-online".to_owned(),
        "srml-indices".to_owned(),
        "srml-membership".to_owned(),
        "srml-metadata".to_owned(),
        "srml-offences".to_owned(),
        "srml-scored-pool".to_owned(),
        "srml-session".to_owned(),
        "srml-staking".to_owned(),
        "srml-sudo".to_owned(),
        "srml-support".to_owned(),
        "srml-system".to_owned(),
        "srml-timestamp".to_owned(),
        "srml-treasury".to_owned(),
    ];
}

pub fn execute_graph(m: &ArgMatches) -> CliResult<()> {
    // debug!("Manifest path: {:?}", manifest_path);

    let mut cfg = Config::from_matches(m)?;
    let manifest = read_manifest(&cfg.manifest_path)?;

    let mut filter = vec![manifest.package().as_ref().unwrap().name().to_owned()];
    filter.append(&mut SUBSTRATE_SRML.to_vec());
    cfg.filter = Some(filter);
    cfg.transitive_deps = false;

    // cargo_deps::execute(cfg)?;

    // let dot_file = cfg.dot_file.clone();

    // Get dependency graph & render it
    let o = get_dep_graph(cfg)
        .and_then(|g| render_dep_graph(g))
        .map_err(|e| e.exit())
        .unwrap();

    io::stdout()
        .write_all(&o.into_bytes())
        .expect("Unable to write graph");

    // Output to stoud or render the dot file
    // match dot_file {
    //     None => Box::new(io::stdout()) as Box<dyn Write>,
    //     Some(file) => Box::new(File::create(&Path::new(&file)).expect("Failed to create file")),
    // }
    // .write_all(&o.into_bytes())
    // .expect("Unable to write graph");

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
