use super::{commands::Command, commands::CommandResponse, Protocol};

/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings, and Arrays.
pub struct Resp {}

struct RespDecoder<I: Iterator<Item = char>> {
    chars: I,
}

impl<I: Iterator<Item = char>> RespDecoder<I> {
    fn new(chars: I) -> Self {
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

                    let mut items: Vec<RespType> = Vec::with_capacity(len as usize);

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

    fn try_from(value: String) -> Result<Self, <RespType as TryFrom<String>>::Error> {
        let it = value.chars();
        let mut rd = RespDecoder::new(it);

        rd.next_chunk()
    }
}

impl Protocol for Resp {
    fn decode(raw: &[u8]) -> Result<Command, String> {
        let s = String::from_utf8(raw.to_vec()).map_err(|_| "Error while decoding RESP")?;
        if s.contains("PING") {
            return Ok(Command::Ping);
        }

        let rt = RespType::try_from(s)?;
        match rt {
            RespType::SimpleString { value } => todo!(),
            RespType::Error { value } => todo!(),
            RespType::Integer { value } => todo!(),
            RespType::BulkString { value } => todo!(),
            RespType::Array { value } => {
                let mut it = value.into_iter();
                let operation = if let Some(s) = it.next() {
                    match s {
                        RespType::SimpleString { value } | RespType::BulkString { value } => value,
                        _ => panic!(),
                    }
                } else {
                    panic!()
                };

                let key = if let Some(s) = it.next() {
                    match s {
                        RespType::SimpleString { value } | RespType::BulkString { value } => value,
                        _ => panic!(),
                    }
                } else {
                    panic!()
                };

                let v = if let Some(s) = it.next() {
                    match s {
                        RespType::SimpleString { value } | RespType::BulkString { value } => {
                            Some(value)
                        }
                        _ => panic!(),
                    }
                } else {
                    None
                };

                let options = it
                    .map(|v| match v {
                        RespType::BulkString { value } => value,
                        _ => panic!(),
                    })
                    .collect::<Vec<String>>();

                let c = Command::new(&operation, &key, v, options);
                Ok(c)
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

                for el in value.into_iter() {
                    s.append(&mut Self::encode(el));
                }

                s
            }
            CommandResponse::Err { value } => format!("-{value}").as_bytes().to_vec(),
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
        assert!(cmd == Command::Config);
    }

    #[test]
    fn command_command() {
        let s = "PING\r\n";
        let cmd = Resp::decode(s.as_bytes()).unwrap();
        assert!(cmd == Command::Ping);
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
            let s = "*2\r\n*2\r\n:1\r\n+OK\r\n*2\r\n:4\r\n+TEST\r\n".to_string();
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
        assert!(
            cmd == Command::Set {
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
}
