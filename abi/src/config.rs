use std::path::Path;

use serde::{Deserialize, Serialize};
use std::fs;

use crate::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let f = fs::read_to_string(path)?;
        let config = toml::from_str(&f).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let f = toml::to_string(self).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        fs::write(path, f).map_err(Error::IoError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_config() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_config.toml");
        let config = Config {
            db: DbConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "postgres".to_string(),
                password: "password".to_string(),
                database: "reservation".to_string(),
            },
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
            },
        };
        let result = || -> Result<Config, Error> {
            config.save(&path)?;
            Config::load(&path)
        }();
        fs::remove_file(&path).unwrap();
        assert_eq!(result.unwrap(), config);
    }
}
