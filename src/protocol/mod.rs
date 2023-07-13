pub mod resp;

pub(crate) enum Command {
    _Set { key: String, value: String },
    _Get { key: String },
    _Del { key: String },
}

pub(crate) trait Protocol {
    fn decode(raw: Vec<u8>) -> Result<Command, &'static str>;
    fn encode(command: Command) -> Vec<u8>;
}
