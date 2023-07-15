use std::sync::{Arc, Mutex};

use engine::db::{HashMapDb, KeyValueStore};

mod engine;
mod protocol;

use monoio::io::{AsyncReadRent, AsyncWriteRentExt};

#[monoio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9999".to_string());
    let listener = monoio::net::TcpListener::bind(addr).unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    let db = Arc::new(Mutex::new(HashMapDb::new()));

    loop {
        let incoming = listener.accept().await;
        let db = db.clone();

        monoio::spawn(async move {
            match incoming {
                Ok((mut stream, addr)) => {
                    println!("accepted a connection from {}", addr);
                    let buf: Vec<u8> = Vec::with_capacity(8 * 1024);
                    let (res, b) = stream.read(buf).await;
                    if res.is_ok() {
                        let content = String::from_utf8_lossy(&b[..]);
                        let request = parse_request(content.to_string()).unwrap();
                        let mut db = db.lock().unwrap();

                        let response = match request.method {
                            Operation::Get { key } => {
                                println!("GET detected {key}");

                                db.get(key.as_str()).map_or("not found", |v| v)
                            }
                            Operation::Set { key, value } => {
                                println!("SET detected {key} - {value}");

                                db.set(key.as_str(), value);

                                "ok"
                            }
                            Operation::Del { key } => {
                                println!("DEL detected {key}");

                                db.del(key.as_str());

                                "ok"
                            }
                        };

                        let (res, _) = stream
                            .write_all(
                                format!("HTTP/1.1 200 OK\r\n\r\n{}", response.to_string())
                                    .into_bytes(),
                            )
                            .await;
                        match res {
                            Ok(_) => (),
                            Err(e) => println!("error on stream write: {}", e),
                        }
                    }
                }
                Err(e) => {
                    println!("accepted connection failed: {}", e);
                    return;
                }
            }
        });
    }
}

#[derive(PartialEq, Eq)]
enum Operation {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
}

impl Operation {
    fn new(kind: &str, key: &str, value: Option<String>) -> Self {
        let key = key.trim_matches('/').to_string();
        match kind {
            "GET" => Operation::Get { key },
            "POST" | "SET" => match value {
                Some(v) => Operation::Set {
                    key,
                    value: v.trim().to_string(),
                },
                None => Operation::Del { key },
            },
            "DELETE" | "DEL" => Operation::Del { key },
            _ => unimplemented!("not implemented"),
        }
    }
}

struct Request {
    method: Operation,
    path: String,
    version: String,
    headers: std::collections::HashMap<String, String>,
}

fn parse_request(request: String) -> Result<Request, String> {
    let request = request.trim();
    println!("req -> {request}");
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap();
    let path = parts.next().unwrap();
    let version = parts.next().unwrap();
    let mut headers = std::collections::HashMap::new();
    let mut body = None;
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            body = Some(line.to_string());
        }
    }

    Ok(Request {
        method: Operation::new(method, path, body),
        path: path.to_string(),
        version: version.to_string(),
        headers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_parse_get() {
        let raw = r#"GET /key HTTP/1.1
Host: 127.0.0.1:9999
User-Agent: curl/7.74.0
Accept: */*
"#;

        let output = parse_request(raw.to_string()).unwrap();

        assert!(
            output.method
                == Operation::Get {
                    key: "key".to_string()
                }
        );
        assert_eq!(output.path, "/key");
        assert_eq!(output.version, "HTTP/1.1");
        assert_eq!(output.headers.len(), 3);
        assert_eq!(output.headers.get("Host").unwrap(), "127.0.0.1:9999");
        assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
    }

    #[test]
    fn check_parse_set() {
        let raw = r#"POST /key HTTP/1.1
        Host: localhost:9999
        User-Agent: curl/7.74.0
        Accept: */*
        Content-Length: 5
        Content-Type: application/x-www-form-urlencoded
        
        value"#;

        let output = parse_request(raw.to_string()).unwrap();
        assert!(
            output.method
                == Operation::Set {
                    key: "key".to_string(),
                    value: "value".to_string()
                }
        );
        assert_eq!(output.path, "/key");
        assert_eq!(output.headers.len(), 5);
        assert_eq!(output.headers.get("Host").unwrap(), "localhost:9999");
        assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
        assert_eq!(output.headers.get("Content-Length").unwrap(), "5");
        assert_eq!(
            output.headers.get("Content-Type").unwrap(),
            "application/x-www-form-urlencoded"
        );
    }

    #[test]
    fn check_parse_del() {
        let raw = r#"POST /key HTTP/1.1
        Host: localhost:9999
        User-Agent: curl/7.74.0
        Accept: */*
        Content-Length: 5
        Content-Type: application/x-www-form-urlencoded
"#;

        let output = parse_request(raw.to_string()).unwrap();
        assert!(
            output.method
                == Operation::Del {
                    key: "key".to_string(),
                }
        );
        assert_eq!(output.path, "/key");
        assert_eq!(output.headers.len(), 5);
        assert_eq!(output.headers.get("Host").unwrap(), "localhost:9999");
        assert_eq!(output.headers.get("User-Agent").unwrap(), "curl/7.74.0");
        assert_eq!(output.headers.get("Accept").unwrap(), "*/*");
        assert_eq!(output.headers.get("Content-Length").unwrap(), "5");
        assert_eq!(
            output.headers.get("Content-Type").unwrap(),
            "application/x-www-form-urlencoded"
        );
    }
}
