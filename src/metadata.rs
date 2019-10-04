use crate::error::{CliError, CliResult};
use crate::registry::registry_path;
use cargo_edit::Dependency;
use cargo_metadata::{Metadata, MetadataCommand};
use git2::Repository;
use reqwest::header::CONTENT_LENGTH;
use serde::Deserialize;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
struct Manifest {
    package: Option<Package>,
}

#[derive(Clone, Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    metadata: Option<PackageMetadata>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PackageMetadata {
    substrate: Option<SubstrateMetadata>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SubstrateMetadata {
    module_name: String,
    module_label: Option<String>,
    icon: Option<String>,
    module_categories: Option<Vec<String>>,
    module_deps_defaults: Option<Vec<String>>,
    trait_deps_defaults: Option<Vec<String>>,
}

impl SubstrateMetadata {
    pub fn module_name(&self) -> &String {
        &self.module_name
    }

    pub fn module_label(&self) -> &Option<String> {
        &self.module_label
    }

    pub fn module_categories(&self) -> &Option<Vec<String>> {
        &self.module_categories
    }

    pub fn module_deps_defaults(&self) -> &Option<Vec<String>> {
        &self.module_deps_defaults
    }

    pub fn trait_deps_defaults(&self) -> &Option<Vec<String>> {
        &self.trait_deps_defaults
    }
}

pub fn get_metadata(
    module: &Dependency,
    manifest_path: &Path,
    registry_path: &Path,
) -> CliResult<SubstrateMetadata> {
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

    // Get registry config file
    let reg_cfg = match tree.get_path(&PathBuf::from("config.json")) {
        Ok(p) => p
            .to_object(&repo)?
            .peel_to_blob()
            .map_err(|e| CliError::Metadata(e.to_string())),
        Err(e) => Err(CliError::Metadata(e.to_string())),
    }?;
    let cfg = String::from_utf8(reg_cfg.content().to_vec())
        .map_err(|e| CliError::Metadata(e.to_string()))?;
    println!("{:?}", cfg);

    let reg_dl_uri = json::parse(cfg.as_str()).unwrap()["dl"]
        .take_string()
        .unwrap();
    println!("{:?}", reg_dl_uri);
    let crate_bytes = download_crate(module, reg_dl_uri).unwrap();
    //TODO: Write crate to cache
    // let mut file = fs::OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .open(p)
    //     .unwrap_or_else(|e| {
    //         error!("Failed to open output file {}: {}", p.display(), e);
    //         exit(exitcode::IOERR)
    //     });
    // file.write(&crate_bytes).unwrap();

    // Deflate & read Cargo.toml
    let gzip = flate2::read::GzDecoder::new(&crate_bytes[..]);
    let mut archive = tar::Archive::new(gzip);
    let mut manifest = archive
        .entries()
        .unwrap()
        .find(|f| {
            f.as_ref()
                .unwrap()
                .header()
                .path()
                .unwrap()
                .file_name()
                .unwrap()
                == std::ffi::OsStr::new("Cargo.toml")
        })
        .unwrap()
        .unwrap();

    let mut s = String::new();
    manifest.read_to_string(&mut s).unwrap();

    let manifest_toml: Manifest = toml::from_str(&s)?;

    // .data
    //     .as_table()
    //     .get("package.metadata.substrate")
    //     .and_then(|m| m["name"].as_str().map(std::string::ToString::to_string))
    //     .ok_or_else(|| ErrorKind::ParseCargoToml.into())

    // for file in archive.entries().unwrap() {
    //     // Make sure there wasn't an I/O error
    //     let mut file = file.unwrap();

    //     // Inspect metadata about the file
    //     println!("{:?}", file.header().path().unwrap());
    //     println!("{}", file.header().size().unwrap());

    //     // files implement the Read trait
    //     let mut s = String::new();
    //     file.read_to_string(&mut s).unwrap();
    //     println!("{}", s);
    // }
    // match archive.unpack(".") {
    //     Ok(_) => {
    //         // If -x option was passed, we need to move the extracted directory
    //         // to wherever the user wanted.
    //         let mut dir = dir;
    //         if let Some(&Output::Path(ref p)) = opts.output.as_ref() {
    //             fs::rename(&dir, p).unwrap_or_else(|e| {
    //                 error!(
    //                     "Failed to move extracted archive from {} to {}: {}",
    //                     dir.display(),
    //                     p.display(),
    //                     e
    //                 );
    //                 exit(exitcode::IOERR)
    //             });
    //             dir = p.clone();
    //         }
    //         info!("Crate content extracted to {}/", dir.display());
    //     }
    //     Err(e) => {
    //         error!("Couldn't extract crate to {}/: {}", dir.display(), e);
    //         exit(exitcode::TEMPFAIL)
    //     }
    // }

    // let file = match tree.get_path(&PathBuf::from(summary_raw_path(&module))) {
    //     Ok(p) => p
    //         .to_object(&repo)?
    //         .peel_to_blob()
    //         .map_err(|e| CliError::Metadata(e.to_string())),
    //     Err(e) => Err(CliError::Metadata(e.to_string())),
    // }?;
    // let content = String::from_utf8(file.content().to_vec())
    //     .map_err(|e| CliError::Metadata(e.to_string()))?;
    // println!("{:?}", content);

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
    Ok(manifest_toml
        .package
        .unwrap()
        .metadata
        .unwrap()
        .substrate
        .unwrap())
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

// See https://github.com/Xion/cargo-download/blob/master/src/main.rs
fn download_crate(module: &Dependency, reg_dl_uri: String) -> Result<Vec<u8>, Box<CliError>> {
    // Check if {crate} & {version} markers are present, if yes replace,
    // if not, assume {crate}/{version}/download URI
    let name = module.name.as_str();
    let version = module.version().unwrap();
    let download_url = reg_dl_uri
        .replace("{crate}", name)
        .replace("{version}", version);
    println!(
        "Downloading crate `{}=={}` from {}",
        name, version, download_url
    );

    let mut response = reqwest::get(&download_url).unwrap();

    let content_length: Option<usize> = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok());
    println!(
        "Download size: {}",
        content_length.map_or("<unknown>".into(), |cl| format!("{} bytes", cl))
    );
    let mut bytes = match content_length {
        Some(cl) => Vec::with_capacity(cl),
        None => Vec::new(),
    };
    response.read_to_end(&mut bytes).unwrap();

    println!("Crate `{}=={}` downloaded successfully", name, version);
    Ok(bytes)
}
