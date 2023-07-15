use super::{Command, Protocol};

/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings, and Arrays.
pub struct Resp {}

#[derive(Debug, PartialEq)]
pub(crate) enum RespType {
    SimpleString { value: String },
    Error { value: String },
    Integer { value: i64 },
    BulkString { value: String },
    Array { value: Vec<RespType> },
}

impl RespType {
    fn decode_integer(chars: std::str::Chars) -> Result<i64, &'static str> {
        let s: String = chars.take_while(|char| *char != '\r').collect();

        s.as_str()
            .parse::<i64>()
            .map_err(|_| "Error while parsing integer")
    }
}

impl TryFrom<String> for RespType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, <RespType as TryFrom<String>>::Error> {
        let mut it = value.chars();

        // Remove the last two characters
        it.next_back();
        it.next_back();

        match it.next() {
            Some(c) => match c {
                '+' => Ok(Self::SimpleString {
                    value: it.collect(),
                }),
                '-' => Ok(Self::Error {
                    value: it.collect(),
                }),
                ':' => Self::decode_integer(it).map(|v| Self::Integer { value: v }),
                '$' => {
                    let len = Self::decode_integer(it)?;
                    let v = value.as_bytes();
                    Ok(Self::BulkString {
                        value: String::from_utf8(v[4..4 + len as usize].to_vec())
                            .map_err(|_| "Error while parsing RESP")?,
                    })
                }
                '*' => {
                    let len = Self::decode_integer(it.clone())?;
                    let mut v = Vec::with_capacity(len as usize);
                    let mut items = value
                        .split("\r\n")
                        .skip(1)
                        .map(|v| v.to_owned())
                        .collect::<Vec<String>>();
                    items.remove(items.len() - 1);

                    let chunk = if items[0].starts_with(':') { 1 } else { 2 };

                    for item in items.chunks(chunk) {
                        let mut tmp = item.join("\r\n");
                        tmp.push('\r');
                        tmp.push('\n');

                        v.push(Self::try_from(tmp)?);
                    }

                    Ok(Self::Array { value: v })
                }
                _ => Err("Invalid first character in RESP"),
            },
            None => Err("Error while parsing RESP"),
        }
    }
}

impl Protocol for Resp {
    fn decode(raw: &[u8]) -> Result<Command, String> {
        let s = String::from_utf8(raw.to_vec()).map_err(|_| "Error while decoding RESP")?;
        let rt = RespType::try_from(s)?;
        match rt {
            RespType::SimpleString { value } => todo!(),
            RespType::Error { value } => todo!(),
            RespType::Integer { value } => todo!(),
            RespType::BulkString { value } => todo!(),
            RespType::Array { value } => Ok(Command::COMMAND),
        }
    }

    fn encode(_command: Command) -> Vec<u8> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
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
    fn test_error() {
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
    fn test_integer() {
        let s = ":574\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(rt, Ok(RespType::Integer { value: 574 }));

        let s = ":-574\r\n".to_string();
        let rt = RespType::try_from(s);

        assert_eq!(rt, Ok(RespType::Integer { value: -574 }))
    }

    #[test]
    fn test_bulk_string() {
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
    fn test_array() {
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
    }
}
