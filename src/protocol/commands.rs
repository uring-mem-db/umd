#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Get the value of key.
    /// If the key does not exist the special value nil is returned.
    /// An error is returned if the value stored at key is not a string, because GET only handles string values.
    Get { key: String },

    /// Set key to hold the string value.
    /// If key already holds a value, it is overwritten, regardless of its type.
    Set {
        key: String,
        value: String,
        ttl: Option<std::time::Duration>,
    },

    /// Removes the specified keys. A key is ignored if it does not exist.
    Del { key: String },

    /// Return documentary information about commands.
    COMMAND_DOCS,

    /// Returns the config for the server instance.
    Config,

    /// Returns the server's liveliness response.
    Ping,

    /// Increments the number stored at key by one.
    /// If the key does not exist, it is set to 0 before performing the operation
    Incr { key: String },
}

impl Command {
    pub fn new(kind: &str, key: &str, value: Option<String>, options: Vec<String>) -> Self {
        let key = key.trim_matches('/').to_string();
        let kind = kind.to_lowercase();
        match kind.as_str() {
            "command" if key == "DOCS" => Command::COMMAND_DOCS,
            "get" => Command::Get { key },
            "post" | "set" => match value {
                Some(v) => Command::Set {
                    key,
                    value: v.trim().to_string(),
                    ttl: if options.is_empty() {
                        None
                    } else {
                        if options.len() != 2 {
                            tracing::info!("Options for set not supported");
                        }

                        let cmd = &options[0];
                        let value = options[1].parse().unwrap();
                        if cmd == "EX" {
                            Some(std::time::Duration::from_secs(value))
                        } else {
                            tracing::info!("Options for set not supported");
                            None
                        }
                    },
                },
                None => Command::Del { key },
            },
            "del" => Command::Del { key },
            "config" => Command::Config,
            "ping" => Command::Ping,
            "incr" => Command::Incr { key },
            _ => unimplemented!("not implemented"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandResponse {
    String { value: String },
    Integer { value: i64 },
    Array { value: Vec<CommandResponse> },
    Err { value: String },
}
