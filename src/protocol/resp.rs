use super::{
    commands::{Command, CommandResponse},
    Protocol, ProtocolError,
};

/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings, and Arrays.
pub struct Resp {}

struct RespDecoder<I: Iterator<Item = char>> {
    chars: I,
}

impl<I: Iterator<Item = char>> RespDecoder<I> {
    const fn new(chars: I) -> Self {
        Self { chars }
    }

    fn take_next_string(&mut self) -> String {
        let s = self
            .chars
            .by_ref()
            .take_while(|char| *char != '\r')
            .collect();

        self.chars.next(); // removing \n

        s
    }

    fn decode_integer(&mut self) -> Result<i64, &'static str> {
        self.take_next_string()
            .parse()
            .map_err(|_| "Error while parsing integer")
    }

    fn next_chunk(&mut self) -> Result<RespType, &'static str> {
        match self.chars.next() {
            Some(c) => match c {
                '+' => Ok(RespType::SimpleString {
                    value: self.take_next_string(),
                }),
                '-' => Ok(RespType::Error {
                    value: self.take_next_string(),
                }),
                ':' => self
                    .decode_integer()
                    .map(|i| RespType::Integer { value: i }),
                '$' => {
                    let len = self.decode_integer()?;

                    if len == -1 {
                        return Ok(RespType::None);
                    }

                    Ok(RespType::BulkString {
                        value: self.take_next_string(),
                    })
                }
                '*' => {
                    let len = self.decode_integer()?;

                    if len == -1 {
                        return Ok(RespType::None);
                    }

                    let mut items = Vec::with_capacity(
                        usize::try_from(len).map_err(|_| "Error while parsing array")?,
                    );

                    for _ in 0..len {
                        if let Ok(rt) = self.next_chunk() {
                            items.push(rt);
                        } else {
                            return Err("Error while parsing array");
                        }
                    }

                    Ok(RespType::Array { value: items })
                }
                _ => Err("Invalid first character in RESP"),
            },
            None => Err("Error while parsing RESP"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum RespType {
    SimpleString { value: String },
    Error { value: String },
    Integer { value: i64 },
    BulkString { value: String },
    Array { value: Vec<RespType> },
    None,
}

impl TryFrom<String> for RespType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        let it = value.chars();
        let mut rd = RespDecoder::new(it);

        rd.next_chunk()
    }
}

impl TryFrom<&str> for RespType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let it = value.chars();
        let mut rd = RespDecoder::new(it);

        rd.next_chunk()
    }
}

impl Protocol for Resp {
    fn decode(raw: &[u8]) -> Result<Command, ProtocolError> {
        let s = String::from_utf8(raw.to_vec())
            .map_err(|_| ProtocolError::RespProtocolDecodingError)?;

        if s.contains("PING") {
            return Ok(Command::Ping);
        }

        let rt = RespType::try_from(s).map_err(|_| ProtocolError::RespProtocolDecodingError)?;
        match rt {
            RespType::SimpleString { .. } => todo!(),
            RespType::Error { .. } => todo!(),
            RespType::Integer { .. } => todo!(),
            RespType::BulkString { .. } => todo!(),
            RespType::Array { value } => {
                let mut it = value.into_iter();
                let operation = it.next().map_or_else(
                    || panic!(),
                    |s| match s {
                        RespType::SimpleString { value } | RespType::BulkString { value } => value,
                        _ => panic!(),
                    },
                );

                let key = if let Some(s) = it.next() {
                    match s {
                        RespType::SimpleString { value } | RespType::BulkString { value } => value,
                        _ => panic!(),
                    }
                } else {
                    // No key means, single command
                    return Command::new(&operation, "", None, &[]);
                };

                let v = it.next().map(|v| match v {
                    RespType::SimpleString { value } | RespType::BulkString { value } => value,
                    _ => panic!("array can be only of strings"),
                });

                let options = it
                    .map(|v| {
                        if let RespType::BulkString { value } = v {
                            value
                        } else {
                            panic!()
                        }
                    })
                    .collect::<Vec<String>>();

                Command::new(&operation, &key, v, &options)
            }
            RespType::None => todo!(),
        }
    }

    fn encode(response: CommandResponse) -> Vec<u8> {
        match response {
            CommandResponse::String { value } => {
                let s = format!("+{value}\r\n");
                tracing::debug!(response = s, "resp");
                s.as_bytes().to_vec()
            }
            CommandResponse::Integer { value } => format!("*{value}\r\n").as_bytes().to_vec(),
            CommandResponse::Array { value } => {
                let mut s = format!("*{}\r\n", value.len()).as_bytes().to_vec();

                for el in value {
                    s.append(&mut Self::encode(el));
                }

                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_command() {
        let s = "*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$4\r\nsave\r\n*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$10\r\nappendonly\r\n";
        let cmd = Resp::decode(s.as_bytes()).unwrap();
        assert_eq!(cmd, Command::Config);
    }

    #[test]
    fn command_command() {
        let s = "PING\r\n";
        let cmd = Resp::decode(s.as_bytes()).unwrap();
        assert_eq!(cmd, Command::Ping);
    }

    #[test]
    fn simple_string() {
        let s = "+OK\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(
            rt,
            Ok(RespType::SimpleString {
                value: "OK".to_string()
            })
        )
    }

    #[test]
    fn error() {
        let s = "-ERR: test\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(
            rt,
            Ok(RespType::Error {
                value: "ERR: test".to_string()
            })
        )
    }

    #[test]
    fn integer() {
        let s = ":574\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(rt, Ok(RespType::Integer { value: 574 }));

        let s = ":-574\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(rt, Ok(RespType::Integer { value: -574 }))
    }

    #[test]
    fn bulk_string() {
        {
            // random string
            let s = "$5\r\nhello\r\n".to_string();
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::BulkString {
                    value: "hello".to_string()
                })
            )
        }
        {
            // empty string
            let s = "$0\r\n\r\n".to_string();
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::BulkString {
                    value: "".to_string()
                })
            )
        }
    }

    #[test]
    fn array() {
        {
            // string array
            let s = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".to_string();
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::Array {
                    value: vec![
                        RespType::BulkString {
                            value: "hello".to_string()
                        },
                        RespType::BulkString {
                            value: "world".to_string()
                        }
                    ]
                })
            )
        }
        {
            // int array
            let s = "*3\r\n:1\r\n:2\r\n:3\r\n".to_string();
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::Array {
                    value: vec![
                        RespType::Integer { value: 1 },
                        RespType::Integer { value: 2 },
                        RespType::Integer { value: 3 }
                    ]
                })
            )
        }
        {
            // heterogeneous array
            let s = "*3\r\n:1\r\n+OK\r\n$5\r\nhello\r\n".to_string();
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::Array {
                    value: vec![
                        RespType::Integer { value: 1 },
                        RespType::SimpleString {
                            value: "OK".to_string()
                        },
                        RespType::BulkString {
                            value: "hello".to_string()
                        }
                    ]
                })
            )
        }
        {
            // array of array
            let s = "*2\r\n*2\r\n:1\r\n+OK\r\n*2\r\n:4\r\n+TEST\r\n";
            let rt = RespType::try_from(s);

            assert_eq!(
                rt,
                Ok(RespType::Array {
                    value: vec![
                        RespType::Array {
                            value: vec![
                                RespType::Integer { value: 1 },
                                RespType::SimpleString {
                                    value: "OK".to_string()
                                }
                            ]
                        },
                        RespType::Array {
                            value: vec![
                                RespType::Integer { value: 4 },
                                RespType::SimpleString {
                                    value: "TEST".to_string()
                                }
                            ]
                        }
                    ]
                })
            )
        }
    }

    #[test]
    fn set_request() {
        let s = "*3\r\n$3\r\nset\r\n$4\r\nciao\r\n$4\r\ncome\r\n";
        let cmd = Resp::decode(s.as_bytes()).unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                key: "ciao".to_string(),
                value: "come".to_string(),
                ttl: None,
            }
        );
    }

    #[test]
    fn set_with_expire() {
        let s = "*5\r\n$3\r\nset\r\n$4\r\nciao\r\n$4\r\ncome\r\n$2\r\nEX\r\n$2\r\n10\r\n";
        let cmd = Resp::decode(s.as_bytes()).unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                key: "ciao".to_string(),
                value: "come".to_string(),
                ttl: Some(std::time::Duration::from_secs(10)),
            }
        );
    }

    #[test]
    fn dirty() {
        let payload = r#"hello"#;
        let cmd = Resp::decode(payload.as_bytes());
        assert!(cmd.is_err());
        assert_eq!(cmd, Err(ProtocolError::RespProtocolDecodingError));

        let payload = "*3\r\n$3\r\nNOTACOMMAND\r\n$1\r\nx\r\n$2\r\n11\r\n";
        let cmd = Resp::decode(payload.as_bytes());
        assert!(cmd.is_err());
        assert_eq!(
            cmd,
            Err(ProtocolError::CommandNotRecognized("notacommand".into()))
        );
    }
}
