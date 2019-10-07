use self::code_from_cargo::Kind;
use crate::error::*;

use std::path::{Path, PathBuf};

use cargo_edit::registry_url;
use url::Url;

// From https://github.com/tofay/cargo-edit/blob/alt-registries/src/registry.rs

fn cargo_home() -> CliResult<PathBuf> {
    let default_cargo_home = dirs::home_dir()
        .map(|x| x.join(".cargo"))
        .ok_or_else(|| CliError::Generic("Error reading cargo home dir.".to_owned()))?;
    // .chain_err(|| ErrorKind::ReadHomeDirFailure)?;
    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or(default_cargo_home);
    Ok(cargo_home)
}

pub fn registry_path(manifest_path: &Path, registry: Option<&str>) -> CliResult<PathBuf> {
    registry_path_from_url(
        &registry_url(manifest_path, registry).map_err(|e| CliError::Registry(e.to_string()))?,
    )
}

pub fn registry_path_from_url(registry: &Url) -> CliResult<PathBuf> {
    Ok(cargo_home()?
        .join("registry")
        .join("index")
        .join(short_name(registry)))
}

fn short_name(registry: &Url) -> String {
    // ref: https://github.com/rust-lang/cargo/blob/4c1fa54d10f58d69ac9ff55be68e1b1c25ecb816/src/cargo/sources/registry/mod.rs#L386-L390
    #![allow(deprecated)]
    use std::hash::{Hash, Hasher, SipHasher};

    let mut hasher = SipHasher::new_with_keys(0, 0);
    Kind::Registry.hash(&mut hasher);
    registry.as_str().hash(&mut hasher);
    let hash = hex::encode(hasher.finish().to_le_bytes());

    let ident = registry.host_str().unwrap_or("").to_string();

    format!("{}-{}", ident, hash)
}

mod code_from_cargo {
    #![allow(dead_code)]

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Kind {
        Git(GitReference),
        Path,
        Registry,
        LocalRegistry,
        Directory,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum GitReference {
        Tag(String),
        Branch(String),
        Rev(String),
    }
}
