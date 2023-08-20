#[derive(Default, serde::Deserialize)]
pub struct Config {
    pub logger: Logger,
    pub engine: Engine,
}

impl Config {
    pub fn new() -> Self {
        let config_file = std::fs::read_to_string("configs/local.toml");
        if config_file.is_err() {
            return Self::default();
        }

        toml::from_str(&config_file.unwrap())
            .map_err(|e| e.to_string())
            .unwrap()
    }
}

#[derive(Default, serde::Deserialize)]
pub struct Logger {
    #[serde(default = "default_level")]
    pub level: String,
}

fn default_level() -> String {
    "info".to_string()
}

#[derive(Default, serde::Deserialize)]
pub struct Engine {
    pub max_items: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = Config::new();
        assert_eq!(config.logger.level, "info");
    }
}
