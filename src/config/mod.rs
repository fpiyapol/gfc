use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to open config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ResourcesConfig {
    pub projects_dir: String,
    pub repositories_dir: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub resources: ResourcesConfig,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut file: File = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn given_valid_yaml_when_loaded_then_config_is_parsed_correctly() {
        let yaml = r#"
            server:
            host: 127.0.0.1
            port: 8080
            resources:
            projects_dir: /tmp/projects
            repositories_dir: /tmp/repos
            "#;
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "{}", yaml).unwrap();

        let config = Config::from_file(tmpfile.path());

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.resources.projects_dir, "/tmp/projects");
        assert_eq!(config.resources.repositories_dir, "/tmp/repos");
    }

    #[test]
    fn given_invalid_yaml_when_loaded_then_returns_error() {
        let yaml = "not: valid: yaml";
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "{}", yaml).unwrap();

        let config = Config::from_file(tmpfile.path());

        assert!(config.is_err());
    }
}
