#[derive(Default, serde::Deserialize)]
pub struct Config {
    pub logger: Logger,
    pub engine: Engine,
}

impl Config {
    pub fn new(config_file: &str) -> Self {
        toml::from_str(config_file)
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

#[derive(Clone, Default, serde::Deserialize)]
pub struct Engine {
    /// Maximum number of items in the cache, if None, the cache is unbounded.
    pub max_items: Option<u64>,
    pub persistence: Option<Persistence>,
}

#[derive(Clone, Default, serde::Deserialize)]
pub struct Persistence {
    /// Enable or disable persistence
    pub enabled: bool,

    /// File to persist the data
    #[serde(default = "default_file")]
    pub file: String,

    /// Flush the data to the file every N changes
    #[serde(default = "default_flush_every_changes")]
    pub flush_every_changes: u64,
}

const fn default_flush_every_changes() -> u64 {
    1000
}

fn default_file() -> String {
    "./tmp/umd/persistence.bin".to_string()
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
            file = "/tmp/umd/persistence.bin"
            flush_every_changes = 10
        "#;

        let config = Config::new(config_file);
        assert_eq!(config.logger.level, "warn");
        assert_eq!(config.engine.max_items, Some(99));
        let p = config.engine.persistence.as_ref().unwrap();
        assert_eq!(p.enabled, true);
        assert_eq!(p.file, "/tmp/umd/persistence.bin");
        assert_eq!(p.flush_every_changes, 10);
    }
}
