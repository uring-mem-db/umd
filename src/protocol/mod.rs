pub mod curl;
pub mod resp;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Get the value of key.
    /// If the key does not exist the special value nil is returned.
    /// An error is returned if the value stored at key is not a string, because GET only handles string values.
    Get {
        key: String,
    },

    /// Set key to hold the string value.
    /// If key already holds a value, it is overwritten, regardless of its type.
    Set {
        key: String,
        value: String,
    },

    /// Removes the specified keys. A key is ignored if it does not exist.
    Del {
        key: String,
    },

    // FIXME: not sure what it does, but it's the first sent by redis-cli
    COMMAND,
}

impl Command {
    fn new(kind: &str, key: &str, value: Option<String>) -> Self {
        let key = key.trim_matches('/').to_string();
        let kind = kind.to_lowercase();
        match kind.as_str() {
            "get" => Command::Get { key },
            "post" | "set" => match value {
                Some(v) => Command::Set {
                    key,
                    value: v.trim().to_string(),
                },
                None => Command::Del { key },
            },
            "del" => Command::Del { key },
            _ => unimplemented!("not implemented"),
        }
    }
}

#[derive(Debug)]
pub enum CommandResponse {
    String { value: String },
    Integer { value: i64 },
    Array { value: Vec<CommandResponse> },
    Err { value: String },
}

pub(crate) trait Protocol {
    fn decode(raw: &[u8]) -> Result<Command, String>;
    fn encode(command: CommandResponse) -> Vec<u8>;
}
