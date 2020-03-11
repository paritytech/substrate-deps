use regex::Regex;
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
}

impl Package {
    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn version(&self) -> &str {
        &self.version
    }
}
