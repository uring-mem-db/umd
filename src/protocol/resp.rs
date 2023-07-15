use super::{Command, Protocol};

/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings, and Arrays.
pub(crate) struct Resp {}

#[derive(Debug, PartialEq)]
pub(crate) enum RespType {
    SimpleString { value: String },
    Error { value: String },
    Integer { value: i64 },
    BulkString { value: String },
    _Array { value: Vec<Box<RespType>> },
}

impl RespType {
    fn decode_integer(chars: std::str::Chars) -> Result<i64, &'static str> {
        let s: String = chars.take_while(|char| *char != '\r').collect();

        i64::from_str_radix(s.as_str(), 10).map_err(|_| "Error while parsing integer")
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
                    println!("{v:?}");
                    Ok(Self::BulkString {
                        value: String::from_utf8(v[4..4 + len as usize].to_vec())
                            .map_err(|_| "Error while parsing RESP")?,
                    })
                }
                '*' => unimplemented!(),
                _ => Err("Invalid first character in RESP"),
            },
            None => Err("Error while parsing RESP"),
        }
    }
}

impl Protocol for Resp {
    fn decode(raw: Vec<u8>) -> Result<Command, &'static str> {
        let _s = match String::from_utf8(raw) {
            Ok(v) => v,
            Err(_) => return Err("Error while decoding RESP"),
        };

        unimplemented!()
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
}
