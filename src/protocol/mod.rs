pub mod commands;
pub mod curl;
pub mod resp;

pub(crate) trait Protocol {
    fn decode(raw: &[u8]) -> Result<commands::Command, ProtocolError>;
    fn encode(command: commands::CommandResponse) -> Vec<u8>;
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ProtocolError {
    #[error("command not recognized {0}")]
    CommandNotRecognized(String),

    #[error("curl protocol decoding error {0}")]
    CurlProtocolDecodingError(String),
}
