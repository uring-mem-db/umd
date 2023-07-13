use std::str::Chars;

use super::{Command, Protocol};

pub(crate) struct Resp {}

#[derive(Debug, PartialEq)]
pub(crate) enum RespType {
    SimpleString { value: String },
    Error { value: String },
    Integer { value: i64 },
    _BulkString { value: String },
    _Array { value: Vec<Box<RespType>> },
}

impl RespType {
    fn decode_simple_string(chars: Chars) -> String {
        chars.take_while(|char| *char != '\r').collect()
    }

    fn decode_integer(chars: Chars) -> Result<i64, &'static str> {
        let s: String = chars.take_while(|char| *char != '\r').collect();

        i64::from_str_radix(s.as_str(), 10).map_err(|_| "Error while parsing integer")
    }
}

impl TryFrom<String> for RespType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, <RespType as TryFrom<String>>::Error> {
        let mut it = value.chars();

        match it.next() {
            Some(c) => match c {
                '+' => Ok(Self::SimpleString {
                    value: Self::decode_simple_string(it.clone()),
                }),
                '-' => Ok(Self::Error {
                    value: Self::decode_simple_string(it.clone()),
                }),
                ':' => Self::decode_integer(it.clone()).map(|v| Self::Integer { value: v }),
                '$' => unimplemented!(),
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
    use super::RespType;

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
}
