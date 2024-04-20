use super::{
    commands::{Command, CommandResponse},
    Protocol, ProtocolError,
};

pub struct Curl {}

impl Protocol for Curl {
    fn decode(raw: &[u8]) -> Result<Command, ProtocolError> {
        let request = String::from_utf8(raw.to_vec())
            .map_err(|_| ProtocolError::CurlProtocolDecodingError)?;
        let request = request.trim();
        let mut lines = request.lines();
        let first_line = lines
            .next()
            .ok_or(ProtocolError::CurlProtocolDecodingError)?;
        let mut parts = first_line.split_whitespace();
        let method = parts
            .next()
            .ok_or(ProtocolError::CurlProtocolDecodingError)?;
        let path = parts
            .next()
            .ok_or(ProtocolError::CurlProtocolDecodingError)?;
        let _version = parts
            .next()
            .ok_or(ProtocolError::CurlProtocolDecodingError)?;
        let mut headers = std::collections::HashMap::new();
        let mut body = None;
        let mut options = vec![];
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                let v = line
                    .trim()
                    .split(' ')
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>();
                options = v[1..].to_vec();
                body = Some(v[0].to_string());
            }
        }

        Command::new(method, path, body, options)
    }

    fn encode(response: CommandResponse) -> Vec<u8> {
        let mut s = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        let body = match response {
            CommandResponse::String { value } => value,
            _ => panic!(),
        };

        s.push_str(&body);

        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_with_ttl() {
        let raw = r#"POST /key HTTP/1.1
        Host: localhost:9999
        User-Agent: curl/7.74.0
        Accept: */*
        Content-Length: 5
        Content-Type: application/x-www-form-urlencoded

        value EX 10"#;

        let output = Curl::decode(raw.as_bytes()).unwrap();
        assert_eq!(
            output,
            Command::Set {
                key: "key".to_string(),
                value: "value".to_string(),
                ttl: Some(std::time::Duration::from_secs(10)),
            }
        );
    }

    #[test]
    fn parse_get() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = Curl::decode(raw.as_bytes()).unwrap();
        assert_eq!(
            output,
            Command::Get {
                key: "key".to_string()
            }
        );
    }

    #[test]
    fn parse_set() {
        let raw = r#"POST /key HTTP/1.1
        Host: localhost:9999
        User-Agent: curl/7.74.0
        Accept: */*
        Content-Length: 5
        Content-Type: application/x-www-form-urlencoded

        value"#;

        let output = Curl::decode(raw.as_bytes()).unwrap();
        assert_eq!(
            output,
            Command::Set {
                key: "key".to_string(),
                value: "value".to_string(),
                ttl: None,
            }
        );
    }

    #[test]
    fn parse_del() {
        let raw = r#"POST /key HTTP/1.1
        Host: localhost:9999
        User-Agent: curl/7.74.0
        Accept: */*
        Content-Length: 5
        Content-Type: application/x-www-form-urlencoded
"#;

        let output = Curl::decode(raw.as_bytes()).unwrap();
        assert_eq!(
            output,
            Command::Del {
                key: "key".to_string(),
            }
        );
    }
}
