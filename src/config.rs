/// Config via env vars for the app
use serde::{Deserialize, Serialize};

use crate::shortener::ShorteningStrategy;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_digest")]
    shortener_digest: String,
    #[serde(default = "default_shortener_length")]
    shortener_length: usize,
}

impl Config {
    pub fn strategy(&self) -> ShorteningStrategy {
        match self.shortener_digest.as_str() {
            "sha256" => ShorteningStrategy::Sha256 {
                length: self.shortener_length,
            },
            _ => panic!("Unknown digest algorithm {}", self.shortener_digest),
        }
    }
}

fn default_host() -> String {
    "https://tier.app".into()
}

fn default_digest() -> String {
    "sha256".into()
}

fn default_shortener_length() -> usize {
    8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            database_url: "postgres://user:password@host:5432/db".into(),
            host: "https://tier.app".into(),
            shortener_digest: "sha256".into(),
            shortener_length: 8,
        }
    }

    #[test]
    fn test_strategy() {
        let config = test_config();
        assert_eq!(config.strategy(), ShorteningStrategy::Sha256 { length: 8 });
    }
}
