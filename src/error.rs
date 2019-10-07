use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Dependency(String),
    Generic(String),
    Io(io::Error),
    Manifest(String),
    Metadata(String),
    Registry(String),
    Toml(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Self::Dependency(ref e) => write!(f, "{}", e),
            Self::Generic(ref e) => write!(f, "{}", e),
            Self::Io(ref e) => write!(f, "{}", e),
            Self::Manifest(ref e) => write!(f, "{}", e),
            Self::Metadata(ref e) => write!(f, "{}", e),
            Self::Registry(ref e) => write!(f, "{}", e),
            Self::Toml(ref e) => write!(f, "Could not parse toml file: {}", e),
        }
    }
}

impl CliError {
    /// Print this error and immediately exit the program.
    pub fn exit(&self) -> ! {
        eprintln!("error: {}", self);
        ::std::process::exit(1)
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<git2::Error> for CliError {
    fn from(err: git2::Error) -> Self {
        Self::Metadata(format!("Error reading module metadata: {}", err))
    }
}

impl From<json::Error> for CliError {
    fn from(err: json::Error) -> Self {
        Self::Metadata(format!("Error reading module metadata: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for CliError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Metadata(format!("Error reading module metadata: {}", err))
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        Self::Metadata(format!("Error reading module metadata: {}", err))
    }
}

impl From<regex::Error> for CliError {
    fn from(err: regex::Error) -> Self {
        Self::Generic(format!("Invalid regex definition: {}", err))
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(format!("Could not parse input as TOML: {}", err))
    }
}
