use crate::error::{CliError, CliResult};

use std::io::Read;
use std::path::{Path, PathBuf};

use cargo_edit::Dependency;
use git2::Repository;
use log::debug;
use regex::Regex;
use reqwest::header::CONTENT_LENGTH;
use serde::Deserialize;

lazy_static! {
    static ref PALLET_DEPS_REGEX: Regex = Regex::new(r"([\w\d_-]+):([\w\d_-]+)").unwrap();
    static ref TRAIT_DEPS_REGEX: Regex = Regex::new(r"([\w\d_-]+)=([\w\d_-]+)").unwrap();
}

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    package: Option<Package>,
}

impl Manifest {
    pub fn package(&self) -> &Option<Package> {
        &self.package
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    metadata: Option<PackageMetadata>,
}

impl Package {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn metadata(&self) -> &Option<PackageMetadata> {
        &self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PackageMetadata {
    substrate: Option<SubstrateMetadata>,
}

impl PackageMetadata {
    pub fn substrate(&self) -> &Option<SubstrateMetadata> {
        &self.substrate
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SubstrateMetadata {
    pallet_alias: Option<String>,
    pallet_label: Option<String>,
    icon: Option<String>,
    pallet_categories: Option<Vec<String>>,
    pallet_deps_defaults: Option<Vec<String>>,
    trait_deps_defaults: Option<Vec<String>>,
    pallet_cfg_defaults: Option<Vec<String>>,
}

impl SubstrateMetadata {
    pub fn pallet_alias(&self) -> &Option<String> {
        &self.pallet_alias
    }

    pub fn pallet_label(&self) -> &Option<String> {
        &self.pallet_label
    }

    pub fn pallet_categories(&self) -> &Option<Vec<String>> {
        &self.pallet_categories
    }

    pub fn pallet_deps_defaults(&self) -> Option<Vec<(String, String)>> {
        match &self.pallet_deps_defaults {
            Some(deps) => deps
                .iter()
                .map(|dep| match PALLET_DEPS_REGEX.captures(dep) {
                    Some(cap) => Some((cap[1].to_owned(), cap[2].to_owned())),
                    None => None,
                })
                .collect(),
            None => None,
        }
    }

    pub fn trait_deps_defaults(&self) -> Option<Vec<(String, String)>> {
        match &self.trait_deps_defaults {
            Some(deps) => deps
                .iter()
                .map(|dep| match TRAIT_DEPS_REGEX.captures(dep) {
                    Some(cap) => Some((cap[1].to_owned(), cap[2].to_owned())),
                    None => None,
                })
                .collect(),
            None => None,
        }
    }

    pub fn pallet_cfg_defaults(&self) -> &Option<Vec<String>> {
        &self.pallet_cfg_defaults
    }
}

pub fn get_pallet_metadata(
    pallet: &Dependency,
    _manifest_path: &Path,
    registry_path: &Path,
) -> CliResult<Option<SubstrateMetadata>> {
    // Open registry local index repo
    let repo = Repository::open(registry_path)?;
    let tree = repo
        .find_reference("refs/remotes/origin/master")?
        .peel_to_tree()?;

    // Get registry config file
    let reg_cfg = match tree.get_path(&PathBuf::from("config.json")) {
        Ok(p) => p.to_object(&repo)?.peel_to_blob(),
        Err(e) => Err(e),
    }?;

    // Read registry config file
    let reg_cfg_str = String::from_utf8(reg_cfg.content().to_vec())?;

    // Read registry download URL
    let mut reg_cfg_json = json::parse(reg_cfg_str.as_str())?;
    let reg_dl_url = reg_cfg_json["dl"].take_string().ok_or_else(|| {
        CliError::Metadata(
            "Error reading pallet metadata: could not read registry download URL.".to_owned(),
        )
    })?;
    debug!("Registry download URL: {}", reg_dl_url);

    // Download pallet crate from registry
    let mod_crate = download_crate(pallet, reg_dl_url)?;

    //TODO: Write crate to local registry cache

    // Read Cargo.toml from crate
    let mod_manifest = read_manifest_from_crate(mod_crate)?;
    debug!("Successfully read manifest from pallet crate.");

    Ok(mod_manifest
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.substrate))
}

fn read_manifest_from_crate(crate_bytes: Vec<u8>) -> CliResult<Manifest> {
    // Deflate crate & read manifest entry
    let gzip = flate2::read::GzDecoder::new(&crate_bytes[..]);
    let mut archive = tar::Archive::new(gzip);
    let mut manifest = archive
        .entries()?
        .find(|x| match x {
            Ok(e) => match e.header().path() {
                Ok(p) => p.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")),
                Err(_) => false,
            },
            Err(_) => false,
        })
        .ok_or_else(|| {
            CliError::Metadata(
                "Error reading pallet metadata: could not read crate manifest.".to_owned(),
            )
        })??;

    let mut s = String::new();
    manifest.read_to_string(&mut s).unwrap();

    toml::from_str(&s).map_err(|_| {
        CliError::Metadata(
            "Error reading pallet metadata: could parse crate manifest as TOML.".to_owned(),
        )
    })
}

// See https://github.com/Xion/cargo-download/blob/master/src/main.rs
fn download_crate(pallet: &Dependency, reg_dl_url: String) -> CliResult<Vec<u8>> {
    // Check if {crate} & {version} markers are present, if yes replace,
    // if not, assume {crate}/{version}/download URI
    let name = pallet.name.as_str();
    let version = pallet.version().unwrap();
    let download_url = reg_dl_url
        .replace("{crate}", name)
        .replace("{version}", version);

    debug!(
        "Downloading pallet crate `{} v{}` from {}",
        name, version, download_url
    );

    let mut response = reqwest::get(&download_url)?;

    let content_length: Option<usize> = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok());

    let mut bytes = match content_length {
        Some(cl) => Vec::with_capacity(cl),
        None => Vec::new(),
    };
    response.read_to_end(&mut bytes)?;

    debug!("Pallet crate `{} v{}` downloaded", name, version);
    Ok(bytes)
}
