use crate::error::*;
use log::debug;
use std::path::PathBuf;

pub fn execute_graph(manifest_path: &PathBuf) -> CliResult<()> {
    debug!("Manifest path: {:?}", manifest_path);
    debug!("graphing");
    Ok(())
}
