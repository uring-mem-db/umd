pub mod commands;
pub mod curl;
pub mod resp;

pub trait Protocol {
    fn decode(raw: &[u8]) -> Result<commands::Command, ProtocolError>;
    fn encode(command: commands::CommandResponse) -> Vec<u8>;
}

#[allow(clippy::module_name_repetitions)]
#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    #[error("command not recognized {0}")]
    /// Error for when a command is not recognized, but decoding was successful.
    CommandNotRecognized(String),

    #[error("curl protocol decoding error")]
    /// Generic error for curl protocol decoding, maybe it is just another protocol.
    CurlProtocolDecodingError,

    #[error("resp protocol decoding error")]
    /// Generic error for RESP protocol decoding, maybe it is just another protocol.
    RespProtocolDecodingError,
}
