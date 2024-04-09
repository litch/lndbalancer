use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Clone, Serialize)]
pub struct Source {
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
pub struct Config {
    pub application_port: u16,
    pub sources: Vec<Source>,
    pub dynamic_fees: bool,
    pub dynamic_fee_update_frequency: u64,
    pub dynamic_fee_intervals: u64,
    pub dynamic_fee_min: u64,
    pub dynamic_fee_max: u64,

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
            dynamic_fees: true,
            dynamic_fee_update_frequency: 100,
            dynamic_fee_intervals: 5,
            dynamic_fee_min: 100,
            dynamic_fee_max: 1000,

        }
    }
}


impl Config {


    pub fn current() -> Arc<Config> {
        CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
    }
    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Config>> = RwLock::new(Default::default());
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
