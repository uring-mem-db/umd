pub mod commands;
pub mod curl;
pub mod resp;

pub(crate) trait Protocol {
    fn decode(raw: &[u8]) -> Result<commands::Command, String>;
    fn encode(command: commands::CommandResponse) -> Vec<u8>;
}
