#[derive(Default, serde::Deserialize)]
pub struct Config {
    pub logger: Logger,
    pub engine: Engine,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        toml::from_str(&config_file)
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
    pub persistence: Option<Persistence>,
}

#[derive(Default, serde::Deserialize)]
pub struct Persistence {
    pub enabled: bool,

    #[serde(default = "default_path")]
    pub path: String,

    #[serde(default = "default_flush_interval")]
    #[serde(with = "humantime_serde")]
    pub flush_interval: std::time::Duration,
}

fn default_flush_interval() -> std::time::Duration {
    std::time::Duration::from_secs(10)
}

fn default_path() -> String {
    "./tmp/umd".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config_file = r#"
            [logger]
            level = "warn"

            [engine]
            max_items = 99

            [engine.persistence]
            enabled = true
            path = "./tmp/umd"
            flush_interval = "15s"
        "#;

        let config = Config::new(config_file);
        assert_eq!(config.logger.level, "warn");
        assert_eq!(config.engine.max_items, Some(99));
        let p = config.engine.persistence.as_ref().unwrap();
        assert_eq!(p.enabled, true);
        assert_eq!(p.path, "./tmp/umd");
        assert_eq!(p.flush_interval.as_secs(), 15);
    }
}
