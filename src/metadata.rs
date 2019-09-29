use crate::error::{CliError, CliResult};
use crate::registry::registry_path;
use cargo_metadata::{Metadata, MetadataCommand};
use git2::Repository;

use std::path::{Path, PathBuf};

pub fn get_metadata(module: &str, manifest_path: &Path, registry_path: &Path) -> CliResult<()> {
    // let reg_path = registry_path(manifest_path.as_ref(), registry)
    //     .map_err(|e| CliError::Registry(e.to_string()));
    // println!("Registry path: {:?}", reg_path);
    // let registry_path = match registry {
    //     Some(url) => registry_path_from_url(url)?,
    //     None => registry_path(manifest_path, None)?,
    // };

    let repo = Repository::open(registry_path)?;
    let tree = repo
        .find_reference("refs/remotes/origin/master")?
        .peel_to_tree()?;

    let file = match tree.get_path(&PathBuf::from(summary_raw_path(&module))) {
        Ok(p) => p
            .to_object(&repo)?
            .peel_to_blob()
            .map_err(|e| CliError::Metadata(e.to_string())),
        Err(e) => Err(CliError::Metadata(e.to_string())),
    }?;
    let content = String::from_utf8(file.content().to_vec())
        .map_err(|e| CliError::Metadata(e.to_string()))?;
    println!("{:?}", content);

    // return content
    //     .lines()
    //     .map(|line: &str| {
    //         serde_json::from_str::<CrateVersion>(line)
    //             .map_err(|_| ErrorKind::InvalidSummaryJson.into())
    //     })
    //     .collect::<Result<Vec<CrateVersion>>>();

    // let metadata = MetadataCommand::new()
    //     .manifest_path(manifest_path)
    //     .exec()
    //     .map_err(|e| CliError::Metadata(e.to_string()))?;
    // Ok(metadata)
    Ok(())
}

fn summary_raw_path(crate_name: &str) -> String {
    match crate_name.len() {
        0 => unreachable!("we check that crate_name is not empty here"),
        1 => format!("1/{}", crate_name),
        2 => format!("2/{}", crate_name),
        3 => format!("3/{}/{}", &crate_name[..1], crate_name),
        _ => format!("{}/{}/{}", &crate_name[..2], &crate_name[2..4], crate_name),
    }
}
