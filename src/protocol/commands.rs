use super::ProtocolError;

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

    /// Exists returns if key exists.
    Exists { key: String },

    /// Return documentary information about commands.
    Docs,

    /// Returns the config for the server instance.
    Config,

    /// Returns the server's liveliness response.
    Ping,

    /// Increments the number stored at key by one.
    /// If the key does not exist, it is set to 0 before performing the operation
    Incr { key: String },

    /// Removes all keys from the current database.
    FlushDb,
}

impl Command {
    pub fn new(
        kind: &str,
        key: &str,
        value: Option<String>,
        options: Vec<String>,
    ) -> Result<Self, ProtocolError> {
        let key = key.trim_matches('/').to_string();
        let kind = kind.to_lowercase();

        let cmd = match kind.as_str() {
            "command" if key == "DOCS" => Command::Docs,
            "get" => Command::Get { key },
            "set" => make_set(key, value.unwrap(), options),
            "post" => match value {
                Some(v) => make_set(key, v, options),
                None => Command::Del { key },
            },
            "del" => Command::Del { key },
            "exists" => Command::Exists { key },
            "config" => Command::Config,
            "ping" => Command::Ping,
            "incr" => Command::Incr { key },
            "flushdb" => Command::FlushDb,
            _ => return Err(ProtocolError::CommandNotRecognized(kind)),
        };

        Ok(cmd)
    }
}

fn make_set(key: String, v: String, options: Vec<String>) -> Command {
    Command::Set {
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
    }
}

#[derive(PartialEq, Debug)]
pub enum CommandResponse {
    String { value: String },
    Integer { value: i64 },
    Array { value: Vec<CommandResponse> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command_ok() {
        let cmd = Command::new("GET", "key", None, vec![]);
        assert_eq!(
            cmd.unwrap(),
            Command::Get {
                key: "key".to_string()
            }
        );
    }

    #[test]
    fn test_new_command_error() {
        let cmd = Command::new("abc", "key", None, vec![]);
        assert_eq!(
            cmd,
            Err(ProtocolError::CommandNotRecognized("abc".to_string()))
        );
    }
}
