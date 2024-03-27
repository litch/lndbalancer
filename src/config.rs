use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Clone, Serialize)]
pub(crate) struct Source {
    pub endpoint: String,
    pub macaroon: String,
    pub cert: String,
}

impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Source {{ endpoint: {:?}, macaroon: {:?}, cert: {:?} }}",
            self.endpoint,
            self.macaroon,
            self.cert
        )
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub(crate) struct Config {
    pub application_port: u16,
    pub sources: Vec<Source>,
}

impl Config {
    pub fn new() -> Self {
        let config_path = std::env::args()
            .nth(1)
            .unwrap_or_else(|| String::from("config.yaml"));
        match std::fs::File::open(config_path) {
            Ok(config_file) => {
                let config: Config =
                    serde_yaml::from_reader(config_file).expect("yaml formating error");
                config
            }
            Err(_) => {
                println!("No config file found, using default configuration");
                let c = Config::default();
                println!("Config: {:?}", &c);
                c
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            application_port: 8080,
            sources: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = Config::new();
        assert_eq!(config.application_port, 8080);
    }
}
