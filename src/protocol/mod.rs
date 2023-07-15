pub mod resp;

#[derive(Debug)]
pub(crate) enum Command {
    _Set { key: String, value: String },
    _Get { key: String },
    _Del { key: String },
    COMMAND,
}

pub(crate) trait Protocol {
    fn decode(raw: &[u8]) -> Result<Command, String>;
    fn encode(command: Command) -> Vec<u8>;
}
