use super::{Command, CommandResponse, Protocol};

pub struct Curl {}

impl Protocol for Curl {
    fn decode(raw: &[u8]) -> Result<Command, String> {
        let request = String::from_utf8(raw.to_vec()).map_err(|_| "Error while decoding RESP")?;
        let request = request.trim();
        let mut lines = request.lines();
        let first_line = lines.next().ok_or("invalid request")?;
        let mut parts = first_line.split_whitespace();
        let method = parts.next().ok_or("invalid request")?;
        let path = parts.next().ok_or("invalid request")?;
        let _version = parts.next().ok_or("invalid request")?;
        let mut headers = std::collections::HashMap::new();
        let mut body = None;
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                body = Some(line.to_string());
            }
        }

        Ok(Command::new(method, path, body))
    }

    fn encode(response: CommandResponse) -> Vec<u8> {
        match response {
            CommandResponse::Ok(body) => {
                format!("HTTP/1.1 200 OK\r\n\r\n{body}").as_bytes().to_vec()
            }
            CommandResponse::Err(body) => format!("HTTP/1.1 422 Error\r\n\r\n{body}")
                .as_bytes()
                .to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = Curl::decode(raw.as_bytes()).unwrap();
        assert!(
            output
                == Command::Get {
                    key: "key".to_string()
                }
        );

        // assert!(
        //     output.method.unwrap()
        //         == Operation::Get {
        //             key: "key".to_string()
        //         }
        // );
        // assert_eq!(output.path, "/key");
        // assert_eq!(output.version, "HTTP/1.1");
        // assert_eq!(output.headers.len(), 3);
        // assert_eq!(output.headers.get("Host").unwrap(), "127.0.0.1:9999");
        // assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        // assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
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
        assert!(
            output
                == Command::Set {
                    key: "key".to_string(),
                    value: "value".to_string()
                }
        );
        // assert_eq!(output.path, "/key");
        // assert_eq!(output.headers.len(), 5);
        // assert_eq!(output.headers.get("Host").unwrap(), "localhost:9999");
        // assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        // assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
        // assert_eq!(output.headers.get("Content-Length").unwrap(), "5");
        // assert_eq!(
        //     output.headers.get("Content-Type").unwrap(),
        //     "application/x-www-form-urlencoded"
        // );
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
        assert!(
            output
                == Command::Del {
                    key: "key".to_string(),
                }
        );
        // assert_eq!(output.path, "/key");
        // assert_eq!(output.headers.len(), 5);
        // assert_eq!(output.headers.get("Host").unwrap(), "localhost:9999");
        // assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        // assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
        // assert_eq!(output.headers.get("Content-Length").unwrap(), "5");
        // assert_eq!(
        //     output.headers.get("Content-Type").unwrap(),
        //     "application/x-www-form-urlencoded"
        // );
    }
}
