use crate::protocol::Protocol;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RequestKind {
    /// Http kind is used to handle the HTTP requests using CURL.
    #[default]
    Http,

    /// `RedisCLI` kind is used to handle the Redis CLI requests, which is a special protocol designed for redis.
    RedisCLI,
}

#[derive(Debug)]
pub struct Request {
    pub kind: RequestKind,
    pub cmd: crate::protocol::commands::Command,
}

pub fn parse_request(raw_request: &[u8]) -> Result<Request, String> {
    // TODO: This should be optimize because in this way we try to do 2 decoding instead of stopping at first, bonus,
    // http can be under feature flag in order to skip when we are stable.
    // Maybe filter if it start with GET/POST/PUT/DELETE or not.
    let http_cmd = crate::protocol::curl::Curl::decode(raw_request);
    let redis_cli_cmd = crate::protocol::resp::Resp::decode(raw_request);

    match (http_cmd, redis_cli_cmd) {
        (Err(crate::protocol::ProtocolError::CurlProtocolDecodingError), r) => Ok(Request {
            kind: RequestKind::RedisCLI,
            cmd: r.map_err(|e| e.to_string())?,
        }),
        (r, Err(crate::protocol::ProtocolError::RespProtocolDecodingError)) => Ok(Request {
            kind: RequestKind::Http,
            cmd: r.map_err(|e| e.to_string())?,
        }),
        _ => panic!("all situations should be handled"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_redis_cli() {
        // NOTE: This is the first command redis-cli sends.
        let raw = "*2\r\n$7\r\nCOMMAND\r\n$4\r\nDOCS\r\n";
        let output = parse_request(raw.as_bytes()).unwrap();
        assert_eq!(output.kind, RequestKind::RedisCLI);
    }

    #[test]
    fn parse_http() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = parse_request(raw.as_bytes()).unwrap();
        assert_eq!(output.kind, RequestKind::Http);
    }

    #[test]
    fn parse_invalid_request() {
        let raw = "invalid";
        let output = parse_request(raw.as_bytes());
        assert!(output.is_err());
        assert_eq!(output.err().unwrap(), "resp protocol decoding error");

        let raw = "*3\r\n$3\r\nNOTACOMMAND\r\n$1\r\nx\r\n$2\r\n11\r\n";
        let output = parse_request(raw.as_bytes());
        assert!(output.is_err());
        assert_eq!(output.err().unwrap(), "command not recognized notacommand");
    }
}
